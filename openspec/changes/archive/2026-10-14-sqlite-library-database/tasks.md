## 1. Embedded SQLite Database Layer (`spoty-storage`)

- [x] 1.1 Add `rusqlite` dependency with `bundled` feature to `Cargo.toml` and `crates/storage/Cargo.toml`.
- [x] 1.2 Implement `LibraryDatabase` in `crates/storage/src/db.rs` with tables for `tracks`, `playlists`, `playlist_tracks`, and `favorites`.
- [x] 1.3 Implement CRUD methods: `upsert_tracks`, `load_library`, `toggle_favorite`, `is_favorite`, `get_favorites`, and `create_playlist`.
- [x] 1.4 Add unit tests verifying schema creation, track upsert, favorites toggling, and playlist persistence. Verify with `cargo test -p spoty-storage`.

## 2. UI Favorites & Playlist Creation (`spoty-ui`)

- [x] 2.1 Initialize `LibraryDatabase` in `SpotyApp` and populate library cache on startup.
- [x] 2.2 Add interactive heart (`♡` / `💚`) button in the bottom player bar and playlist track table connected to database favorite state.
- [x] 2.3 Add "+ Create Playlist" button in the "Your Library" sidebar that creates a custom playlist and saves it to SQLite.
- [x] 2.4 Run full workspace test suite and build verification with `cargo test --workspace`.
