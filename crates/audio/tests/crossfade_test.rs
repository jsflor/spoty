use spoty_audio::{
    AudioStreamReader, AudioStreamResolver, BoundedStreamReader, MockSink, PlaybackSession,
    PlayerController, PlayerEvent, SessionEvent,
};
use spoty_core::{AudioFormat, PlaybackStatus, Track, TrackId};
use std::sync::Arc;
use std::time::Duration;

fn make_track(id: &str, title: &str, duration_secs: u32) -> Track {
    Track {
        id: TrackId::new(id),
        title: title.into(),
        artist: "Test Artist".into(),
        album: None,
        duration_secs,
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

struct TestResolver {
    wav_bytes: Vec<u8>,
}

impl AudioStreamResolver for TestResolver {
    fn resolve_stream(&self, _track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
        Ok(BoundedStreamReader::from_bytes(self.wav_bytes.clone()))
    }
}

#[test]
fn test_player_controller_crossfade_flow() {
    let mock_sink = Box::new(MockSink::new());
    let (controller, event_rx) = PlayerController::new_with_sink(mock_sink);

    controller.set_crossfade(2);

    let t1 = make_track("t1", "Track 1", 5);
    let t2 = make_track("t2", "Track 2", 8);

    let wav1 = create_test_wav(440.0, 1.0);
    let wav2 = create_test_wav(880.0, 1.0);

    controller.load_track(t1, wav1);
    controller.queue_next_track(t2, wav2);

    // Initial load state event
    let event = event_rx
        .recv_timeout(Duration::from_millis(100))
        .expect("Expected initial StateChanged event");
    if let PlayerEvent::StateChanged(state) = event {
        assert_eq!(state.status, PlaybackStatus::Playing);
        assert_eq!(state.current_track.unwrap().title, "Track 1");
    } else {
        panic!("Expected StateChanged event");
    }

    // Seek close to the end (e.g. 4.6 seconds in out of 5 -> 0.4 seconds remaining <= 2s crossfade)
    controller.seek(Duration::from_millis(4600));

    // Drain events to verify crossfade completed or state updated
    let mut crossfade_occurred = false;
    let deadline = std::time::Instant::now() + Duration::from_millis(1500);
    while std::time::Instant::now() < deadline {
        if let Ok(ev) = event_rx.recv_timeout(Duration::from_millis(50)) {
            match ev {
                PlayerEvent::CrossfadeTransitionCompleted { outgoing, current } => {
                    assert_eq!(outgoing.title, "Track 1");
                    assert_eq!(current.title, "Track 2");
                    crossfade_occurred = true;
                    break;
                }
                PlayerEvent::StateChanged(s) => {
                    if let Some(trk) = &s.current_track {
                        if trk.title == "Track 2" {
                            crossfade_occurred = true;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    assert!(crossfade_occurred, "Expected crossfade transition to occur");
}

#[test]
fn test_session_crossfade_integration() {
    let wav = create_test_wav(440.0, 1.0);
    let resolver = Arc::new(TestResolver { wav_bytes: wav });
    let (session, event_rx) = PlaybackSession::new(resolver);

    session.set_crossfade(2);

    let t1 = make_track("t1", "Alpha", 4);
    let t2 = make_track("t2", "Beta", 6);
    session.set_queue(vec![t1, t2], Some(0));

    // Wait for playback to begin on Alpha
    let mut started = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(st)) =
            event_rx.recv_timeout(Duration::from_millis(100))
        {
            if st.current_track.as_ref().map(|t| t.title.as_str()) == Some("Alpha") {
                started = true;
                break;
            }
        }
    }
    assert!(started, "Session failed to start playing track Alpha");

    // Seek towards crossfade window
    session.seek(Duration::from_secs(3));

    // Wait for transition to Beta
    let mut transitioned_to_beta = false;
    let deadline = std::time::Instant::now() + Duration::from_millis(800);
    while std::time::Instant::now() < deadline {
        if let Ok(SessionEvent::StateChanged(st)) = event_rx.recv_timeout(Duration::from_millis(50))
        {
            if st.current_track.as_ref().map(|t| t.title.as_str()) == Some("Beta") {
                transitioned_to_beta = true;
                break;
            }
        }
    }

    assert!(
        transitioned_to_beta,
        "Session failed to transition to Beta under crossfade"
    );
}
