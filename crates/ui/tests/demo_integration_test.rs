use spoty_audio::{PlayerController, PlayerEvent};
use spoty_core::{AudioFormat, PlaybackStatus};
use spoty_storage::{DemoProvider, MusicProvider};
use std::time::Duration;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn test_demo_tracks_end_to_end_playback() {
    let provider = DemoProvider::new();
    let library = provider.fetch_library().await.unwrap();

    // Verify 3 CC-BY tracks with Kevin MacLeod attribution
    assert_eq!(library.tracks.len(), 3);
    for track in &library.tracks {
        assert_eq!(track.artist, "Kevin MacLeod");
        assert_eq!(track.format, AudioFormat::Ogg);
    }

    // Stream first track
    let track = &library.tracks[0];
    let mut stream = provider.stream_track(&track.id).await.unwrap();
    let mut audio_bytes = Vec::new();
    stream.read_to_end(&mut audio_bytes).await.unwrap();

    assert!(!audio_bytes.is_empty());
    assert_eq!(&audio_bytes[0..4], b"OggS");

    // Initialize player controller and load track
    let (controller, event_rx) = PlayerController::new();
    controller.load_track(track.clone(), audio_bytes);

    let event = event_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("Timed out waiting for audio worker event");

    if let PlayerEvent::StateChanged(state) = event {
        assert_eq!(state.status, PlaybackStatus::Playing);
        assert_eq!(
            state.current_track.as_ref().map(|t| t.title.as_str()),
            Some("Consort for Brass")
        );
    } else {
        panic!("Expected StateChanged event, got {:?}", event);
    }

    controller.stop();
}
