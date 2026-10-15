# Design: view-action-dispatcher

## Architectural Overview

```text
                                 ┌───────────────────────┐
                                 │     ViewContext       │
                                 │  - catalog / library  │
                                 │  - active_track_id    │
                                 │  - is_fav predicate   │
                                 └───────────┬───────────┘
                                             │ Reads context
                                             ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              Immediate-Mode Views                               │
│                                                                                 │
│   HomeView::show(ui, &ctx)           ──┐                                        │
│   PlaylistView::show(ui, &ctx, id)   ──┼──► Returns Option<ViewAction>          │
│   SongsView::show(ui, &ctx, mode)    ──┤                                        │
│   SearchView::show(ui, &ctx, &query) ──┘                                        │
└────────────────────────────────────────────┬────────────────────────────────────┘
                                             │ Emits single action
                                             ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      SpotyApp Central Action Dispatcher                         │
│                                                                                 │
│   fn handle_view_action(&mut self, action: ViewAction) {                        │
│       match action {                                                            │
│           ViewAction::PlayTrack(t) => self.play_track(t),                       │
│           ViewAction::ToggleFavorite(id) => self.toggle_favorite(id),           │
│           ViewAction::Navigate(route) => self.nav.push(route),                  │
│           ...                                                                   │
│       }                                                                         │
│   }                                                                             │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Seam & Interface Definition

### `ViewContext`

```rust
pub struct ViewContext<'a> {
    pub library: &'a spoty_storage::Library,
    pub current_track_id: Option<&'a spoty_core::TrackId>,
    pub is_favorite: &'a dyn Fn(&spoty_core::TrackId) -> bool,
}
```

### `ViewAction`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewAction {
    Navigate(Route),
    PlayTrack(Track),
    PlayAll(Vec<Track>),
    ToggleFavorite(TrackId),
    AddToQueue(Track),
    AddToPlaylist(PlaylistId, TrackId),
    CreatePlaylistWithTrack(TrackId),
    EditPlaylist(PlaylistId),
    DeletePlaylist(PlaylistId),
    RemoveTrackFromPlaylist(PlaylistId, TrackId),
}
```

### Streamlined View Signatures

```rust
// crates/ui/src/views/home.rs
impl HomeView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>) -> Option<ViewAction>;
}

// crates/ui/src/views/playlist.rs
impl PlaylistView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>, playlist_id: &PlaylistId) -> Option<ViewAction>;
}

// crates/ui/src/views/songs.rs
impl SongsView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>, mode: SongsMode) -> Option<ViewAction>;
}

// crates/ui/src/views/search.rs
impl SearchView {
    pub fn show(ui: &mut egui::Ui, ctx: &ViewContext<'_>, query: &mut String) -> Option<ViewAction>;
}
```

## Key Technical Decisions

### 1. Eliminating Callback Closure Accumulators
Immediate-mode GUIs only process one user interaction per widget per frame (e.g. one clicked button or row). Passing 10 mutable closures to capture that single click is unnecessary overhead that clutters function signatures.
Returning `Option<ViewAction>` aligns with egui's immediate-mode return pattern (like `egui::Response::clicked()`), allowing views to be pure presentation modules that produce a declarative intent.

### 2. Centralized Action Dispatcher on `SpotyApp`
In `app.rs`, the router simply calls:
```rust
let action = match current_route {
    Route::Home => HomeView::show(ui, &view_ctx),
    Route::Search => SearchView::show(ui, &view_ctx, &mut self.search_query),
    Route::Playlist(id) => PlaylistView::show(ui, &view_ctx, &id),
    Route::AllSongs => SongsView::show(ui, &view_ctx, SongsMode::All),
    Route::Favorites => SongsView::show(ui, &view_ctx, SongsMode::Liked),
    Route::Settings => SettingsView::show(ui, ...),
};

if let Some(action) = action {
    self.handle_view_action(action);
}
```
This replaces over 300 lines of boilerplate with a single 15-line router and one centralized `handle_view_action` function.

### 3. Context Menu Streamlining (`track_menu.rs`)
`TrackMenuActions` currently holds 6 mutable closures:
```rust
pub struct TrackMenuActions<'a> {
    pub is_fav: bool,
    pub custom_playlists: &'a [Playlist],
    pub current_playlist_id: Option<&'a PlaylistId>,
    pub on_toggle_fav: &'a mut dyn FnMut(TrackId),
    ...
}
```
This is refactored into:
```rust
pub fn render_track_context_menu(
    ui: &mut egui::Ui,
    track: &Track,
    custom_playlists: &[Playlist],
    current_playlist_id: Option<&PlaylistId>,
    is_fav: bool,
) -> Option<ViewAction>;
```
The track context menu directly returns `Some(ViewAction)` when a menu item is clicked, eliminating another nested layer of closure passing.

## Testability Gains

Views can now be unit-tested without mock closures or UI scaffolding. Tests simply instantiate `ViewContext` with static data, simulate an egui interaction, and assert on the returned `ViewAction`.
