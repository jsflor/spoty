# Tasks: playback-session-pipeline

## 1. Core State Unification (`crates/audio/src/session.rs`)

- [x] 1.1 Define `SessionState`, `SessionCommand`, and `SessionEvent` types in `crates/audio/src/session.rs`.
- [x] 1.2 Implement `PlaybackSession::new` with an internal command channel and background coordinator loop.
- [x] 1.3 Embed `spoty_core::Queue` inside the session coordinator to own queue order, history, and shuffle state.
- [x] 1.4 Write unit tests in `crates/audio/tests/session_test.rs` covering queue commands (`PlayTrack`, `PlayAll`, `Next`, `Prev`, `SetQueue`). Verify with `cargo test -p spoty-audio`.

## 2. Streaming Audio Pipeline without Complete File Buffering

- [x] 2.1 Refactor `AudioWorker` to consume `Box<dyn Read + Send>` or streaming chunked buffers instead of static `Vec<u8>`.
- [x] 2.2 Implement bounded stream feeder to maintain `<50MB RAM` consumption during playback of large files.
- [x] 2.3 Wire EOF and track-ended events directly from `AudioWorker` to the session coordinator for automatic track progression.
- [x] 2.4 Verify with `cargo test -p spoty-audio`.

## 3. Shell Integration & UI Simplification (`crates/ui/src/app.rs`)

- [x] 3.1 Replace `SpotyApp`'s separate `Queue`, `PlayerController`, and `Receiver<PlayerEvent>` fields with `PlaybackSession`.
- [x] 3.2 Remove manual `tokio_rt.spawn` byte-downloading routines from `SpotyApp::play_track`.
- [x] 3.3 Remove duplicate shuffle and repeat synchronization logic in `handle_keyboard_shortcuts` and `player_bar`.
- [x] 3.4 Wire queue drawer and context menu actions to `PlaybackSession` queue methods.
- [x] 3.5 Verify compilation with `cargo check -p spoty-ui`.

## 4. Test Suite and Quality Gates

- [x] 4.1 Update integration tests in `crates/ui/tests/demo_integration_test.rs` and `crates/ui/tests/queue_drawer_test.rs` to assert on `PlaybackSession`.
- [x] 4.2 Run full workspace test suite `cargo test --workspace`.
- [x] 4.3 Verify code formatting with `cargo fmt --all -- --check` and lint checks with `cargo clippy --workspace`.
