use spoty_audio::{MockSink, PlayerController, PlayerEvent};
use spoty_core::{AudioFormat, PlaybackStatus, Track, TrackId};
use std::time::Duration;

#[test]
fn test_player_controller_lifecycle() {
    let (controller, event_rx) = PlayerController::new_with_sink(Box::new(MockSink::new()));

    let track = Track {
        id: TrackId::new("test_trk"),
        title: "Test Track".into(),
        artist: "Test Artist".into(),
        album: None,
        duration_secs: 10,
        cover_uri: None,
        source_uri: "memory://test".into(),
        format: AudioFormat::Wav,
    };

    // Synthesize 0.5s of valid PCM WAV
    let wav_bytes = create_test_wav(440.0, 0.5);

    controller.load_track(track, wav_bytes);

    // Wait for state event from worker
    let event = event_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("Timed out waiting for worker event");
    if let PlayerEvent::StateChanged(state) = event {
        assert_eq!(state.status, PlaybackStatus::Playing);
        assert_eq!(state.current_track.unwrap().title, "Test Track");
    } else {
        panic!("Expected StateChanged event");
    }

    controller.pause();
    let event = event_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("Timed out waiting for pause event");
    if let PlayerEvent::StateChanged(state) = event {
        assert_eq!(state.status, PlaybackStatus::Paused);
    }
}

#[test]
fn test_player_controller_seek_updates_position_and_state() {
    let (controller, event_rx) = PlayerController::new_with_sink(Box::new(MockSink::new()));

    let track = Track {
        id: TrackId::new("test_trk_seek"),
        title: "Test Track For Seek".into(),
        artist: "Test Artist".into(),
        album: None,
        duration_secs: 30,
        cover_uri: None,
        source_uri: "memory://test_seek".into(),
        format: AudioFormat::Wav,
    };

    let wav_bytes = create_test_wav(440.0, 5.0);
    controller.load_track(track, wav_bytes);

    // Initial load event
    let event = event_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("Expected initial load event");
    if let PlayerEvent::StateChanged(st) = event {
        assert_eq!(st.status, PlaybackStatus::Playing);
    }

    // Seek forward to 12.0 seconds
    controller.seek(Duration::from_secs_f64(12.0));

    let event = event_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("Expected forward seek event");
    if let PlayerEvent::StateChanged(st) = event {
        assert_eq!(st.status, PlaybackStatus::Playing);
        assert!((st.position_secs - 12.0).abs() < 1e-3);
    } else {
        panic!("Expected StateChanged event for seek");
    }

    // Seek backward to 3.0 seconds
    controller.seek(Duration::from_secs_f64(3.0));

    let event = event_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("Expected backward seek event");
    if let PlayerEvent::StateChanged(st) = event {
        assert_eq!(st.status, PlaybackStatus::Playing);
        assert!((st.position_secs - 3.0).abs() < 1e-3);
    } else {
        panic!("Expected StateChanged event for seek");
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
        let sample = ((2.0 * std::f32::consts::PI * freq * t).sin() * 0.3 * i16::MAX as f32) as i16;
        let bytes = sample.to_le_bytes();
        buf.extend_from_slice(&bytes);
        buf.extend_from_slice(&bytes);
    }
    buf
}
