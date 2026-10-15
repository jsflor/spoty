# Tasks: queue-drawer-panel

## 1. Core Queue Operations (`crates/core/src/queue.rs`)

- [x] 1.1 Implement `remove_at`, `clear_upcoming`, and `upcoming_tracks` in `Queue`.
- [x] 1.2 Write unit tests in `crates/core/src/queue.rs` for queue removal, clearing, and traversal. Verify with `cargo test -p spoty-core`.

## 2. Queue Toggle Widget (`crates/ui/src/widgets/icons.rs`)

- [x] 2.1 Implement `queue_button` painter in `crates/ui/src/widgets/icons.rs` with active green indicator and hover feedback.
- [x] 2.2 Expose `queue_button` in `crates/ui/src/widgets/mod.rs`. Verify compilation with `cargo check -p spoty-ui`.

## 3. Queue Drawer Panel & Wiring (`crates/ui/src/app.rs`)

- [x] 3.1 Add `queue_drawer_open: bool` state to `SpotyApp`.
- [x] 3.2 Add `queue_button` toggle to the right section of the 72px player bar.
- [x] 3.3 Implement `egui::SidePanel::right` queue drawer with "Now Playing" and "Next Up" sections.
- [x] 3.4 Wire up track jump playback, track removal (`✕`), and "Clear Queue" actions.
- [x] 3.5 Verify compilation with `cargo check -p spoty-ui`.

## 4. Verification and Quality Gates

- [x] 4.1 Run full workspace test suite `cargo test --workspace`.
- [x] 4.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
