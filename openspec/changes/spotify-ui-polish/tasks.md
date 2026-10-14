## 1. Design Tokens and Theme (`crates/ui/src/theme.rs`)

- [x] 1.1 Update `crates/ui/src/theme.rs` with Spotify Encore color tokens (Canvas, Base, Elevated, Highlight, Subdued text, Spotify Green). Verify compilation with `cargo check -p spoty-ui`.
- [x] 1.2 Configure `setup_spotify_theme` for panels, rounded widget corner radii (4px, 8px, 500px), and scrollbar styling. Verify theme initialization in `main.rs`.

## 2. Custom Spotify Widgets (`crates/ui/src/widgets/`)

- [x] 2.1 Implement `SpotifySlider` in `crates/ui/src/widgets/slider.rs`: 4px track height, white progress fill turning to Spotify green on hover, and circular scrub thumb. Verify with `cargo check -p spoty-ui`.
- [x] 2.2 Implement vector playback icon painters (Play, Pause, Shuffle, Repeat, Heart, Speaker) in `crates/ui/src/widgets/icons.rs`. Verify with `cargo check -p spoty-ui`.

## 3. Shell Architecture and Player Bar (`crates/ui/src/app.rs`)

- [x] 3.1 Restructure root window layout to Spotify "Cards on Canvas" architecture with true black background (`#000000`), 8px gaps, and 8px rounded elevated cards for sidebar and central panel. Verify layout with `cargo check -p spoty-ui`.
- [x] 3.2 Redesign top bar with circular navigation pill buttons and rounded search pill input. Verify navigation buttons compile cleanly.
- [x] 3.3 Overhaul the persistent bottom player bar to 72px fixed height with 3-section layout, track details, `SpotifySlider` timeline and volume controls, and vector playback buttons. Verify with `cargo check -p spoty-ui`.

## 4. Home Screen Polish (`crates/ui/src/views/home.rs`)

- [x] 4.1 Update `HomeView` with dynamic greeting banner and subtle top ambient background. Verify rendering logic compiles.
- [x] 4.2 Implement 2-column "Quick Play" rectangular cards with 48px covers and hover green circle play button. Verify with `cargo check -p spoty-ui`.
- [x] 4.3 Update "Featured / Recent" cards with 160×160 artwork, 6px corner radius, and hover card elevation. Verify with `cargo check -p spoty-ui`.

## 5. Playlist View Polish (`crates/ui/src/views/playlist.rs`)

- [x] 5.1 Implement full-bleed hero banner with vertical color gradient, 160×160 artwork, bold title, and metadata summary. Verify with `cargo check -p spoty-ui`.
- [x] 5.2 Add prominent 48px circular green Play button and action row. Verify with `cargo check -p spoty-ui`.
- [x] 5.3 Polish track list table with header divider line, row hover background, track number -> play button hover swap, active track in green, and tabular time formatting. Verify with `cargo check -p spoty-ui`.

## 6. Workspace Verification

- [x] 6.1 Run full workspace build and test suite. Verify with `cargo test --workspace`.
- [x] 6.2 Verify clean binary build with `cargo build --bin spoty`.
