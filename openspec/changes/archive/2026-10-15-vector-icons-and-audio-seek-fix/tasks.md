# Tasks: Vector Icons and Audio Seek Engine Fix

- [x] 1. Implement vector icons in `crates/ui/src/widgets/icons.rs` (`draw_home_icon`, `draw_plus_icon`, `draw_close_icon`, `draw_sort_arrow`, vector `shuffle_button`)
- [x] 2. Update `crates/ui/src/app.rs` to use vector icons for Home, Create Playlist, Queue Close, and Queue Remove
- [x] 3. Update `crates/ui/src/views/songs.rs` and `crates/ui/src/widgets/track_menu.rs` to use vector sort arrows and standard text
- [x] 4. Update `crates/ui/src/views/playlist.rs` to use vector close/remove icon
- [x] 5. Implement audio seeking fallback with `skip_duration` and `last_tick` reset in `crates/audio/src/worker.rs`
- [x] 6. Enhance pointer interaction in `crates/ui/src/widgets/slider.rs`
- [x] 7. Add integration test verifying audio seek behavior in `crates/audio/tests/controller_test.rs`
- [x] 8. Verify all workspace tests and formatting checks pass (`cargo test --workspace`, `cargo check --workspace`, `cargo fmt --all -- --check`)
