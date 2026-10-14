# Tasks: all-songs-and-metadata-recovery

## 1. Metadata Heuristics & Entity Aggregation (`crates/storage/`)

- [x] 1.1 Implement `parse_path_heuristics` in `crates/storage/src/metadata.rs` to extract Artist, Album, and Title from directory paths when tags are missing.
- [x] 1.2 Implement `Library::aggregate_artists_and_albums` in `crates/storage/src/provider.rs` to populate `library.artists` and `library.albums` from track lists.
- [x] 1.3 Wire up entity aggregation in `OpenDalProvider::fetch_library` and `LibraryDatabase::load_library`.
- [x] 1.4 Write unit tests in `metadata.rs` and `provider.rs` for path heuristics and entity aggregation. Verify with `cargo test -p spoty-storage`.

## 2. Navigation and Routing (`crates/ui/src/navigation.rs`)

- [x] 2.1 Add `Route::AllSongs` and `Route::LikedSongs` to `Route` enum.
- [x] 2.2 Update navigation tests in `crates/ui/src/navigation.rs`. Verify with `cargo test -p spoty-ui`.

## 3. All Songs & Liked Songs View (`crates/ui/src/views/songs.rs`)

- [x] 3.1 Create `crates/ui/src/views/songs.rs` implementing `SongsView` with support for `SongsMode::All` and `SongsMode::Liked`.
- [x] 3.2 Implement full-bleed hero banner with custom gradients (Purple for Liked Songs, Slate/Teal for All Songs).
- [x] 3.3 Implement interactive track list table with clickable column headers for sorting (Title, Artist, Album, Duration, Track Number) and real-time filter text edit.
- [x] 3.4 Wire up playback and favorite callbacks. Verify compilation with `cargo check -p spoty-ui`.

## 4. Sidebar Integration (`crates/ui/src/app.rs`)

- [x] 4.1 Add pinned "Liked Songs" and "All Songs" rows at the top of the Library sidebar.
- [x] 4.2 Make category chips ("Playlists", "Songs", "Artists") interactive and functional.
- [x] 4.3 Connect `Route::AllSongs` and `Route::LikedSongs` in `SpotyApp::update`.

## 5. Verification and Quality Gates

- [x] 5.1 Run full workspace test suite `cargo test --workspace`.
- [x] 5.2 Verify formatting and clean compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
