# Design: all-songs-and-metadata-recovery

## Architecture & Data Flow

```text
       Scanned Track Bytes & Path
                  │
                  ▼
   [crates/storage/src/metadata.rs]
   - Lofty embedded tags (Title, Artist, Album, Picture)
   - Fallback: parse_path_heuristics("Artist/Album/01 Title.mp3")
                  │
                  ▼
   [crates/storage/src/provider.rs: Library::aggregate_entities()]
   - Extract unique Artist models -> library.artists
   - Extract unique Album models -> library.albums
                  │
                  ▼
   [crates/ui/src/app.rs & navigation.rs]
   - Route::AllSongs ──► SongsView (all library.tracks)
   - Route::LikedSongs ──► SongsView (filtered to favorites)
   - Sidebar Pinned Rows: [Liked Songs], [All Songs]
   - Interactive Chips: [Playlists], [Songs], [Artists]
```

## Technical Details

### 1. Enhanced Metadata Fallback (`crates/storage/src/metadata.rs`)
- Expand `parse_path_heuristics(path_str: &str) -> (String, String, Option<String>)`:
  - Split path components.
  - If components >= 3 (e.g. `[..., "Pink Floyd", "The Dark Side of the Moon", "01 Speak to Me.mp3"]`), extract:
    - Artist: `Pink Floyd`
    - Album: `The Dark Side of the Moon`
    - Title: Clean track stem stripping leading digits and punctuation (`01 `, `01 - `, `01. `).
  - If components == 2 (e.g. `["Daft Punk", "Get Lucky.flac"]`), extract:
    - Artist: `Daft Punk`
    - Album: None
    - Title: Clean track stem.
  - Fall back to standard delimiter heuristics (`" - "` or `'_'`) and finally `"Unknown Artist"`.

### 2. Automatic Entity Aggregation (`crates/storage/src/provider.rs`)
- Add method `Library::rebuild_indices(&mut self)` or `Library::aggregate_artists_and_albums(&mut self)`:
  - Iterates over `tracks`.
  - Collects unique `artist` strings (excluding empty / unknown or grouping them), creating `Artist { id: ArtistId::new(format!("art_{}", slug)), name: artist_name, ... }`.
  - Collects unique `album` strings, creating `Album { id: AlbumId::new(format!("alb_{}", slug)), title: album_name, artist: artist_name, ... }`.
  - Automatically called whenever `OpenDalProvider::fetch_library` or `LibraryDatabase::load_library` loads tracks.

### 3. Route Expansion (`crates/ui/src/navigation.rs`)
- Add variants:
  - `Route::AllSongs`
  - `Route::LikedSongs`

### 4. Songs View (`crates/ui/src/views/songs.rs`)
- Implement `SongsView::show(ui, &Library, mode, current_track_id, is_fav, on_toggle_fav, on_play_track, on_play_all)`
  - `SongsMode::All`:
    - Hero banner: Dark teal/slate gradient, large "🎵" artwork, Title: "All Songs", subtitle: "Local Library • {count} tracks".
  - `SongsMode::Liked`:
    - Hero banner: Spotify purple gradient (`#450af5` to `#8e8ee5`), large "💜" artwork, Title: "Liked Songs", subtitle: "Spoty • {count} songs".
  - Interactive Table:
    - Sort state: `sort_by: Column`, `ascending: bool`.
    - Column headers `#`, `Title`, `Artist`, `Album`, `Duration` are clickable to toggle sorting.
    - Inline search text field to filter songs without leaving the page.
    - Play circle button on action bar to play all sorted/filtered tracks.
    - Interactive heart button and right-click context menu on each row.

### 5. Sidebar Integration (`crates/ui/src/app.rs`)
- Add pinned entries above custom playlists:
  - "Liked Songs" row (with 40x40 purple gradient square and white heart).
  - "All Songs" row (with 40x40 slate card and music note icon).
- Category filter chips:
  - Track active sidebar filter: `All`, `Playlists`, `Songs`, `Artists`.
  - Clicking "Songs" sets the filter or navigates directly to `Route::AllSongs`.
