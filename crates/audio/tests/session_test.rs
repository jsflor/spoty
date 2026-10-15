use spoty_audio::{
    AudioStreamReader, AudioStreamResolver, BoundedStreamReader, PlaybackSession, SessionEvent,
};
use spoty_core::{AudioFormat, PlaybackStatus, RepeatMode, Track, TrackId};
use std::sync::Arc;
use std::time::Duration;

struct MockResolver {
    wav_bytes: Vec<u8>,
}

impl MockResolver {
    fn new() -> Self {
        Self {
            wav_bytes: create_test_wav(440.0, 1.0),
        }
    }
}

impl AudioStreamResolver for MockResolver {
    fn resolve_stream(&self, _track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
        Ok(BoundedStreamReader::from_bytes(self.wav_bytes.clone()))
    }
}

fn make_test_track(id: &str, title: &str) -> Track {
    Track {
        id: TrackId::new(id),
        title: title.into(),
        artist: "Test Artist".into(),
        album: Some("Test Album".into()),
        duration_secs: 180,
        cover_uri: None,
        source_uri: format!("mock://{}", id),
        format: AudioFormat::Wav,
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

    for i in 0..total_samples as usize {
        let t = i as f32 / sample_rate as f32;
        let sample = ((2.0 * std::f32::consts::PI * freq * t).sin() * 0.2 * i16::MAX as f32) as i16;
        let bytes = sample.to_le_bytes();
        buf.extend_from_slice(&bytes);
        buf.extend_from_slice(&bytes);
    }
    buf
}

#[test]
fn test_session_queue_commands() {
    let resolver = Arc::new(MockResolver::new());
    let (session, event_rx) = PlaybackSession::new(resolver);

    let t1 = make_test_track("t1", "Track 1");
    let t2 = make_test_track("t2", "Track 2");
    let t3 = make_test_track("t3", "Track 3");
    let tracks = vec![t1.clone(), t2.clone(), t3.clone()];

    // 1. SetQueue
    session.set_queue(tracks.clone(), Some(0));

    // Wait for queue state
    let mut state_received = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 1") {
                assert_eq!(state.upcoming_tracks.len(), 2);
                assert_eq!(state.upcoming_tracks[0].title, "Track 2");
                assert_eq!(state.upcoming_tracks[1].title, "Track 3");
                state_received = true;
                break;
            }
        }
    }
    assert!(state_received, "Failed to receive state after set_queue");

    // 2. PlayTrack
    session.play_track(TrackId::new("t2"));
    let mut track2_playing = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 2")
                && state.playback.status == PlaybackStatus::Playing
            {
                assert_eq!(state.upcoming_tracks.len(), 1);
                assert_eq!(state.upcoming_tracks[0].title, "Track 3");
                track2_playing = true;
                break;
            }
        }
    }
    assert!(track2_playing, "Failed to switch to Track 2 via PlayTrack");

    // 3. Next
    session.next();
    let mut track3_playing = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 3")
                && state.playback.status == PlaybackStatus::Playing
            {
                assert_eq!(state.upcoming_tracks.len(), 0);
                track3_playing = true;
                break;
            }
        }
    }
    assert!(track3_playing, "Failed to advance to Track 3 via Next");

    // 4. Prev
    session.prev();
    let mut track2_back = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 2") {
                track2_back = true;
                break;
            }
        }
    }
    assert!(track2_back, "Failed to retreat to Track 2 via Prev");

    // 5. PlayAll
    let t4 = make_test_track("t4", "Track 4");
    let t5 = make_test_track("t5", "Track 5");
    session.play_all(vec![t4.clone(), t5.clone()], 1);
    let mut play_all_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 5")
                && state.playback.status == PlaybackStatus::Playing
            {
                play_all_ok = true;
                break;
            }
        }
    }
    assert!(play_all_ok, "Failed to PlayAll with start_index = 1");

    // 6. EnqueueNext, EnqueueLast, ClearUpcoming
    let t6 = make_test_track("t6", "Track 6");
    let t7 = make_test_track("t7", "Track 7");
    session.enqueue_last(t6);
    session.enqueue_next(t7);

    let mut queue_ops_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.upcoming_tracks.len() == 2 && state.upcoming_tracks[0].title == "Track 7" {
                assert_eq!(state.upcoming_tracks[1].title, "Track 6");
                queue_ops_ok = true;
                break;
            }
        }
    }
    assert!(queue_ops_ok, "Failed to enqueue next/last");

    // Remove from upcoming index 0
    session.remove_from_queue(0);
    let mut remove_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.upcoming_tracks.len() == 1 && state.upcoming_tracks[0].title == "Track 6" {
                remove_ok = true;
                break;
            }
        }
    }
    assert!(remove_ok, "Failed to remove from queue");

    // Clear upcoming
    session.clear_upcoming();
    let mut clear_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.upcoming_tracks.is_empty() {
                clear_ok = true;
                break;
            }
        }
    }
    assert!(clear_ok, "Failed to clear upcoming queue");

    // 7. Shuffle and Repeat
    session.toggle_shuffle();
    let mut shuffle_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.is_shuffle {
                shuffle_ok = true;
                break;
            }
        }
    }
    assert!(shuffle_ok, "Failed to toggle shuffle");

    session.cycle_repeat_mode();
    let mut repeat_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.repeat_mode == RepeatMode::All {
                repeat_ok = true;
                break;
            }
        }
    }
    assert!(repeat_ok, "Failed to cycle repeat mode");
}

#[test]
fn test_session_automatic_track_progression_on_eof() {
    let resolver = Arc::new(MockResolver {
        wav_bytes: create_test_wav(440.0, 0.2), // Very short 0.2s track
    });
    let (session, event_rx) = PlaybackSession::new(resolver);

    let mut t1 = make_test_track("t1", "Track 1");
    t1.duration_secs = 1;
    let mut t2 = make_test_track("t2", "Track 2");
    t2.duration_secs = 1;

    // Start playing Track 1
    session.play_all(vec![t1, t2], 0);

    // Initial state: Track 1 playing
    let mut track1_started = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 1")
                && state.playback.status == PlaybackStatus::Playing
            {
                track1_started = true;
                break;
            }
        }
    }
    assert!(track1_started, "Track 1 should start playing");

    // Wait for autonomous progression to Track 2 (without calling session.next())
    let mut track2_auto_started = false;
    for _ in 0..20 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(300))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Track 2")
                && state.playback.status == PlaybackStatus::Playing
            {
                track2_auto_started = true;
                break;
            }
        }
    }
    assert!(
        track2_auto_started,
        "Session coordinator should autonomously advance to Track 2 on EOF"
    );
}
