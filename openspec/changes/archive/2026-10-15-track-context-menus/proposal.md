# Proposal: track-context-menus

## Why

Currently, track rows in Spoty only respond to left clicks (to play the song) and simple favorite toggling. Users have no way to add tracks from Home, Search, or All Songs into custom playlists, nor can they append tracks to their active playback queue. Furthermore, Search view uses generic, non-interactive table cells rather than Spotify-grade hoverable rows with options. Adding rich track context menus and playlist assignment bridges the workflow gap between song discovery and personal curation.

## What Changes

1. **Queue Management Capabilities (`crates/core/src/queue.rs`)**:
   - Add `push_back(&mut self, track: Track)` to append a track to the end of the upcoming queue.
   - Add `insert_next(&mut self, track: Track)` to insert a track directly after the currently playing song ("Play Next").
2. **Track Context Menu & Actions**:
   - Provide right-click context menus across all track presentations (`PlaylistView`, `SongsView`, `SearchView`, and `HomeView` cards):
     - **"Add to Playlist"**: Flyout submenu listing custom playlists, plus a "+ New Playlist" action that creates a playlist and adds the track immediately.
     - **"Add to Queue"**: Appends the track to the current active playback queue.
     - **"Save to Your Liked Songs" / "Remove from Your Liked Songs"**: Toggles favorite status in SQLite.
     - **"Remove from this playlist"**: Visible when inside a custom playlist.
3. **Search View Table Polish (`crates/ui/src/views/search.rs`)**:
   - Align `SearchView` table styling with `PlaylistView` and `SongsView` (row hover highlights, track thumbnail, tabular layout, interactive heart, and context menus).

## Capabilities

### New Capabilities
- `ui/track-context-menu`: Unified right-click context menu and options action for track rows enabling playlist assignment, queue insertion, and favorite toggling across all views.

### Modified Capabilities
- `audio/queue-management`: Enhanced with `push_back` and `insert_next` queue manipulation primitives.
- `ui/navigation-and-views`: Standardized interactive track rows across Search, Playlist, Songs, and Home views.

## Impact

- Affects `crates/core/src/queue.rs` and its unit tests.
- Affects `crates/ui/src/views/search.rs`, `crates/ui/src/views/playlist.rs`, `crates/ui/src/views/songs.rs`, `crates/ui/src/views/home.rs`, and `crates/ui/src/app.rs`.
- Zero breaking changes to storage providers or audio workers.
