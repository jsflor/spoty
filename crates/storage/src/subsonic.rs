use async_trait::async_trait;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use spoty_core::{AudioFormat, Playlist, PlaylistId, Track, TrackId};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::provider::{AudioStream, Library, MusicProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

impl SubsonicConfig {
    pub fn new(server_url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            username: username.into(),
            password: password.into(),
        }
    }

    /// Compute token and salt according to Subsonic API specification (v1.13+)
    pub fn auth_params(&self, salt: &str) -> (String, String) {
        let mut hasher = Md5::new();
        hasher.update(format!("{}{}", self.password, salt).as_bytes());
        let result = hasher.finalize();
        let token = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (token, salt.to_string())
    }

    pub fn endpoint_url(&self, view: &str) -> String {
        let salt = "spoty_client_salt";
        let (token, salt_str) = self.auth_params(salt);
        let base = self.server_url.trim_end_matches('/');
        format!(
            "{}/rest/{}.view?u={}&t={}&s={}&v=1.16.1&c=Spoty&f=json",
            base, view, self.username, token, salt_str
        )
    }
}

pub struct SubsonicProvider {
    config: SubsonicConfig,
    library_cache: Arc<RwLock<Option<Library>>>,
    subsonic_id_map: Arc<RwLock<HashMap<TrackId, String>>>,
}

impl SubsonicProvider {
    pub fn new(config: SubsonicConfig) -> Self {
        Self {
            config,
            library_cache: Arc::new(RwLock::new(None)),
            subsonic_id_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Test server reachability with ping.view
    pub fn ping(&self) -> anyhow::Result<bool> {
        let url = self.config.endpoint_url("ping");
        let resp: serde_json::Value = ureq::get(&url).timeout(std::time::Duration::from_secs(5)).call()?.into_json()?;
        let status = resp["subsonic-response"]["status"].as_str().unwrap_or("failed");
        Ok(status == "ok")
    }
}

#[async_trait]
impl MusicProvider for SubsonicProvider {
    fn id(&self) -> &str {
        "subsonic"
    }

    fn display_name(&self) -> &str {
        "Navidrome / Subsonic"
    }

    async fn fetch_library(&self) -> anyhow::Result<Library> {
        if let Some(cached) = self.library_cache.read().await.as_ref() {
            return Ok(cached.clone());
        }

        let mut tracks = Vec::new();
        let mut playlists = Vec::new();
        let mut id_map = HashMap::new();

        // 1. Fetch playlists from /rest/getPlaylists.view
        let playlists_url = self.config.endpoint_url("getPlaylists");
        if let Ok(resp) = ureq::get(&playlists_url).call() {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                if let Some(items) = json["subsonic-response"]["playlists"]["playlist"].as_array() {
                    for item in items {
                        let id = item["id"].as_str().unwrap_or("").to_string();
                        let name = item["name"].as_str().unwrap_or("Untitled").to_string();
                        let comment = item["comment"].as_str().map(|s| s.to_string());

                        playlists.push(Playlist {
                            id: PlaylistId::new(format!("subsonic_pl_{}", id)),
                            title: name,
                            description: comment,
                            cover_uri: None,
                            track_ids: Vec::new(),
                        });
                    }
                }
            }
        }

        // 2. Fetch random songs sample to populate library catalog
        let random_url = format!("{}&size=50", self.config.endpoint_url("getRandomSongs"));
        if let Ok(resp) = ureq::get(&random_url).call() {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                if let Some(songs) = json["subsonic-response"]["randomSongs"]["song"].as_array() {
                    for s in songs {
                        let sid = s["id"].as_str().unwrap_or("").to_string();
                        let title = s["title"].as_str().unwrap_or("Unknown Title").to_string();
                        let artist = s["artist"].as_str().unwrap_or("Unknown Artist").to_string();
                        let album = s["album"].as_str().map(|a| a.to_string());
                        let duration = s["duration"].as_u64().unwrap_or(180) as u32;
                        let suffix = s["suffix"].as_str().unwrap_or("mp3");

                        let track_id = TrackId::new(format!("subsonic_{}", sid));
                        id_map.insert(track_id.clone(), sid.clone());

                        tracks.push(Track {
                            id: track_id,
                            title,
                            artist,
                            album,
                            duration_secs: duration,
                            cover_uri: None,
                            source_uri: format!("subsonic://{}", sid),
                            format: AudioFormat::from_extension(suffix),
                        });
                    }
                }
            }
        }

        let library = Library {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists,
        };

        *self.library_cache.write().await = Some(library.clone());
        *self.subsonic_id_map.write().await = id_map;

        Ok(library)
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>> {
        let search_url = format!("{}&query={}&songCount=30", self.config.endpoint_url("search3"), query);
        let resp: serde_json::Value = ureq::get(&search_url).call()?.into_json()?;

        let mut results = Vec::new();
        if let Some(songs) = resp["subsonic-response"]["searchResult3"]["song"].as_array() {
            for s in songs {
                let sid = s["id"].as_str().unwrap_or("").to_string();
                let title = s["title"].as_str().unwrap_or("Unknown Title").to_string();
                let artist = s["artist"].as_str().unwrap_or("Unknown Artist").to_string();
                let album = s["album"].as_str().map(|a| a.to_string());
                let duration = s["duration"].as_u64().unwrap_or(180) as u32;
                let suffix = s["suffix"].as_str().unwrap_or("mp3");

                let track_id = TrackId::new(format!("subsonic_{}", sid));
                self.subsonic_id_map.write().await.insert(track_id.clone(), sid.clone());

                results.push(Track {
                    id: track_id,
                    title,
                    artist,
                    album,
                    duration_secs: duration,
                    cover_uri: None,
                    source_uri: format!("subsonic://{}", sid),
                    format: AudioFormat::from_extension(suffix),
                });
            }
        }

        Ok(results)
    }

    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream> {
        let guard = self.subsonic_id_map.read().await;
        let sid = guard
            .get(track_id)
            .ok_or_else(|| anyhow::anyhow!("Track ID not found in Subsonic map: {}", track_id))?;

        let stream_url = format!("{}&id={}", self.config.endpoint_url("stream"), sid);
        let response = ureq::get(&stream_url).call()?;
        let mut bytes = Vec::new();
        response.into_reader().read_to_end(&mut bytes)?;

        let cursor = Cursor::new(bytes);
        Ok(Box::pin(cursor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsonic_auth_token_calculation() {
        let config = SubsonicConfig::new("http://localhost:4533", "admin", "secret123");
        let (token, salt) = config.auth_params("test_salt");

        assert_eq!(salt, "test_salt");
        // md5("secret123test_salt")
        let mut hasher = Md5::new();
        hasher.update(b"secret123test_salt");
        let result = hasher.finalize();
        let expected = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        assert_eq!(token, expected);
    }

    #[test]
    fn test_subsonic_endpoint_url() {
        let config = SubsonicConfig::new("https://navidrome.home.local", "user", "pass");
        let url = config.endpoint_url("ping");

        assert!(url.starts_with("https://navidrome.home.local/rest/ping.view?u=user&t="));
        assert!(url.contains("&c=Spoty&f=json"));
    }
}
