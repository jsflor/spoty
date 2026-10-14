use async_trait::async_trait;
use spoty_core::{
    Album, AlbumId, Artist, ArtistId, AudioFormat, Playlist, PlaylistId, Track, TrackId,
};
use std::collections::HashMap;
use std::io::Cursor;
use crate::provider::{AudioStream, Library, MusicProvider};

pub struct DemoProvider {
    library: Library,
    audio_data: HashMap<TrackId, Vec<u8>>,
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

        // Synthesize valid 44.1kHz stereo PCM WAV audio for each track with distinct harmonic frequencies
        audio_data.insert(track1_id.clone(), generate_pcm_wav(440.0, 5.0)); // A4 note (440Hz)
        audio_data.insert(track2_id.clone(), generate_pcm_wav(523.25, 6.0)); // C5 note (523Hz)
        audio_data.insert(track3_id.clone(), generate_pcm_wav(659.25, 4.5)); // E5 note (659Hz)

        let tracks = vec![
            Track {
                id: track1_id.clone(),
                title: "Neon Horizon".into(),
                artist: "Synthwave Collective".into(),
                album: Some("Midnight Drive".into()),
                duration_secs: 5,
                cover_uri: Some("https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=300&q=80".into()),
                source_uri: "demo://track_1".into(),
                format: AudioFormat::Wav,
            },
            Track {
                id: track2_id.clone(),
                title: "Coffee & Code".into(),
                artist: "Lo-Fi Beats".into(),
                album: Some("Study Sessions".into()),
                duration_secs: 6,
                cover_uri: Some("https://images.unsplash.com/photo-1501386761578-eac5c94b800a?w=300&q=80".into()),
                source_uri: "demo://track_2".into(),
                format: AudioFormat::Wav,
            },
            Track {
                id: track3_id.clone(),
                title: "Aurora Borealis".into(),
                artist: "Solaris".into(),
                album: Some("Atmospheres".into()),
                duration_secs: 4,
                cover_uri: Some("https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=300&q=80".into()),
                source_uri: "demo://track_3".into(),
                format: AudioFormat::Wav,
            },
        ];

        let albums = vec![
            Album {
                id: AlbumId::new("demo_album_1"),
                title: "Midnight Drive".into(),
                artist: "Synthwave Collective".into(),
                year: Some(2024),
                cover_uri: Some("https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=300&q=80".into()),
                track_ids: vec![track1_id.clone()],
            },
            Album {
                id: AlbumId::new("demo_album_2"),
                title: "Study Sessions".into(),
                artist: "Lo-Fi Beats".into(),
                year: Some(2023),
                cover_uri: Some("https://images.unsplash.com/photo-1501386761578-eac5c94b800a?w=300&q=80".into()),
                track_ids: vec![track2_id.clone()],
            },
        ];

        let artists = vec![
            Artist {
                id: ArtistId::new("demo_artist_1"),
                name: "Synthwave Collective".into(),
                avatar_uri: None,
            },
            Artist {
                id: ArtistId::new("demo_artist_2"),
                name: "Lo-Fi Beats".into(),
                avatar_uri: None,
            },
            Artist {
                id: ArtistId::new("demo_artist_3"),
                name: "Solaris".into(),
                avatar_uri: None,
            },
        ];

        let playlists = vec![
            Playlist {
                id: PlaylistId::new("demo_playlist_favorites"),
                title: "Spoty Favorites".into(),
                description: Some("Curated showcase tracks for Spoty player".into()),
                cover_uri: Some("https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=300&q=80".into()),
                track_ids: vec![track1_id, track2_id, track3_id],
            },
        ];

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
                let cursor = Cursor::new(data.clone());
                Ok(Box::pin(cursor))
            }
            None => anyhow::bail!("Track not found in demo provider: {}", track_id),
        }
    }
}

/// Generates a valid 16-bit 44.1kHz stereo PCM WAV file in memory
fn generate_pcm_wav(freq: f32, duration_secs: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let num_channels = 2u16;
    let bits_per_sample = 16u16;
    let total_samples = (sample_rate as f32 * duration_secs) as u32;
    let bytes_per_sample = bits_per_sample / 8;
    let subchunk2_size = total_samples * num_channels as u32 * bytes_per_sample as u32;
    let chunk_size = 36 + subchunk2_size;
    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
    let block_align = num_channels * bytes_per_sample;

    let mut buf = Vec::with_capacity(44 + subchunk2_size as usize);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&chunk_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // "fmt " subchunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    buf.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 for PCM)
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // "data" subchunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&subchunk2_size.to_le_bytes());

    // Write samples with envelope (fade in and fade out)
    let attack_samples = (sample_rate as f32 * 0.05) as usize; // 50ms attack
    let release_samples = (sample_rate as f32 * 0.1) as usize; // 100ms release

    for i in 0..total_samples as usize {
        let t = i as f32 / sample_rate as f32;
        // Two harmonic sines for a warmer musical timbre
        let sample_val = 0.7 * (2.0 * std::f32::consts::PI * freq * t).sin()
            + 0.3 * (4.0 * std::f32::consts::PI * freq * t).sin();

        // Envelope
        let amp = if i < attack_samples {
            i as f32 / attack_samples as f32
        } else if i > total_samples as usize - release_samples {
            (total_samples as usize - i) as f32 / release_samples as f32
        } else {
            1.0
        };

        let sample_i16 = (sample_val * amp * 0.5 * i16::MAX as f32) as i16;
        let sample_bytes = sample_i16.to_le_bytes();

        // Left channel
        buf.extend_from_slice(&sample_bytes);
        // Right channel
        buf.extend_from_slice(&sample_bytes);
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_demo_provider_library() {
        let provider = DemoProvider::new();
        let library = provider.fetch_library().await.unwrap();

        assert_eq!(library.tracks.len(), 3);
        assert_eq!(library.albums.len(), 2);
        assert_eq!(library.playlists.len(), 1);
        assert_eq!(library.playlists[0].track_ids.len(), 3);
    }

    #[tokio::test]
    async fn test_demo_provider_search() {
        let provider = DemoProvider::new();
        let matches = provider.search("Horizon").await.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].title, "Neon Horizon");

        let artist_matches = provider.search("Lo-Fi").await.unwrap();
        assert_eq!(artist_matches.len(), 1);
        assert_eq!(artist_matches[0].artist, "Lo-Fi Beats");
    }

    #[tokio::test]
    async fn test_demo_provider_stream_track() {
        let provider = DemoProvider::new();
        let track_id = TrackId::new("demo_track_1");
        let mut stream = provider.stream_track(&track_id).await.unwrap();

        let mut header = [0u8; 12];
        stream.read_exact(&mut header).await.unwrap();
        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(&header[8..12], b"WAVE");
    }
}
