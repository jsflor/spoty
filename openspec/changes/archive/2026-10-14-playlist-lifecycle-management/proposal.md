# Proposal: playlist-lifecycle-management

## Why

Users can currently create new custom playlists via the sidebar `+` button, but there is no mechanism to delete custom playlists, rename them, or remove individual tracks from them. The options (`•••`) button on the playlist view is a non-functional placeholder, and right-clicking playlists in the sidebar produces no context menu. This leaves users with clutter and no control over their curated music.

## What Changes

1. **Storage Layer (`crates/storage/src/db.rs`)**:
   - Add `delete_playlist(playlist_id: &PlaylistId)` to SQLite, deleting both the playlist entity and associated track mappings (`playlist_tracks`).
   - Add `rename_playlist(playlist_id: &PlaylistId, new_title: &str, new_description: Option<&str>)` to update playlist titles and descriptions.
   - Add `remove_track_from_playlist(playlist_id: &PlaylistId, track_id: &TrackId)` to delete track associations from `playlist_tracks`.
2. **UI Navigation and Deletion Flow (`crates/ui/src/app.rs`)**:
   - Provide a right-click context menu on sidebar playlist rows containing "Delete playlist" and "Rename playlist".
   - Introduce a confirmation modal dialog for playlist deletion (*"Delete from Your Library? This will delete [Playlist Name] from Your Library."*) to prevent accidental loss.
   - When a playlist is deleted, immediately purge it from the in-memory library state and navigate back to `Route::Home` if that playlist was actively open.
   - Support inline renaming of playlists.
3. **Playlist View Options and Track Removal (`crates/ui/src/views/playlist.rs`)**:
   - Make the `•••` action button interactive, opening a popup menu with "Edit details" and "Delete playlist".
   - Allow removing individual tracks from custom playlists via track row context menus or dedicated removal buttons.

## Capabilities

### New Capabilities
- `storage/playlist-crud-operations`: Extends the SQLite persistence layer with full playlist lifecycle management: deletion with cascading track association cleanup, renaming, and individual track removal.

### Modified Capabilities
- `ui/navigation-and-views`: Enhances playlist navigation with right-click context menus in the library sidebar, interactive `•••` action menu in playlist header, confirmation modal dialogs, and track removal.

## Impact

- Affects `crates/storage/src/db.rs` and its unit tests.
- Affects `crates/ui/src/app.rs` and `crates/ui/src/views/playlist.rs`.
- Zero breaking changes to `spoty-core` or `spoty-audio`.
