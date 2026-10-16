use spoty_audio::{
    AudioStreamReader, AudioStreamResolver, BoundedStreamReader, PlaybackSession, SessionEvent,
};
use spoty_core::{AudioFormat, PlaybackStatus, Track, TrackId};
use spoty_storage::async_trait;
use spoty_storage::{AudioStream, Catalog, DemoProvider, Library, MusicProvider};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

struct MockLocalProvider {
    tracks: Vec<Track>,
    audio_data: Vec<u8>,
}

impl MockLocalProvider {
    fn new() -> Self {
        let wav_data = create_test_wav(520.0, 1.5);
        let track = Track {
            id: TrackId::new("local_t1"),
            title: "Local Track One".into(),
            artist: "Local Artist".into(),
            album: Some("Local Album".into()),
            duration_secs: 120,
            cover_uri: None,
            source_uri: "local://song1.wav".into(),
            format: AudioFormat::Wav,
        };
        Self {
            tracks: vec![track],
            audio_data: wav_data,
        }
    }
}

#[async_trait]
impl MusicProvider for MockLocalProvider {
    fn id(&self) -> &str {
        "mock_local"
    }

    fn display_name(&self) -> &str {
        "Mock Local Storage"
    }

    async fn fetch_library(&self) -> anyhow::Result<Library> {
        let mut lib = Library {
            tracks: self.tracks.clone(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        };
        lib.aggregate_artists_and_albums();
        Ok(lib)
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>> {
        let q = query.to_lowercase();
        Ok(self
            .tracks
            .iter()
            .filter(|t| t.title.to_lowercase().contains(&q))
            .cloned()
            .collect())
    }

    async fn stream_track(&self, track_id: &TrackId) -> anyhow::Result<AudioStream> {
        if self.tracks.iter().any(|t| &t.id == track_id) {
            Ok(Box::pin(Cursor::new(self.audio_data.clone())))
        } else {
            anyhow::bail!("Track not found in MockLocalProvider: {}", track_id)
        }
    }
}

struct TestResolver {
    provider: Arc<dyn MusicProvider>,
    rt: Arc<tokio::runtime::Runtime>,
}

impl AudioStreamResolver for TestResolver {
    fn resolve_stream(&self, track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
        let stream = self.rt.block_on(self.provider.stream_track(&track.id))?;
        Ok(BoundedStreamReader::from_async_read(stream, &self.rt))
    }
}

fn create_test_wav(freq: f32, duration_secs: f32) -> Vec<u8> {
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
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&chunk_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&subchunk2_size.to_le_bytes());

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let sample_i16 = (sample * 16000.0) as i16;
        for _ in 0..num_channels {
            buf.extend_from_slice(&sample_i16.to_le_bytes());
        }
    }

    buf
}

#[test]
fn test_provider_switching_and_playback() {
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let catalog = Arc::new(Catalog::open_in_memory().unwrap());

    // 1. Start with DemoProvider
    let demo_provider: Arc<dyn MusicProvider> = Arc::new(DemoProvider::new());
    rt.block_on(catalog.sync_provider(demo_provider.as_ref()))
        .unwrap();

    let demo_resolver = Arc::new(TestResolver {
        provider: Arc::clone(&demo_provider),
        rt: Arc::clone(&rt),
    });
    let (session, event_rx) = PlaybackSession::new(demo_resolver);
    let initial_tracks = catalog.library().tracks;
    session.set_queue(initial_tracks.clone(), None);

    // Play a demo track
    session.play_track(initial_tracks[0].id.clone());
    let mut demo_playing = false;
    for _ in 0..15 {
        if let Ok(SessionEvent::StateChanged(st)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if st.playback.status == PlaybackStatus::Playing
                && st.current_track.as_ref().map(|t| &t.id) == Some(&initial_tracks[0].id)
            {
                demo_playing = true;
                break;
            }
        }
    }
    assert!(demo_playing, "Demo track should start playing");

    // 2. Switch provider to MockLocalProvider
    let local_provider: Arc<dyn MusicProvider> = Arc::new(MockLocalProvider::new());
    session.pause();
    session.set_queue(Vec::new(), None);

    let local_resolver = Arc::new(TestResolver {
        provider: Arc::clone(&local_provider),
        rt: Arc::clone(&rt),
    });
    session.set_resolver(local_resolver);

    // Sync provider in background task and set queue upon completion
    let cat = Arc::clone(&catalog);
    let prov = Arc::clone(&local_provider);
    let sess = session.clone();
    rt.block_on(async move {
        cat.sync_provider(prov.as_ref()).await.unwrap();
        sess.set_queue(cat.library().tracks, None);
    });

    let new_library = catalog.library();
    assert_eq!(new_library.tracks.len(), 1);
    let local_track = &new_library.tracks[0];
    assert_eq!(local_track.title, "Local Track One");

    // 3. User plays the song from the new source
    // Simulates SpotyApp::play_track logic
    let state = session.state();
    let in_queue = state
        .current_track
        .as_ref()
        .map(|t| t.id == local_track.id)
        .unwrap_or(false)
        || state.upcoming_tracks.iter().any(|t| t.id == local_track.id);

    if in_queue {
        session.play_track(local_track.id.clone());
    } else {
        let tracks = catalog.library().tracks;
        if let Some(pos) = tracks.iter().position(|t| t.id == local_track.id) {
            session.play_all(tracks, pos);
        } else {
            session.play_all(vec![local_track.clone()], 0);
        }
    }

    // Verify local track starts playing without breaking or failing
    let mut local_playing = false;
    for _ in 0..15 {
        if let Ok(SessionEvent::StateChanged(st)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if st.playback.status == PlaybackStatus::Playing
                && st.current_track.as_ref().map(|t| &t.id) == Some(&local_track.id)
            {
                local_playing = true;
                break;
            }
        }
    }
    assert!(
        local_playing,
        "Song from new provider source should play successfully"
    );

    // 4. Switch back to DemoProvider
    session.pause();
    session.set_queue(Vec::new(), None);

    let demo_resolver_2 = Arc::new(TestResolver {
        provider: Arc::clone(&demo_provider),
        rt: Arc::clone(&rt),
    });
    session.set_resolver(demo_resolver_2);

    let cat2 = Arc::clone(&catalog);
    let prov2 = Arc::clone(&demo_provider);
    let sess2 = session.clone();
    rt.block_on(async move {
        cat2.sync_provider(prov2.as_ref()).await.unwrap();
        sess2.set_queue(cat2.library().tracks, None);
    });

    let switched_back_lib = catalog.library();
    assert_eq!(switched_back_lib.tracks.len(), 3);
    let demo_track_again = &switched_back_lib.tracks[1];

    let state = session.state();
    let in_queue = state
        .current_track
        .as_ref()
        .map(|t| t.id == demo_track_again.id)
        .unwrap_or(false)
        || state
            .upcoming_tracks
            .iter()
            .any(|t| t.id == demo_track_again.id);

    if in_queue {
        session.play_track(demo_track_again.id.clone());
    } else {
        let tracks = catalog.library().tracks;
        if let Some(pos) = tracks.iter().position(|t| t.id == demo_track_again.id) {
            session.play_all(tracks, pos);
        } else {
            session.play_all(vec![demo_track_again.clone()], 0);
        }
    }

    let mut demo_again_playing = false;
    for _ in 0..15 {
        if let Ok(SessionEvent::StateChanged(st)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if st.playback.status == PlaybackStatus::Playing
                && st.current_track.as_ref().map(|t| &t.id) == Some(&demo_track_again.id)
            {
                demo_again_playing = true;
                break;
            }
        }
    }
    assert!(
        demo_again_playing,
        "Switching back to demo provider should play demo song successfully"
    );
}
