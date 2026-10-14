use async_trait::async_trait;
use spoty_core::{Album, AlbumId, Artist, ArtistId, Playlist, Track, TrackId};
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

impl Library {
    /// Dynamically constructs unique Artist and Album collections from tracks
    pub fn aggregate_artists_and_albums(&mut self) {
        use std::collections::BTreeMap;

        let mut artist_map: BTreeMap<String, Vec<TrackId>> = BTreeMap::new();
        let mut album_map: BTreeMap<(String, String), Vec<TrackId>> = BTreeMap::new();

        for track in &self.tracks {
            let artist_name = track.artist.trim();
            if !artist_name.is_empty() {
                artist_map
                    .entry(artist_name.to_string())
                    .or_default()
                    .push(track.id.clone());
            }

            if let Some(album_name) = &track.album {
                let alb = album_name.trim();
                if !alb.is_empty() {
                    album_map
                        .entry((alb.to_string(), artist_name.to_string()))
                        .or_default()
                        .push(track.id.clone());
                }
            }
        }

        if self.artists.is_empty() {
            self.artists = artist_map
                .into_iter()
                .map(|(name, _)| {
                    let slug = name
                        .to_lowercase()
                        .replace(|c: char| !c.is_alphanumeric(), "_");
                    Artist {
                        id: ArtistId::new(format!("art_{}", slug)),
                        name,
                        avatar_uri: None,
                    }
                })
                .collect();
        }

        if self.albums.is_empty() {
            self.albums = album_map
                .into_iter()
                .map(|((title, artist), tids)| {
                    let slug = title
                        .to_lowercase()
                        .replace(|c: char| !c.is_alphanumeric(), "_");
                    Album {
                        id: AlbumId::new(format!("alb_{}", slug)),
                        title,
                        artist,
                        year: None,
                        cover_uri: None,
                        track_ids: tids,
                    }
                })
                .collect();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use spoty_core::AudioFormat;

    #[test]
    fn test_library_aggregate_artists_and_albums() {
        let mut lib = Library {
            tracks: vec![
                Track {
                    id: TrackId::new("t1"),
                    title: "Song 1".into(),
                    artist: "Daft Punk".into(),
                    album: Some("Discovery".into()),
                    duration_secs: 200,
                    cover_uri: None,
                    source_uri: "".into(),
                    format: AudioFormat::Mp3,
                },
                Track {
                    id: TrackId::new("t2"),
                    title: "Song 2".into(),
                    artist: "Daft Punk".into(),
                    album: Some("Discovery".into()),
                    duration_secs: 180,
                    cover_uri: None,
                    source_uri: "".into(),
                    format: AudioFormat::Mp3,
                },
                Track {
                    id: TrackId::new("t3"),
                    title: "Song 3".into(),
                    artist: "Queen".into(),
                    album: Some("A Night at the Opera".into()),
                    duration_secs: 350,
                    cover_uri: None,
                    source_uri: "".into(),
                    format: AudioFormat::Mp3,
                },
            ],
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        };

        lib.aggregate_artists_and_albums();
        assert_eq!(lib.artists.len(), 2);
        assert!(lib.artists.iter().any(|a| a.name == "Daft Punk"));
        assert!(lib.artists.iter().any(|a| a.name == "Queen"));

        assert_eq!(lib.albums.len(), 2);
        let discovery = lib.albums.iter().find(|a| a.title == "Discovery").unwrap();
        assert_eq!(discovery.track_ids.len(), 2);
    }
}
