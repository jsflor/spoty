use async_trait::async_trait;
use spoty_core::{Album, Artist, Playlist, Track, TrackId};
use std::pin::Pin;
use tokio::io::AsyncRead;

pub type AudioStream = Pin<Box<dyn AsyncRead + Send>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Library {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[async_trait]
pub trait MusicProvider: Send + Sync {
    /// Unique machine identifier for this provider (e.g. "demo", "local_fs", "s3_main", "spotify")
    fn id(&self) -> &str;

    /// Human-friendly display label (e.g. "Local Music", "Demo Catalog", "Spotify Connect")
    fn display_name(&self) -> &str;

    /// Retrieve the catalog or collection from this provider
    async fn fetch_library(&self) -> anyhow::Result<Library>;

    /// Search tracks within this provider's catalog
    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>>;

    /// Resolve an asynchronous audio byte stream for the given track
    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream>;
}
