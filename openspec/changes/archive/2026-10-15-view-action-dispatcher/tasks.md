# Tasks: view-action-dispatcher

## 1. Context and Action Definitions (`crates/ui/src/views/context.rs`)

- [x] 1.1 Create `crates/ui/src/views/context.rs` defining `ViewContext` and `ViewAction` enum.
- [x] 1.2 Export `ViewContext` and `ViewAction` in `crates/ui/src/views/mod.rs`.
- [x] 1.3 Verify compilation with `cargo check -p spoty-ui`.

## 2. Track Context Menu Refactoring (`crates/ui/src/widgets/track_menu.rs`)

- [x] 2.1 Refactor `render_track_context_menu` to return `Option<ViewAction>` instead of accepting `TrackMenuActions` closure struct.
- [x] 2.2 Remove `TrackMenuActions` struct and its closure fields.
- [x] 2.3 Verify with `cargo test -p spoty-ui`.

## 3. View Interface Refactoring (`crates/ui/src/views/`)

- [x] 3.1 Refactor `HomeView::show` signature to `(ui, &ViewContext) -> Option<ViewAction>`.
- [x] 3.2 Refactor `PlaylistView::show` signature to `(ui, &ViewContext, &PlaylistId) -> Option<ViewAction>`.
- [x] 3.3 Refactor `SongsView::show` signature to `(ui, &ViewContext, SongsMode) -> Option<ViewAction>`.
- [x] 3.4 Refactor `SearchView::show` signature to `(ui, &ViewContext, &mut String) -> Option<ViewAction>`.
- [x] 3.5 Verify compilation with `cargo check -p spoty-ui`.

## 4. Shell Router Consolidation (`crates/ui/src/app.rs`)

- [x] 4.1 Implement `SpotyApp::handle_view_action(&mut self, action: ViewAction)`.
- [x] 4.2 Replace duplicate accumulator setups and unpacking blocks in `CentralPanel` router with a single `handle_view_action` dispatch.
- [x] 4.3 Delete obsolete callback closures from `app.rs`.
- [x] 4.4 Verify compilation with `cargo check -p spoty-ui`.

## 5. Verification and Quality Gates

- [x] 5.1 Run UI integration tests (`crates/ui/tests/track_context_menu_test.rs`, `crates/ui/tests/all_songs_test.rs`).
- [x] 5.2 Run workspace test suite `cargo test --workspace`.
- [x] 5.3 Verify formatting with `cargo fmt --all -- --check` and lint checks with `cargo clippy --workspace`.
