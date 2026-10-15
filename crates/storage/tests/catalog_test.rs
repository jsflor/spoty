use spoty_core::{AudioFormat, Track, TrackId};
use spoty_storage::{Catalog, DemoProvider};

#[tokio::test]
async fn test_catalog_open_in_memory_and_favorites() {
    let catalog = Catalog::open_in_memory().expect("failed to open in memory catalog");

    let track_id = TrackId::new("trk_fav_1");
    assert!(!catalog.is_favorite(&track_id));
    assert!(catalog.get_favorites().is_empty());

    // Toggle on
    let favorited = catalog.toggle_favorite(&track_id).expect("toggle failed");
    assert!(favorited);
    assert!(catalog.is_favorite(&track_id));
    assert_eq!(catalog.get_favorites(), vec![track_id.clone()]);

    // Verify persisted in underlying DB
    assert!(catalog.db().is_favorite(&track_id).unwrap());

    // Toggle off
    let unfavorited = catalog.toggle_favorite(&track_id).expect("toggle failed");
    assert!(!unfavorited);
    assert!(!catalog.is_favorite(&track_id));
    assert!(catalog.get_favorites().is_empty());
    assert!(!catalog.db().is_favorite(&track_id).unwrap());
}

#[test]
fn test_catalog_atomic_playlist_crud() {
    let catalog = Catalog::open_in_memory().expect("failed to open in memory catalog");

    // 1. Create playlist
    let pl = catalog
        .create_playlist("Chill Vibes", Some("Evening chill"))
        .expect("create playlist failed");
    assert_eq!(catalog.library().playlists.len(), 1);
    assert_eq!(catalog.get_playlist(&pl.id).unwrap().title, "Chill Vibes");

    // 2. Add track to playlist
    let track_id = TrackId::new("trk_audio_1");
    catalog
        .add_track_to_playlist(&pl.id, &track_id)
        .expect("add track failed");

    let updated_pl = catalog.get_playlist(&pl.id).unwrap();
    assert_eq!(updated_pl.track_ids, vec![track_id.clone()]);

    // 3. Rename playlist
    catalog
        .rename_playlist(&pl.id, "Late Night Chill", Some("Midnight vibes"))
        .expect("rename playlist failed");
    let renamed_pl = catalog.get_playlist(&pl.id).unwrap();
    assert_eq!(renamed_pl.title, "Late Night Chill");
    assert_eq!(renamed_pl.description.as_deref(), Some("Midnight vibes"));

    // 4. Remove track from playlist
    catalog
        .remove_track_from_playlist(&pl.id, &track_id)
        .expect("remove track failed");
    let emptied_pl = catalog.get_playlist(&pl.id).unwrap();
    assert!(emptied_pl.track_ids.is_empty());

    // 5. Delete playlist
    catalog.delete_playlist(&pl.id).expect("delete failed");
    assert!(catalog.get_playlist(&pl.id).is_none());
    assert!(catalog.library().playlists.is_empty());

    // Verify DB empty
    let db_lib = catalog.db().load_library().unwrap();
    assert!(db_lib.playlists.is_empty());
}

#[tokio::test]
async fn test_catalog_sync_provider() {
    let catalog = Catalog::open_in_memory().expect("failed to open in memory catalog");
    let demo_provider = DemoProvider::new();

    // 1. Sync demo provider
    catalog
        .sync_provider(&demo_provider)
        .await
        .expect("sync demo provider failed");

    let lib = catalog.library();
    assert!(!lib.tracks.is_empty());
    assert!(!lib.albums.is_empty());
    assert!(!lib.artists.is_empty());

    // 2. Create custom playlist
    let pl = catalog
        .create_playlist("Demo Favorites", None)
        .expect("create playlist failed");
    catalog
        .add_track_to_playlist(&pl.id, &lib.tracks[0].id)
        .expect("add track failed");

    // 3. Sync again and ensure custom playlist is preserved
    catalog
        .sync_provider(&demo_provider)
        .await
        .expect("sync demo provider failed");

    let resynced_lib = catalog.library();
    assert!(resynced_lib.playlists.iter().any(|p| p.id == pl.id));
    let cached_pl = catalog.get_playlist(&pl.id).unwrap();
    assert_eq!(cached_pl.track_ids, vec![lib.tracks[0].id.clone()]);
}

#[test]
fn test_catalog_upsert_tracks() {
    let catalog = Catalog::open_in_memory().expect("failed to open in memory catalog");

    let trk = Track {
        id: TrackId::new("trk_upsert_1"),
        title: "Test Track".into(),
        artist: "Test Artist".into(),
        album: Some("Test Album".into()),
        duration_secs: 150,
        cover_uri: None,
        source_uri: "local://test.mp3".into(),
        format: AudioFormat::Mp3,
    };

    catalog.upsert_tracks(&[trk.clone()]).unwrap();

    let lib = catalog.library();
    assert_eq!(lib.tracks.len(), 1);
    assert_eq!(lib.tracks[0].title, "Test Track");
    assert_eq!(lib.artists.len(), 1);
    assert_eq!(lib.albums.len(), 1);
}
