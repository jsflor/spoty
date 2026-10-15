# Tasks: track-context-menus

## 1. Core Queue Operations (`crates/core/src/queue.rs`)

- [x] 1.1 Implement `push_back` and `insert_next` in `Queue`.
- [x] 1.2 Write unit tests in `crates/core/src/queue.rs` for queue insertions. Verify with `cargo test -p spoty-core`.

## 2. Track Context Menu Architecture (`crates/ui/src/`)

- [x] 2.1 Implement `render_track_context_menu` in `crates/ui/src/widgets/track_menu.rs`.
- [x] 2.2 Wire up "Add to Playlist", "Add to Queue", "Liked Songs", and "Remove from Playlist" actions.
- [x] 2.3 Integrate context menu into `crates/ui/src/views/songs.rs` (All Songs & Liked Songs).
- [x] 2.4 Integrate context menu into `crates/ui/src/views/playlist.rs` (Playlist View).
- [x] 2.5 Integrate context menu into `crates/ui/src/views/home.rs` (Featured track cards).
- [x] 2.6 Overhaul `crates/ui/src/views/search.rs` track list to match Spotify Encore table styling with interactive context menus.

## 3. Shell Application Wiring (`crates/ui/src/app.rs`)

- [x] 3.1 Handle "Add to Queue" action in `SpotyApp` to update `self.queue`.
- [x] 3.2 Handle "Add to Playlist" action in `SpotyApp` to update SQLite and in-memory playlists.
- [x] 3.3 Verify compilation with `cargo check -p spoty-ui`.

## 4. Verification and Quality Gates

- [x] 4.1 Run full workspace test suite `cargo test --workspace`.
- [x] 4.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
