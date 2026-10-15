# Design: album-and-artist-views

## Architecture & Navigation Flow

```text
Any Track Row / Card / Sidebar
       │
       ├─ Click Album Name / Card ──► Route::Album(AlbumId)
       │                                     │
       │                                     ▼
       │                              [AlbumView]
       │                              - Full-bleed gradient hero (cover, title, artist link, year)
       │                              - Play All album tracks
       │                              - Album tracklist table (#, Title, Duration, Heart, Context Menu)
       │
       └─ Click Artist Name / Avatar ─► Route::Artist(ArtistId)
                                             │
                                             ▼
                                      [ArtistView]
                                      - Artist hero portrait banner & verified badge
                                      - Play All artist tracks
                                      - "Popular Tracks" top list
                                      - "Albums" discography cards ──► Route::Album
```

## Detailed Component Specifications

### 1. `Route::Artist` in `crates/ui/src/navigation.rs`
- Add `Artist(ArtistId)` variant to `Route`.
- Update tests for navigation stack pushing and popping.

### 2. `AlbumView` in `crates/ui/src/views/album.rs`
- Resolves `Album` by `AlbumId` from `library.albums`.
- Gathers album tracks from `album.track_ids` or filters `library.tracks` matching the album and artist.
- Full-bleed hero banner:
  - Deep atmospheric color tone (e.g. `#1e3240` slate charcoal gradient).
  - 160×160 cover artwork with 4px corner radius.
  - "ALBUM" uppercase badge.
  - Bold album title (38pt).
  - Clickable artist name link (navigating to `Route::Artist`).
  - Metadata: Release year (if known), track count, total minutes.
- Action Bar:
  - 48px circular green play button (plays all album tracks).
- Interactive Track Table:
  - Columns: `#`, `Title`, `Duration`, `Heart`, Context Menu.

### 3. `ArtistView` in `crates/ui/src/views/artist.rs`
- Resolves `Artist` by `ArtistId` from `library.artists`.
- Gathers all tracks belonging to this artist.
- Top Hero Section:
  - Full-bleed atmospheric header (`#242436`).
  - 140×140 circular artist portrait.
  - Verified artist badge `✓ Verified Artist`.
  - Large artist name (42pt bold white).
  - Monthly listeners / track count indicator.
- Action Bar:
  - 48px circular green play button.
- "Popular Tracks" Section:
  - Top 5 or 10 tracks by the artist in an interactive table row format.
- "Albums" Section:
  - Horizontal carousel or grid of album cards for this artist.
  - Clicking an album card pushes `Route::Album(album.id)`.

### 4. Interactive Links in Track Tables
- In `PlaylistView`, `SongsView`, `SearchView`, and `AlbumView`:
  - When rendering the `Artist` column: wrap in a clickable text button/label that underlines on hover and triggers `on_navigate(Route::Artist(artist_id))`.
  - When rendering the `Album` column: wrap in a clickable text button/label that underlines on hover and triggers `on_navigate(Route::Album(album_id))`.
