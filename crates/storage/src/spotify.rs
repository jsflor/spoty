use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};
use spoty_core::{AudioFormat, Playlist, PlaylistId, Track, TrackId};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::provider::{AudioStream, Library, MusicProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub device_name: String,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            device_name: "Spoty Desktop".into(),
        }
    }
}

impl SpotifyConfig {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            device_name: "Spoty Desktop".into(),
        }
    }

    /// Generates the HTTP Basic Authorization header value for Spotify token exchange
    pub fn basic_auth_header(&self) -> String {
        let creds = format!("{}:{}", self.client_id, self.client_secret);
        let encoded = base64::engine::general_purpose::STANDARD.encode(creds.as_bytes());
        format!("Basic {}", encoded)
    }
}

pub struct SpotifyProvider {
    config: SpotifyConfig,
    cached_token: Arc<RwLock<Option<(String, Instant)>>>,
    library_cache: Arc<RwLock<Option<Library>>>,
    stream_urls: Arc<RwLock<HashMap<TrackId, String>>>,
}

impl SpotifyProvider {
    pub fn new(config: SpotifyConfig) -> Self {
        Self {
            config,
            cached_token: Arc::new(RwLock::new(None)),
            library_cache: Arc::new(RwLock::new(None)),
            stream_urls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Obtains or refreshes the Spotify OAuth access token via Client Credentials flow
    pub async fn get_access_token(&self) -> anyhow::Result<String> {
        let mut lock = self.cached_token.write().await;
        if let Some((token, expiry)) = lock.as_ref() {
            if Instant::now() < *expiry {
                return Ok(token.clone());
            }
        }

        let basic_auth = self.config.basic_auth_header();
        let resp: serde_json::Value = ureq::post("https://accounts.spotify.com/api/token")
            .set("Authorization", &basic_auth)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string("grant_type=client_credentials")?
            .into_json()?;

        let access_token = resp["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to obtain Spotify access token"))?
            .to_string();

        let expires_in = resp["expires_in"].as_u64().unwrap_or(3600);
        let expiry = Instant::now() + Duration::from_secs(expires_in.saturating_sub(60));

        *lock = Some((access_token.clone(), expiry));
        Ok(access_token)
    }
}

#[async_trait]
impl MusicProvider for SpotifyProvider {
    fn id(&self) -> &str {
        "spotify"
    }

    fn display_name(&self) -> &str {
        "Spotify"
    }

    async fn fetch_library(&self) -> anyhow::Result<Library> {
        if let Some(cached) = self.library_cache.read().await.as_ref() {
            return Ok(cached.clone());
        }

        let token = self.get_access_token().await?;
        let resp: serde_json::Value = ureq::get("https://api.spotify.com/v1/browse/featured-playlists?limit=10")
            .set("Authorization", &format!("Bearer {}", token))
            .call()?
            .into_json()?;

        let mut playlists = Vec::new();
        let mut tracks = Vec::new();

        if let Some(items) = resp["playlists"]["items"].as_array() {
            for item in items {
                let pid = item["id"].as_str().unwrap_or("").to_string();
                let name = item["name"].as_str().unwrap_or("Featured").to_string();
                let desc = item["description"].as_str().map(|s| s.to_string());
                let cover = item["images"][0]["url"].as_str().map(|s| s.to_string());

                playlists.push(Playlist {
                    id: PlaylistId::new(format!("spotify_pl_{}", pid)),
                    title: name,
                    description: desc,
                    cover_uri: cover,
                    track_ids: Vec::new(),
                });
            }
        }

        // Fetch top hits to populate default library tracks
        let hits_resp: serde_json::Value = ureq::get("https://api.spotify.com/v1/search?q=top&type=track&limit=25")
            .set("Authorization", &format!("Bearer {}", token))
            .call()?
            .into_json()?;

        if let Some(items) = hits_resp["tracks"]["items"].as_array() {
            for item in items {
                let sid = item["id"].as_str().unwrap_or("").to_string();
                let title = item["name"].as_str().unwrap_or("Unknown").to_string();
                let artist = item["artists"][0]["name"].as_str().unwrap_or("Unknown").to_string();
                let album = item["album"]["name"].as_str().map(|s| s.to_string());
                let duration_ms = item["duration_ms"].as_u64().unwrap_or(180000);
                let cover = item["album"]["images"][0]["url"].as_str().map(|s| s.to_string());
                let preview_url = item["preview_url"].as_str().map(|s| s.to_string());

                let track_id = TrackId::new(format!("spotify_{}", sid));

                if let Some(url) = preview_url {
                    self.stream_urls.write().await.insert(track_id.clone(), url);
                }

                tracks.push(Track {
                    id: track_id,
                    title,
                    artist,
                    album,
                    duration_secs: (duration_ms / 1000) as u32,
                    cover_uri: cover,
                    source_uri: format!("spotify:track:{}", sid),
                    format: AudioFormat::Mp3,
                });
            }
        }

        let library = Library {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists,
        };

        *self.library_cache.write().await = Some(library.clone());
        Ok(library)
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>> {
        let token = self.get_access_token().await?;
        let encoded_q = query.replace(' ', "+");
        let search_url = format!("https://api.spotify.com/v1/search?q={}&type=track&limit=20", encoded_q);

        let resp: serde_json::Value = ureq::get(&search_url)
            .set("Authorization", &format!("Bearer {}", token))
            .call()?
            .into_json()?;

        let mut results = Vec::new();
        if let Some(items) = resp["tracks"]["items"].as_array() {
            for item in items {
                let sid = item["id"].as_str().unwrap_or("").to_string();
                let title = item["name"].as_str().unwrap_or("Unknown").to_string();
                let artist = item["artists"][0]["name"].as_str().unwrap_or("Unknown").to_string();
                let album = item["album"]["name"].as_str().map(|s| s.to_string());
                let duration_ms = item["duration_ms"].as_u64().unwrap_or(180000);
                let cover = item["album"]["images"][0]["url"].as_str().map(|s| s.to_string());
                let preview_url = item["preview_url"].as_str().map(|s| s.to_string());

                let track_id = TrackId::new(format!("spotify_{}", sid));

                if let Some(url) = preview_url {
                    self.stream_urls.write().await.insert(track_id.clone(), url);
                }

                results.push(Track {
                    id: track_id,
                    title,
                    artist,
                    album,
                    duration_secs: (duration_ms / 1000) as u32,
                    cover_uri: cover,
                    source_uri: format!("spotify:track:{}", sid),
                    format: AudioFormat::Mp3,
                });
            }
        }

        Ok(results)
    }

    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream> {
        let guard = self.stream_urls.read().await;
        let url = guard
            .get(track_id)
            .ok_or_else(|| anyhow::anyhow!("Preview audio stream not available for Spotify track: {}", track_id))?;

        let response = ureq::get(url).call()?;
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
    fn test_spotify_basic_auth_header() {
        let config = SpotifyConfig::new("my_client_id", "my_secret");
        let header = config.basic_auth_header();

        assert!(header.starts_with("Basic "));
        let b64_part = header.trim_start_matches("Basic ");
        let decoded = String::from_utf8(base64::engine::general_purpose::STANDARD.decode(b64_part).unwrap()).unwrap();
        assert_eq!(decoded, "my_client_id:my_secret");
    }

    #[test]
    fn test_spotify_config_default() {
        let config = SpotifyConfig::default();
        assert_eq!(config.device_name, "Spoty Desktop");
        assert!(config.client_id.is_empty());
    }
}
