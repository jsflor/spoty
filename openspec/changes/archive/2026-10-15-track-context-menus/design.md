# Design: track-context-menus

## Architecture & Interaction Flow

```text
Track Row / Card (Right-click or '•••')
       │
       ▼
   egui Context Menu
       ├─ "Add to Playlist" ──► [Select Playlist] ──► db.add_track_to_playlist(pl_id, track_id)
       │                                         ──► library.playlists[i].track_ids.push(track_id)
       │
       ├─ "Add to Queue" ────────────────────────► queue.push_back(track)
       │
       ├─ "Save to / Remove from Liked Songs" ──► db.toggle_favorite(track_id)
       │
       └─ "Remove from this playlist" (if custom)► db.remove_track_from_playlist(pl_id, track_id)
```

## Detailed Component Specifications

### 1. `crates/core/src/queue.rs`
- Add `push_back(&mut self, track: Track)`:
  - Appends `track` to `self.tracks`.
  - Regenerates `shuffle_order` if shuffle is active.
- Add `insert_next(&mut self, track: Track)`:
  - Inserts `track` immediately after `current_index` (or at 0 if no current index).
  - Regenerates `shuffle_order` if shuffle is active.
- Add unit tests in `queue.rs`.

### 2. Context Menu Component (`crates/ui/src/widgets/context_menu.rs` or inline helper)
- Create a shared context menu helper:
  ```rust
  pub fn render_track_context_menu(
      ui: &mut egui::Ui,
      track: &Track,
      is_fav: bool,
      custom_playlists: &[Playlist],
      current_playlist_id: Option<&PlaylistId>,
      on_toggle_fav: &mut impl FnMut(TrackId),
      on_add_to_queue: &mut impl FnMut(Track),
      on_add_to_playlist: &mut impl FnMut(PlaylistId, TrackId),
      on_create_playlist_with_track: &mut impl FnMut(TrackId),
      on_remove_from_playlist: &mut impl FnMut(PlaylistId, TrackId),
  )
  ```
- Submenu or dropdown for "Add to playlist":
  - Shows custom playlists.
  - Clicking any playlist adds the track and closes the menu.
  - Option to "+ New Playlist" with this track.

### 3. View Integrations
- Update `PlaylistView`, `SongsView`, `SearchView`, and `HomeView` cards to attach this context menu.
- Ensure `SearchView` renders with full Spotify Encore dark table styling.
