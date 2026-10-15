# Tasks: album-and-artist-views

## 1. Navigation and Routing (`crates/ui/src/navigation.rs`)

- [ ] 1.1 Add `Route::Artist(ArtistId)` variant to `Route` enum.
- [ ] 1.2 Write unit tests in `crates/ui/src/navigation.rs` covering artist route navigation. Verify with `cargo test -p spoty-ui`.

## 2. Dedicated Album View (`crates/ui/src/views/album.rs`)

- [ ] 2.1 Implement `AlbumView` with Spotify full-bleed hero banner, cover art, artist link, and metadata summary.
- [ ] 2.2 Implement album tracklist table with column headers, play on hover, favorite toggle, and context menus.
- [ ] 2.3 Expose `AlbumView` in `crates/ui/src/views/mod.rs`. Verify compilation with `cargo check -p spoty-ui`.

## 3. Dedicated Artist View (`crates/ui/src/views/artist.rs`)

- [ ] 3.1 Implement `ArtistView` with artist portrait header banner, verified badge, and play all button.
- [ ] 3.2 Implement "Popular Tracks" top list table with playback and context menu support.
- [ ] 3.3 Implement "Albums" discography cards that navigate to `Route::Album`.
- [ ] 3.4 Expose `ArtistView` in `crates/ui/src/views/mod.rs`. Verify compilation with `cargo check -p spoty-ui`.

## 4. Shell Wiring and Deep Linking (`crates/ui/src/app.rs`)

- [ ] 4.1 Wire up `Route::Album` and `Route::Artist` in `SpotyApp::update`.
- [ ] 4.2 Make artist entries in sidebar and Home carousel clickable to navigate to `Route::Artist`.
- [ ] 4.3 Update track tables in `playlist.rs`, `songs.rs`, and `search.rs` to make artist and album names clickable links.
- [ ] 4.4 Verify compilation with `cargo check -p spoty-ui`.

## 5. Verification and Quality Gates

- [ ] 5.1 Run full workspace test suite `cargo test --workspace`.
- [ ] 5.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
