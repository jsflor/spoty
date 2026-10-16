# Tasks: provider-switching-audio-fix

## 1. PlaybackSession Guarding (`crates/audio`)

- [x] 1.1 In `crates/audio/src/session.rs`, guard `SessionCommand::PlayTrack(track_id)` to only call `load_and_play_current` when `track_id` is found in the queue.
- [x] 1.2 Add a unit test in `crates/audio/tests/session_test.rs` asserting that `play_track` with an unknown ID does not initiate playback or change state.

## 2. Shell Provider Switching and Playback Resilience (`crates/ui`)

- [x] 2.1 In `crates/ui/src/app.rs`, update `switch_provider` to pause playback, clear the queue, and asynchronously update the session queue upon `catalog.sync_provider` completion.
- [x] 2.2 In `crates/ui/src/app.rs`, update `play_track` to check if the track is in the active queue, and if not, load catalog tracks starting at that track.
- [x] 2.3 Add integration test in `crates/ui/tests/provider_switch_test.rs` verifying switching between demo and local/memory providers and successfully playing tracks from the new provider.

## 3. Verification & Quality Gates

- [x] 3.1 Run `cargo test --workspace` and verify all tests pass.
- [x] 3.2 Run `cargo fmt --all -- --check` and `cargo check --workspace`.
- [x] 3.3 Sync specs and archive change to `openspec/changes/archive/`.
