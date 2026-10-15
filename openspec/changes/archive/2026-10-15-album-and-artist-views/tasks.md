# Tasks: album-and-artist-views

## 1. Navigation and Routing (`crates/ui/src/navigation.rs`)

- [x] 1.1 Add `Route::Artist(ArtistId)` variant to `Route` enum.
- [x] 1.2 Write unit tests in `crates/ui/src/navigation.rs` covering artist route navigation. Verify with `cargo test -p spoty-ui`.

## 2. Dedicated Album View (`crates/ui/src/views/album.rs`)

- [x] 2.1 Implement `AlbumView` with Spotify full-bleed hero banner, cover art, artist link, and metadata summary.
- [x] 2.2 Implement album tracklist table with column headers, play on hover, favorite toggle, and context menus.
- [x] 2.3 Expose `AlbumView` in `crates/ui/src/views/mod.rs`. Verify compilation with `cargo check -p spoty-ui`.

## 3. Dedicated Artist View (`crates/ui/src/views/artist.rs`)

- [x] 3.1 Implement `ArtistView` with artist portrait header banner, verified badge, and play all button.
- [x] 3.2 Implement "Popular Tracks" top list table with playback and context menu support.
- [x] 3.3 Implement "Albums" discography cards that navigate to `Route::Album`.
- [x] 3.4 Expose `ArtistView` in `crates/ui/src/views/mod.rs`. Verify compilation with `cargo check -p spoty-ui`.

## 4. Shell Wiring and Deep Linking (`crates/ui/src/app.rs`)

- [x] 4.1 Wire up `Route::Album` and `Route::Artist` in `SpotyApp::update`.
- [x] 4.2 Make artist entries in sidebar and Home carousel clickable to navigate to `Route::Artist`.
- [x] 4.3 Update track tables in `playlist.rs`, `songs.rs`, and `search.rs` to make artist and album names clickable links.
- [x] 4.4 Verify compilation with `cargo check -p spoty-ui`.

## 5. Verification and Quality Gates

- [x] 5.1 Run full workspace test suite `cargo test --workspace`.
- [x] 5.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
