# Proposal: view-action-dispatcher

## Why

In `crates/ui/src/views/`, all view modules (`HomeView`, `PlaylistView`, `SongsView`, `SearchView`) are designed as shallow function wrappers with excessive interface surface area:
- `PlaylistView::show` takes 14 separate parameters, 10 of which are mutable closure callbacks.
- `HomeView::show` takes 11 parameters, 8 of which are closure callbacks.
- `SongsView::show` takes 11 parameters, 8 of which are closure callbacks.
- `SearchView::show` takes 10 parameters, 7 of which are closure callbacks.

To call these views, the caller in `crates/ui/src/app.rs` must define 8–10 temporary `Option` accumulators before invoking the view, bind closures that mutate those accumulators on user clicks, and then unpack the identical actions after the view returns:
```rust
let mut nav_action = None;
let mut play_action = None;
let mut toggle_fav_action = None;
let mut add_to_queue_action = None;
let mut add_to_playlist_action = None;
let mut create_pl_action = None;
// ... pass 10 closures into View::show ...
// ... unpack 50 lines of action execution ...
```
This identical 50-line unpacking sequence is duplicated verbatim across `Route::Home`, `Route::Search`, `Route::AllSongs`, `Route::Favorites`, and `Route::Playlist`—introducing over 300 lines of pure boilerplate in `app.rs`. Furthermore, unit testing any view in isolation requires constructing 10 dummy mock closures, making views brittle and difficult to test.

## What Changes

1. **Unified View Context (`crates/ui/src/views/context.rs` / `ViewContext`)**:
   - Encapsulate read-only presentation inputs (catalog reference, currently playing track ID, active route) into a compact `ViewContext` struct.
2. **Declarative View Action Enum (`ViewAction`)**:
   - Model all view-initiated operations as strongly-typed enum variants:
     - `PlayTrack(Track)`
     - `PlayAll(Vec<Track>)`
     - `ToggleFavorite(TrackId)`
     - `AddToQueue(Track)`
     - `AddToPlaylist(PlaylistId, TrackId)`
     - `CreatePlaylistWithTrack(TrackId)`
     - `Navigate(Route)`
     - `EditPlaylist(PlaylistId)`
     - `DeletePlaylist(PlaylistId)`
3. **Streamlined View Interface**:
   - Reduce view signatures from 14 parameters down to 2 parameters:
     ```rust
     pub fn show(ui: &mut egui::Ui, ctx: &ViewContext) -> Option<ViewAction>;
     ```
4. **Centralized Action Dispatcher on `SpotyApp`**:
   - Replace 300+ lines of duplicated closure unpacking with a single `handle_view_action(&mut self, action: ViewAction)` method.
5. **Widget Simplification (`crates/ui/src/widgets/track_menu.rs`)**:
   - Simplify `render_track_context_menu` to return `Option<TrackMenuAction>` rather than receiving 6 mutable function pointer closures.

## Capabilities

### New Capabilities
- `ui/view-action-dispatch`: Declarative user intent handling where immediate-mode views return typed action values to a centralized shell dispatcher.

### Modified Capabilities
- `ui/navigation-and-views`: Refactors view function interfaces to consume `ViewContext` and emit `ViewAction`.

## Impact

- **Crates Affected:**
  - `crates/ui`: Refactors `views/*`, `widgets/track_menu.rs`, and deletes ~300 lines from `src/app.rs`.
- **Breaking Changes:** Internal to `spoty-ui`. Does not affect `spoty-core`, `spoty-audio`, or `spoty-storage`.
