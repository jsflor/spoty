# Tasks: playlist-lifecycle-management

## 1. Storage Operations (`crates/storage/src/db.rs`)

- [x] 1.1 Implement `delete_playlist`, `remove_track_from_playlist`, and `rename_playlist` in `LibraryDatabase`.
- [x] 1.2 Write unit tests in `crates/storage/src/db.rs` covering playlist deletion, track removal, and renaming. Verify with `cargo test -p spoty-storage`.

## 2. Playlist View Enhancements (`crates/ui/src/views/playlist.rs`)

- [x] 2.1 Update `PlaylistView::show` signature to support deletion, renaming, and track removal callbacks.
- [x] 2.2 Replace static `•••` action button with an interactive popup menu offering "Edit details" and "Delete playlist".
- [x] 2.3 Add track removal context action for track rows within custom playlists. Verify compilation with `cargo check -p spoty-ui`.

## 3. Shell Architecture and Context Menus (`crates/ui/src/app.rs`)

- [x] 3.1 Add right-click context menu on sidebar playlist rows with "Rename" and "Delete playlist".
- [x] 3.2 Implement Spotify-authentic confirmation dialog modal for playlist deletion.
- [x] 3.3 Implement playlist renaming dialog/prompt and update `SpotyApp` in-memory state.
- [x] 3.4 Wire up deletion and removal handlers: update SQLite, prune in-memory library, and navigate away if active playlist is deleted.

## 4. Verification and Quality Gates

- [x] 4.1 Run full workspace test suite `cargo test --workspace`.
- [x] 4.2 Verify formatting and clean compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
