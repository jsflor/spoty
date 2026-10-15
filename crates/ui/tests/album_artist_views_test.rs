use spoty_core::{AlbumId, ArtistId, AudioFormat, Track, TrackId};
use spoty_storage::Catalog;

#[test]
fn test_album_and_artist_catalog_aggregation_and_lookup() {
    let catalog = Catalog::open_in_memory().unwrap();

    let trk1 = Track {
        id: TrackId::new("t1"),
        title: "One More Time".into(),
        artist: "Daft Punk".into(),
        album: Some("Discovery".into()),
        duration_secs: 320,
        cover_uri: None,
        source_uri: "local://t1.mp3".into(),
        format: AudioFormat::Mp3,
    };
    let trk2 = Track {
        id: TrackId::new("t2"),
        title: "Aerodynamic".into(),
        artist: "Daft Punk".into(),
        album: Some("Discovery".into()),
        duration_secs: 212,
        cover_uri: None,
        source_uri: "local://t2.mp3".into(),
        format: AudioFormat::Mp3,
    };
    let trk3 = Track {
        id: TrackId::new("t3"),
        title: "Harder, Better, Faster, Stronger".into(),
        artist: "Daft Punk".into(),
        album: Some("Discovery".into()),
        duration_secs: 224,
        cover_uri: None,
        source_uri: "local://t3.mp3".into(),
        format: AudioFormat::Mp3,
    };
    let trk4 = Track {
        id: TrackId::new("t4"),
        title: "Around the World".into(),
        artist: "Daft Punk".into(),
        album: Some("Homework".into()),
        duration_secs: 429,
        cover_uri: None,
        source_uri: "local://t4.mp3".into(),
        format: AudioFormat::Mp3,
    };

    catalog
        .upsert_tracks(&[trk1.clone(), trk2.clone(), trk3.clone(), trk4.clone()])
        .unwrap();

    let lib = catalog.library();

    // Verify artists aggregation
    assert_eq!(lib.artists.len(), 1);
    let artist = &lib.artists[0];
    assert_eq!(artist.name, "Daft Punk");
    assert_eq!(artist.id, ArtistId::new("art_daft_punk"));

    // Verify albums aggregation
    assert_eq!(lib.albums.len(), 2);
    let discovery = lib.albums.iter().find(|a| a.title == "Discovery").unwrap();
    assert_eq!(discovery.artist, "Daft Punk");
    assert_eq!(discovery.track_ids.len(), 3);
    assert_eq!(discovery.id, AlbumId::new("alb_discovery"));

    let homework = lib.albums.iter().find(|a| a.title == "Homework").unwrap();
    assert_eq!(homework.artist, "Daft Punk");
    assert_eq!(homework.track_ids.len(), 1);
    assert_eq!(homework.id, AlbumId::new("alb_homework"));

    // Verify artist tracks resolution
    let daft_tracks: Vec<_> = lib
        .tracks
        .iter()
        .filter(|t| t.artist.eq_ignore_ascii_case(&artist.name))
        .collect();
    assert_eq!(daft_tracks.len(), 4);

    // Verify album tracks resolution
    let discovery_tracks: Vec<_> = discovery
        .track_ids
        .iter()
        .filter_map(|id| lib.tracks.iter().find(|t| &t.id == id))
        .collect();
    assert_eq!(discovery_tracks.len(), 3);
    assert_eq!(discovery_tracks[0].title, "One More Time");
}
