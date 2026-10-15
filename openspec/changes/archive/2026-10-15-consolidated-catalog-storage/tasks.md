# Tasks: consolidated-catalog-storage

## 1. Catalog Module Implementation (`crates/storage/src/catalog.rs`)

- [x] 1.1 Implement `Catalog` struct wrapping `LibraryDatabase` and `RwLock<CatalogInner>`.
- [x] 1.2 Implement `open` and `open_in_memory` constructors that preload the in-memory favorites `HashSet<TrackId>`.
- [x] 1.3 Implement atomic favorites management (`is_favorite`, `toggle_favorite`, `get_favorites`).
- [x] 1.4 Implement atomic playlist CRUD operations (`create_playlist`, `rename_playlist`, `delete_playlist`, `add_track_to_playlist`, `remove_track_from_playlist`).
- [x] 1.5 Implement asynchronous `sync_provider` workflow for background library ingestion and SQLite upserts.
- [x] 1.6 Write comprehensive unit tests in `crates/storage/tests/catalog_test.rs`. Verify with `cargo test -p spoty-storage`.

## 2. Shell Migration (`crates/ui/src/app.rs`)

- [x] 2.1 Replace `SpotyApp.db` and `SpotyApp.library` fields with a single `SpotyApp.catalog: Arc<Catalog>`.
- [x] 2.2 Replace per-frame SQLite queries in `is_fav_closure` with `catalog.is_favorite(...)`.
- [x] 2.3 Remove dual-write mutations across playlist modals, context menus, and view callback blocks in `app.rs`.
- [x] 2.4 Refactor `switch_provider` to use non-blocking asynchronous `catalog.sync_provider`.
- [x] 2.5 Verify compilation with `cargo check -p spoty-ui`.

## 3. Test Suite Verification and Quality Gates

- [x] 3.1 Update `crates/ui/tests/playlist_management_test.rs` and `crates/ui/tests/all_songs_test.rs` to assert via `Catalog`.
- [x] 3.2 Run full workspace test suite `cargo test --workspace`.
- [x] 3.3 Verify formatting with `cargo fmt --all -- --check` and compiler checks with `cargo check --workspace`.
