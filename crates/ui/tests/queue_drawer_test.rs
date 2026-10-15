use spoty_audio::{
    AudioStreamReader, AudioStreamResolver, BoundedStreamReader, PlaybackSession, SessionEvent,
};
use spoty_core::{AudioFormat, Track, TrackId};
use std::sync::Arc;
use std::time::Duration;

struct DummyResolver;

impl AudioStreamResolver for DummyResolver {
    fn resolve_stream(&self, _track: &Track) -> anyhow::Result<Box<dyn AudioStreamReader>> {
        Ok(BoundedStreamReader::from_bytes(vec![]))
    }
}

#[test]
fn test_queue_drawer_interactions() {
    let tracks = vec![
        Track {
            id: TrackId::new("t1"),
            title: "Current Track".into(),
            artist: "Artist A".into(),
            album: None,
            duration_secs: 180,
            cover_uri: None,
            source_uri: "demo://t1.wav".into(),
            format: AudioFormat::Wav,
        },
        Track {
            id: TrackId::new("t2"),
            title: "Next Track".into(),
            artist: "Artist B".into(),
            album: None,
            duration_secs: 200,
            cover_uri: None,
            source_uri: "demo://t2.wav".into(),
            format: AudioFormat::Wav,
        },
        Track {
            id: TrackId::new("t3"),
            title: "Third Track".into(),
            artist: "Artist C".into(),
            album: None,
            duration_secs: 220,
            cover_uri: None,
            source_uri: "demo://t3.wav".into(),
            format: AudioFormat::Wav,
        },
    ];

    let (session, event_rx) = PlaybackSession::new(Arc::new(DummyResolver));
    session.set_queue(tracks, Some(0));

    // Wait for initial state: current is t1, upcoming has t2 and t3
    let mut initial_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.current_track.as_ref().map(|t| t.title.as_str()) == Some("Current Track")
                && state.upcoming_tracks.len() == 2
            {
                assert_eq!(state.upcoming_tracks[0].title, "Next Track");
                assert_eq!(state.upcoming_tracks[1].title, "Third Track");
                initial_ok = true;
                break;
            }
        }
    }
    assert!(initial_ok, "Initial queue state not reflected in session");

    // Remove t2 (index 0 of upcoming)
    session.remove_from_queue(0);
    let mut after_remove_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.upcoming_tracks.len() == 1 {
                assert_eq!(state.upcoming_tracks[0].title, "Third Track");
                after_remove_ok = true;
                break;
            }
        }
    }
    assert!(after_remove_ok, "Upcoming track not removed via session");

    // Clear upcoming
    session.clear_upcoming();
    let mut clear_ok = false;
    for _ in 0..10 {
        if let Ok(SessionEvent::StateChanged(state)) =
            event_rx.recv_timeout(Duration::from_millis(200))
        {
            if state.upcoming_tracks.is_empty() {
                assert_eq!(
                    state.current_track.as_ref().map(|t| t.title.as_str()),
                    Some("Current Track")
                );
                clear_ok = true;
                break;
            }
        }
    }
    assert!(clear_ok, "Upcoming tracks not cleared via session");
}
