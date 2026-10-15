use spoty_core::{AudioFormat, Queue, Track, TrackId};

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

    let mut queue = Queue::with_tracks(tracks, Some(0));

    // Initial state: current is t1, upcoming has t2 and t3
    assert_eq!(queue.current_track().unwrap().title, "Current Track");
    let upcoming = queue.upcoming_tracks();
    assert_eq!(upcoming.len(), 2);
    assert_eq!(upcoming[0].1.title, "Next Track");
    assert_eq!(upcoming[1].1.title, "Third Track");

    // Remove t2 (index 1)
    queue.remove_at(1);
    let upcoming_after_remove = queue.upcoming_tracks();
    assert_eq!(upcoming_after_remove.len(), 1);
    assert_eq!(upcoming_after_remove[0].1.title, "Third Track");

    // Clear upcoming
    queue.clear_upcoming();
    assert_eq!(queue.upcoming_tracks().len(), 0);
    assert_eq!(queue.current_track().unwrap().title, "Current Track");
}
