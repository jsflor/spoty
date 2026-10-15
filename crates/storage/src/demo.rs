use crate::provider::{AudioStream, Library, MusicProvider};
use async_trait::async_trait;
use spoty_core::{
    Album, AlbumId, Artist, ArtistId, AudioFormat, Playlist, PlaylistId, Track, TrackId,
};
use std::collections::HashMap;
use std::io::Cursor;

const TRACK_1_BYTES: &[u8] = include_bytes!("../assets/demo/consort_for_brass.ogg");
const TRACK_2_BYTES: &[u8] = include_bytes!("../assets/demo/the_rule.ogg");
const TRACK_3_BYTES: &[u8] = include_bytes!("../assets/demo/monkeys_spinning_monkeys.ogg");

pub struct DemoProvider {
    library: Library,
    audio_data: HashMap<TrackId, &'static [u8]>,
}

impl Default for DemoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoProvider {
    pub fn new() -> Self {
        let mut audio_data = HashMap::new();

        let track1_id = TrackId::new("demo_track_1");
        let track2_id = TrackId::new("demo_track_2");
        let track3_id = TrackId::new("demo_track_3");

        // Embed high-quality Creative Commons (CC-BY) audio tracks by Kevin MacLeod
        audio_data.insert(track1_id.clone(), TRACK_1_BYTES);
        audio_data.insert(track2_id.clone(), TRACK_2_BYTES);
        audio_data.insert(track3_id.clone(), TRACK_3_BYTES);

        let tracks = vec![
            Track {
                id: track1_id.clone(),
                title: "Consort for Brass".into(),
                artist: "Kevin MacLeod".into(),
                album: Some("Classical Sampler".into()),
                duration_secs: 51,
                cover_uri: Some(
                    "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=300&q=80"
                        .into(),
                ),
                source_uri: "demo://consort_for_brass".into(),
                format: AudioFormat::Ogg,
            },
            Track {
                id: track2_id.clone(),
                title: "The Rule".into(),
                artist: "Kevin MacLeod".into(),
                album: Some("Classical Sampler".into()),
                duration_secs: 57,
                cover_uri: Some(
                    "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=300&q=80"
                        .into(),
                ),
                source_uri: "demo://the_rule".into(),
                format: AudioFormat::Ogg,
            },
            Track {
                id: track3_id.clone(),
                title: "Monkeys Spinning Monkeys".into(),
                artist: "Kevin MacLeod".into(),
                album: Some("Music for Comedy".into()),
                duration_secs: 125,
                cover_uri: Some(
                    "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=300&q=80"
                        .into(),
                ),
                source_uri: "demo://monkeys_spinning_monkeys".into(),
                format: AudioFormat::Ogg,
            },
        ];

        let albums = vec![
            Album {
                id: AlbumId::new("demo_album_1"),
                title: "Classical Sampler".into(),
                artist: "Kevin MacLeod".into(),
                year: Some(2011),
                cover_uri: Some(
                    "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=300&q=80"
                        .into(),
                ),
                track_ids: vec![track1_id.clone(), track2_id.clone()],
            },
            Album {
                id: AlbumId::new("demo_album_2"),
                title: "Music for Comedy".into(),
                artist: "Kevin MacLeod".into(),
                year: Some(2014),
                cover_uri: Some(
                    "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=300&q=80"
                        .into(),
                ),
                track_ids: vec![track3_id.clone()],
            },
        ];

        let artists = vec![Artist {
            id: ArtistId::new("demo_artist_1"),
            name: "Kevin MacLeod".into(),
            avatar_uri: Some(
                "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=300&q=80".into(),
            ),
        }];

        let playlists = vec![Playlist {
            id: PlaylistId::new("demo_playlist_favorites"),
            title: "Spoty Favorites".into(),
            description: Some("Curated showcase royalty-free tracks by Kevin MacLeod".into()),
            cover_uri: Some(
                "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=300&q=80".into(),
            ),
            track_ids: vec![track1_id, track2_id, track3_id],
        }];

        Self {
            library: Library {
                tracks,
                albums,
                artists,
                playlists,
            },
            audio_data,
        }
    }
}

#[async_trait]
impl MusicProvider for DemoProvider {
    fn id(&self) -> &str {
        "demo"
    }

    fn display_name(&self) -> &str {
        "Demo Catalog"
    }

    async fn fetch_library(&self) -> anyhow::Result<Library> {
        Ok(self.library.clone())
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>> {
        let q = query.to_lowercase();
        let matches = self
            .library
            .tracks
            .iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&q)
                    || t.artist.to_lowercase().contains(&q)
                    || t.album.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        Ok(matches)
    }

    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream> {
        match self.audio_data.get(track_id) {
            Some(data) => {
                let cursor = Cursor::new(data.to_vec());
                Ok(Box::pin(cursor))
            }
            None => anyhow::bail!("Track not found in demo provider: {}", track_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::extract_audio_metadata;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_demo_provider_library() {
        let provider = DemoProvider::new();
        let library = provider.fetch_library().await.unwrap();

        assert_eq!(library.tracks.len(), 3);
        assert_eq!(library.albums.len(), 2);
        assert_eq!(library.artists.len(), 1);
        assert_eq!(library.artists[0].name, "Kevin MacLeod");
        assert_eq!(library.playlists.len(), 1);
        assert_eq!(library.playlists[0].track_ids.len(), 3);
        assert_eq!(library.tracks[0].format, AudioFormat::Ogg);
    }

    #[tokio::test]
    async fn test_demo_provider_search() {
        let provider = DemoProvider::new();
        let matches = provider.search("Brass").await.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].title, "Consort for Brass");

        let artist_matches = provider.search("MacLeod").await.unwrap();
        assert_eq!(artist_matches.len(), 3);
        assert_eq!(artist_matches[0].artist, "Kevin MacLeod");
    }

    #[tokio::test]
    async fn test_demo_provider_stream_track() {
        let provider = DemoProvider::new();
        let track_id = TrackId::new("demo_track_1");
        let mut stream = provider.stream_track(&track_id).await.unwrap();

        let mut header = [0u8; 4];
        stream.read_exact(&mut header).await.unwrap();
        // Ogg Vorbis container header starts with "OggS"
        assert_eq!(&header, b"OggS");
    }

    #[tokio::test]
    async fn test_demo_provider_audio_metadata_extraction() {
        let metadata = extract_audio_metadata(TRACK_1_BYTES, "consort_for_brass.ogg");
        assert_eq!(metadata.duration_secs, 50); // Lofty extracts ~50s for Consort for Brass
        assert_eq!(metadata.artist, "Kevin MacLeod");
    }
}
