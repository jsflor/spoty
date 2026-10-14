use spoty_core::{AudioFormat, Track, TrackId};
use spoty_storage::LibraryDatabase;

#[test]
fn test_all_songs_and_favorites_filtering() {
    let db = LibraryDatabase::open_in_memory().unwrap();

    let trk1 = Track {
        id: TrackId::new("t1"),
        title: "Track Alpha".into(),
        artist: "Artist One".into(),
        album: Some("Album One".into()),
        duration_secs: 180,
        cover_uri: None,
        source_uri: "local://t1.mp3".into(),
        format: AudioFormat::Mp3,
    };
    let trk2 = Track {
        id: TrackId::new("t2"),
        title: "Track Beta".into(),
        artist: "Unknown Artist".into(),
        album: None,
        duration_secs: 210,
        cover_uri: None,
        source_uri: "local://t2.mp3".into(),
        format: AudioFormat::Mp3,
    };

    db.upsert_tracks(&[trk1.clone(), trk2.clone()]).unwrap();
    let lib = db.load_library().unwrap();
    assert_eq!(lib.tracks.len(), 2);

    // Favoriting
    assert!(!db.is_favorite(&trk1.id).unwrap());
    db.toggle_favorite(&trk1.id).unwrap();
    assert!(db.is_favorite(&trk1.id).unwrap());

    // Filter to liked songs
    let liked_tracks: Vec<_> = lib
        .tracks
        .iter()
        .filter(|t| db.is_favorite(&t.id).unwrap_or(false))
        .collect();
    assert_eq!(liked_tracks.len(), 1);
    assert_eq!(liked_tracks[0].title, "Track Alpha");

    // All songs contains untagged/unknown artist tracks
    assert!(lib.tracks.iter().any(|t| t.artist == "Unknown Artist"));
}
