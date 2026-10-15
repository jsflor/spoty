use spoty_core::{AudioFormat, Queue, Track, TrackId};
use spoty_storage::LibraryDatabase;

#[test]
fn test_track_context_actions_playlist_and_queue() {
    let db = LibraryDatabase::open_in_memory().unwrap();

    let track = Track {
        id: TrackId::new("trk_test_1"),
        title: "Test Track".into(),
        artist: "Test Artist".into(),
        album: Some("Test Album".into()),
        duration_secs: 200,
        cover_uri: None,
        source_uri: "demo://test.wav".into(),
        format: AudioFormat::Wav,
    };

    // 1. Create playlist and assign track
    let playlist = db.create_playlist("Chill Vibes", None).unwrap();
    db.add_track_to_playlist(&playlist.id, &track.id).unwrap();

    let lib = db.load_library().unwrap();
    let pl = lib.playlists.iter().find(|p| p.id == playlist.id).unwrap();
    assert_eq!(pl.track_ids.len(), 1);
    assert_eq!(pl.track_ids[0], track.id);

    // 2. Queue append
    let mut queue = Queue::with_tracks(vec![track.clone()], Some(0));
    let track2 = Track {
        id: TrackId::new("trk_test_2"),
        title: "Upcoming Jam".into(),
        artist: "Test Artist".into(),
        album: None,
        duration_secs: 240,
        cover_uri: None,
        source_uri: "demo://test2.wav".into(),
        format: AudioFormat::Wav,
    };

    queue.push_back(track2.clone());
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.tracks()[1].title, "Upcoming Jam");
}
