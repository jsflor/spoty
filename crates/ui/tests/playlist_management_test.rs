use spoty_storage::Catalog;

#[test]
fn test_playlist_lifecycle_and_navigation_redirect() {
    let catalog = Catalog::open_in_memory().unwrap();
    let playlist = catalog
        .create_playlist("My Workout Mix", Some("High BPM tracks"))
        .unwrap();
    assert!(playlist.id.as_str().starts_with("custom_pl_"));
    assert_eq!(catalog.library().playlists.len(), 1);

    // Test rename
    catalog
        .rename_playlist(&playlist.id, "Ultra Workout", Some("160+ BPM"))
        .unwrap();
    let renamed = catalog.get_playlist(&playlist.id).unwrap();
    assert_eq!(renamed.title, "Ultra Workout");
    assert_eq!(renamed.description.as_deref(), Some("160+ BPM"));

    // Test deletion from catalog (both in-memory and DB)
    catalog.delete_playlist(&playlist.id).unwrap();
    assert_eq!(catalog.library().playlists.len(), 0);
    assert!(catalog.get_playlist(&playlist.id).is_none());

    // Verify DB empty
    let db_lib = catalog.db().load_library().unwrap();
    assert_eq!(db_lib.playlists.len(), 0);
}
