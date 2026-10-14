use async_trait::async_trait;
use futures::StreamExt;
use opendal::services::{Fs, S3};
use opendal::{EntryMode, Operator};
use serde::{Deserialize, Serialize};
use spoty_core::{AudioFormat, Track, TrackId};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::provider::{AudioStream, Library, MusicProvider};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Config {
    pub bucket: String,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub root: Option<String>,
}

pub struct OpenDalProvider {
    id: String,
    display_name: String,
    operator: Operator,
    library_cache: Arc<RwLock<Option<Library>>>,
    path_map: Arc<RwLock<HashMap<TrackId, String>>>,
}

impl OpenDalProvider {
    /// Create a new OpenDAL provider backed by a local filesystem directory
    pub fn new_fs(root_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root_str = root_dir.as_ref().to_string_lossy().to_string();
        let builder = Fs::default().root(&root_str);
        let operator = Operator::new(builder)?;

        Ok(Self {
            id: format!("fs_{}", root_str.replace('/', "_")),
            display_name: format!("Local Folder ({})", root_str),
            operator,
            library_cache: Arc::new(RwLock::new(None)),
            path_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new OpenDAL provider backed by AWS S3, Cloudflare R2, or MinIO
    pub fn new_s3(config: S3Config) -> anyhow::Result<Self> {
        let mut builder = S3::default().bucket(&config.bucket);
        if let Some(endpoint) = &config.endpoint {
            builder = builder.endpoint(endpoint);
        }
        if let Some(region) = &config.region {
            builder = builder.region(region);
        }
        if let Some(key) = &config.access_key_id {
            builder = builder.access_key_id(key);
        }
        if let Some(secret) = &config.secret_access_key {
            builder = builder.secret_access_key(secret);
        }
        if let Some(root) = &config.root {
            builder = builder.root(root);
        }

        let operator = Operator::new(builder)?;
        let display_name = format!("S3 / R2 ({})", config.bucket);
        let id = format!("s3_{}", config.bucket);

        Ok(Self::new(id, display_name, operator))
    }

    /// Create from any configured OpenDAL operator
    pub fn new(id: impl Into<String>, display_name: impl Into<String>, operator: Operator) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            operator,
            library_cache: Arc::new(RwLock::new(None)),
            path_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MusicProvider for OpenDalProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    async fn fetch_library(&self) -> anyhow::Result<Library> {
        if let Some(cached) = self.library_cache.read().await.as_ref() {
            return Ok(cached.clone());
        }

        let mut tracks = Vec::new();
        let mut path_map = HashMap::new();

        // Scan storage operator recursively
        let entries = match self.operator.lister_with("").recursive(true).await {
            Ok(mut lister) => {
                let mut list = Vec::new();
                while let Some(res) = lister.next().await {
                    if let Ok(entry) = res {
                        list.push(entry);
                    }
                }
                list
            }
            Err(e) => {
                tracing::warn!("Failed to list storage directory: {}", e);
                Vec::new()
            }
        };

        for entry in entries {
            let path = entry.path().to_string();
            let metadata = entry.metadata();

            if metadata.mode() == EntryMode::FILE {
                if let Some(ext) = Path::new(&path).extension().and_then(|s| s.to_str()) {
                    let format = AudioFormat::from_extension(ext);
                    if format != AudioFormat::Unknown {
                        let filename = Path::new(&path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown Title");

                        let (artist, title) = parse_filename_metadata(filename);
                        let track_id = TrackId::new(format!("{}:{}", self.id, path));

                        path_map.insert(track_id.clone(), path.clone());

                        tracks.push(Track {
                            id: track_id,
                            title,
                            artist,
                            album: None,
                            duration_secs: 180,
                            cover_uri: None,
                            source_uri: format!("opendal://{}/{}", self.id, path),
                            format,
                        });
                    }
                }
            }
        }

        let library = Library {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        };

        *self.library_cache.write().await = Some(library.clone());
        *self.path_map.write().await = path_map;

        Ok(library)
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>> {
        let lib = self.fetch_library().await?;
        let q = query.to_lowercase();
        Ok(lib
            .tracks
            .into_iter()
            .filter(|t| t.title.to_lowercase().contains(&q) || t.artist.to_lowercase().contains(&q))
            .collect())
    }

    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream> {
        let path_guard = self.path_map.read().await;
        let path = path_guard
            .get(track_id)
            .ok_or_else(|| anyhow::anyhow!("Track not found in provider path map: {}", track_id))?;

        let bytes = self.operator.read(path).await?.to_vec();
        let cursor = std::io::Cursor::new(bytes);
        Ok(Box::pin(cursor))
    }
}

/// Simple heuristic to extract "Artist - Title" from filenames
fn parse_filename_metadata(stem: &str) -> (String, String) {
    if let Some((artist, title)) = stem.split_once(" - ") {
        (artist.trim().to_string(), title.trim().to_string())
    } else if let Some((artist, title)) = stem.split_once('_') {
        (artist.trim().to_string(), title.trim().to_string())
    } else {
        ("Unknown Artist".to_string(), stem.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config_builder() {
        let config = S3Config {
            bucket: "my-music-bucket".into(),
            endpoint: Some("https://s3.us-west-002.backblazeb2.com".into()),
            region: Some("us-west-002".into()),
            access_key_id: Some("test_key".into()),
            secret_access_key: Some("test_secret".into()),
            root: Some("music/".into()),
        };

        let provider = OpenDalProvider::new_s3(config).unwrap();
        assert_eq!(provider.id(), "s3_my-music-bucket");
        assert_eq!(provider.display_name(), "S3 / R2 (my-music-bucket)");
    }
}
