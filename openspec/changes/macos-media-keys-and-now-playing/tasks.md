# Tasks: macos-media-keys-and-now-playing

## 1. macOS System Media Keys (`crates/ui/src/os/`)

- [ ] 1.1 Create `crates/ui/src/os/mod.rs` and `crates/ui/src/os/media_keys.rs` with `#[cfg(target_os = "macos")]` bindings.
- [ ] 1.2 Route hardware Play/Pause, Next, and Previous keypresses to `PlayerController`.

## 2. macOS Now Playing Info Integration

- [ ] 2.1 Implement `crates/ui/src/os/now_playing.rs` updating system media info on playback state changes.
- [ ] 2.2 Wire up state change listener in `SpotyApp::update`.

## 3. Verification and Quality Gates

- [ ] 3.1 Verify clean cross-platform compilation with `cargo check -p spoty-ui`.
- [ ] 3.2 Run workspace tests with `cargo test --workspace`.
- [ ] 3.3 Verify code formatting with `cargo fmt --all -- --check`.
