use spoty_audio::{MockSink, PlayerController, PlayerEvent};
use std::time::Duration;

#[test]
fn test_volume_mute_and_restore_behavior() {
    let (controller, event_rx) = PlayerController::new_with_sink(Box::new(MockSink::new()));

    // Initial state
    assert!(!controller.state().is_muted);

    // Set volume
    controller.set_volume(0.65);
    while let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
        if let PlayerEvent::StateChanged(state) = event {
            controller.update_cached_state(state);
            break;
        }
    }
    assert!((controller.state().volume - 0.65).abs() < 1e-4);

    // Toggle mute
    controller.toggle_mute();
    while let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
        if let PlayerEvent::StateChanged(state) = event {
            controller.update_cached_state(state);
            break;
        }
    }
    assert!(controller.state().is_muted);

    // Toggle unmute restores volume
    controller.toggle_mute();
    while let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
        if let PlayerEvent::StateChanged(state) = event {
            controller.update_cached_state(state);
            break;
        }
    }
    assert!(!controller.state().is_muted);
    assert!((controller.state().volume - 0.65).abs() < 1e-4);
}
