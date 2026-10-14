## Why

While the basic audio playback engine and demo catalog work, the UI needs Spotify's complete visual architecture: multi-view routing (Home, Search, Playlist/Album detail views, Settings), navigation history (Back/Forward), and full queue management (Next/Previous, shuffle, repeat, and continuous playback).

## What Changes

- Add a router in `spoty-ui` supporting `Home`, `Search`, `PlaylistDetail`, `AlbumDetail`, and `Settings` views with a navigation history stack.
- Implement Spotify's Hero header with dynamic gradient backdrops on playlist and album pages.
- Implement queue management in `spoty-audio` or `spoty-ui` with track ordering, shuffle, repeat, and automatic progression to the next track on `TrackEnded`.
- Add a Settings view allowing users to switch between the Demo provider and a custom local folder via OpenDAL.

## Capabilities

### New Capabilities
- `ui/navigation-and-views`: Multi-view desktop routing system with history stack, rich Spotify home cards, browse genre grid, and playlist hero detail view.
- `audio/queue-management`: Track queue state management, playlist queue generation, shuffle/repeat sequence computation, and automatic next-track advancement.

### Modified Capabilities
<!-- None: additive features -->

## Impact

- Replaces the simple central panel grid with rich, dedicated view components.
- Connects next (`⏭`) and previous (`⏮`) player buttons to active queue progression.
- Adds user-configurable storage provider settings.
