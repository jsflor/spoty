# Design: playlist-lifecycle-management

## Architecture & Crate Interactions

The change spans `crates/storage` (SQLite database operations) and `crates/ui` (user interaction, context menus, and confirmation dialogs).

```text
[crates/ui/src/app.rs]
       │
       ├─ Right-click on sidebar playlist row ──► context_menu ("Delete playlist", "Rename")
       │
       ├─ Click "Delete playlist" ──────────────► Show Confirmation Modal
       │                                                 │
       │                                            [Confirm Delete]
       │                                                 │
       │   ┌─────────────────────────────────────────────┘
       │   ▼
       ├─ db.delete_playlist(&id) ──────────────► [crates/storage/src/db.rs]
       │                                                │
       │                                          DELETE FROM playlist_tracks WHERE playlist_id = ?
       │                                          DELETE FROM playlists WHERE id = ?
       │
       ├─ Purge from in-memory library.playlists
       │
       └─ If current route == Route::Playlist(deleted_id) ──► nav.push(Route::Home)
```

## Detailed Component Changes

### 1. `crates/storage/src/db.rs`
- Add database methods wrapped in transactions:
  - `delete_playlist(&self, playlist_id: &PlaylistId) -> anyhow::Result<()>`:
    Executes `DELETE FROM playlist_tracks WHERE playlist_id = ?1; DELETE FROM playlists WHERE id = ?1;`.
  - `remove_track_from_playlist(&self, playlist_id: &PlaylistId, track_id: &TrackId) -> anyhow::Result<()>`:
    Executes `DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2;`.
  - `rename_playlist(&self, playlist_id: &PlaylistId, new_title: &str, new_description: Option<&str>) -> anyhow::Result<()>`:
    Executes `UPDATE playlists SET title = ?1, description = ?2 WHERE id = ?3;`.
- Add comprehensive tests in `crates/storage/src/db.rs`:
  - `test_playlist_delete_cascades_tracks()`
  - `test_playlist_remove_track()`
  - `test_playlist_rename()`

### 2. `crates/ui/src/views/playlist.rs`
- Add actions to `PlaylistView::show`:
  - `on_request_delete: &mut impl FnMut(PlaylistId)`
  - `on_rename: &mut impl FnMut(PlaylistId, String)`
  - `on_remove_track: &mut impl FnMut(PlaylistId, TrackId)`
- Replace the static `•••` label with an interactive menu button (`ui.menu_button("•••", ...)` or custom popup):
  - "Edit details": prompts to change title.
  - "Delete playlist": invokes `on_request_delete`.
- On track rows, add a context menu or button to trigger `on_remove_track`.

### 3. `crates/ui/src/app.rs`
- Add state in `SpotyApp`:
  - `pending_delete_playlist: Option<Playlist>`
  - `editing_playlist: Option<(PlaylistId, String)>`
- In sidebar playlist items:
  - Attach `resp.context_menu(|ui| { ... })` offering "Rename" and "Delete playlist".
- Render a centered Spotify-themed confirmation dialog when `pending_delete_playlist.is_some()`:
  - Window title: "Delete from Your Library?"
  - Body: "This will delete **{title}** from **Your Library**."
  - Action buttons: "Cancel" (neutral) and "Delete" (destructive red/white).
  - On confirm: call `self.db.delete_playlist`, remove from `self.library.playlists`, clear modal state, and redirect to `Route::Home` if necessary.

## Trade-offs and Decisions
- **Confirmation Modal vs Immediate Deletion:** Accidental deletion of custom playlists is frustrating. Spotify desktop always confirms deletion. Using a modal overlay in `app.rs` ensures consistency.
- **Cascading SQL Delete:** We use explicit deletion of `playlist_tracks` followed by `playlists` in a single transaction, ensuring database integrity even without foreign key cascade triggers enabled.
