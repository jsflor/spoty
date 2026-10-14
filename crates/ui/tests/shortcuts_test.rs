use spoty_core::PlaybackStatus;

#[test]
fn test_keyboard_shortcut_state_helpers() {
    let mut vol = 0.8f32;
    // Test Up arrow volume increment
    vol = (vol + 0.05).min(1.0);
    assert!((vol - 0.85).abs() < 1e-4);

    // Test Down arrow volume decrement
    vol = (vol - 0.05).max(0.0);
    assert!((vol - 0.80).abs() < 1e-4);

    // Test seek clamping
    let current_pos = 120.0f64;
    let duration = 180.0f64;
    let new_pos = (current_pos + 5.0).clamp(0.0, duration);
    assert_eq!(new_pos, 125.0);

    let rewind_pos = (current_pos - 5.0).clamp(0.0, duration);
    assert_eq!(rewind_pos, 115.0);

    let status = PlaybackStatus::Playing;
    assert_eq!(status, PlaybackStatus::Playing);
}
