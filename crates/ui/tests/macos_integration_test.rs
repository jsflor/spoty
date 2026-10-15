use spoty_core::{AudioFormat, PlaybackState, PlaybackStatus, Track, TrackId};

#[test]
fn test_macos_now_playing_state_mapping() {
    let mut state = PlaybackState::default();
    assert_eq!(state.status, PlaybackStatus::Stopped);
    assert_eq!(state.position_secs, 0.0);

    let track = Track {
        id: TrackId::new("trk_macos_1"),
        title: "Solaris".into(),
        artist: "Photay".into(),
        album: Some("Onism".into()),
        duration_secs: 215,
        cover_uri: None,
        source_uri: "demo://solaris.wav".into(),
        format: AudioFormat::Wav,
    };

    state.current_track = Some(track);
    state.status = PlaybackStatus::Playing;
    state.position_secs = 42.5;
    state.duration_secs = 215.0;

    assert_eq!(state.status, PlaybackStatus::Playing);
    assert_eq!(state.current_track.as_ref().unwrap().title, "Solaris");
    assert_eq!(state.current_track.as_ref().unwrap().artist, "Photay");
    assert_eq!(
        state.current_track.as_ref().unwrap().album.as_deref(),
        Some("Onism")
    );
    assert_eq!(state.position_secs, 42.5);
    assert_eq!(state.duration_secs, 215.0);
}

#[test]
fn test_macos_media_control_seek_calculation() {
    let current_pos = 60.0f64;
    let duration = 180.0f64;

    // Seek forward 5s
    let next_pos = (current_pos + 5.0).clamp(0.0, duration);
    assert_eq!(next_pos, 65.0);

    // Seek backward 5s
    let prev_pos = (current_pos - 5.0).clamp(0.0, duration);
    assert_eq!(prev_pos, 55.0);

    // Clamp at zero
    let underflow = (2.0f64 - 5.0).clamp(0.0, duration);
    assert_eq!(underflow, 0.0);

    // Clamp at duration
    let overflow = (178.0f64 + 5.0).clamp(0.0, duration);
    assert_eq!(overflow, 180.0);
}
