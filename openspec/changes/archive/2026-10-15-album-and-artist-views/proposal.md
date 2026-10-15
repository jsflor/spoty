# Proposal: album-and-artist-views

## Why

While `crates/storage` aggregates `library.albums` and `library.artists`, Spoty currently treats `Route::Album` as an unfinished placeholder (`ui.label("Album view")`) and completely lacks a `Route::Artist` route and `ArtistView`. Furthermore, clicking on an artist in the library sidebar or Home carousel does nothing, and the Artist and Album columns in track tables are inert text. Users need full Spotify-grade Album and Artist views to explore discographies and navigate their collections organically.

## What Changes

1. **Routing & Navigation (`crates/ui/src/navigation.rs`)**:
   - Add `Route::Artist(ArtistId)` to the navigation route enum.
   - Support back/forward stack traversal for artist profiles.
2. **Dedicated Album View (`crates/ui/src/views/album.rs`)**:
   - Implement `AlbumView` with Spotify full-bleed hero banner: 160×160 artwork, "ALBUM" badge, album title, artist link, release year, track count, total duration, and prominent circular green Play button.
   - Render album track table with disc/track numbers, duration, favorite toggle, and track context menu.
3. **Dedicated Artist View (`crates/ui/src/views/artist.rs`)**:
   - Implement `ArtistView` with full-bleed artist header banner: large circular artist portrait or banner image, artist name, verified artist badge, and circular green Play button (playing all artist tracks).
   - "Popular Tracks" top list showing the artist's most prominent tracks with playback controls and context menus.
   - "Albums & Singles" discography section with cards that navigate directly to `Route::Album`.
4. **Table Column Navigation Links**:
   - Make Artist and Album columns in `PlaylistView`, `SongsView`, `SearchView`, and `AlbumView` interactive links that highlight on hover and navigate to `Route::Artist` and `Route::Album`.
   - Wire up artist clicks in the sidebar and Home carousel.

## Capabilities

### New Capabilities
- `ui/album-view`: Full album detail page with hero artwork, release metadata, one-click album playback, and complete tracklist.
- `ui/artist-view`: Artist profile page with hero portrait banner, popular tracks list, and albums discography cards.

### Modified Capabilities
- `ui/navigation-and-views`: Extends routing and track table columns with deep-linking to albums and artists.

## Impact

- Affects `crates/ui/src/navigation.rs`, `crates/ui/src/app.rs`, `crates/ui/src/views/mod.rs`, `crates/ui/src/views/home.rs`, `crates/ui/src/views/playlist.rs`, `crates/ui/src/views/songs.rs`, `crates/ui/src/views/search.rs`.
- Creates `crates/ui/src/views/album.rs` and `crates/ui/src/views/artist.rs`.
