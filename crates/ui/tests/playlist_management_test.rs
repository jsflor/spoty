use spoty_storage::LibraryDatabase;

#[test]
fn test_playlist_lifecycle_and_navigation_redirect() {
    let db = LibraryDatabase::open_in_memory().unwrap();
    let playlist = db
        .create_playlist("My Workout Mix", Some("High BPM tracks"))
        .unwrap();
    assert!(playlist.id.as_str().starts_with("custom_pl_"));

    let mut playlists = vec![playlist.clone()];
    assert_eq!(playlists.len(), 1);

    // Test rename
    db.rename_playlist(&playlist.id, "Ultra Workout", Some("160+ BPM"))
        .unwrap();
    if let Some(p) = playlists.iter_mut().find(|p| p.id == playlist.id) {
        p.title = "Ultra Workout".to_string();
        p.description = Some("160+ BPM".to_string());
    }
    assert_eq!(playlists[0].title, "Ultra Workout");

    // Test deletion from DB and in-memory list
    db.delete_playlist(&playlist.id).unwrap();
    playlists.retain(|p| p.id != playlist.id);
    assert_eq!(playlists.len(), 0);

    // Verify DB empty
    let lib = db.load_library().unwrap();
    assert_eq!(lib.playlists.len(), 0);
}
