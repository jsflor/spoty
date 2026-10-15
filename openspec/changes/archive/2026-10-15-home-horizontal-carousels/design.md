# Design: home-horizontal-carousels

## Architecture & Layout

```text
[HomeView]
   │
   ├─ Greeting & Quick-Play Grid (2-3 columns, fixed height)
   │
   ├─ Section: "Featured & Made For You"
   │    ├─ Header: Title + [‹] [›] carousel chevron buttons
   │    └─ ScrollArea::horizontal() ──► [Track Card 1] [Track Card 2] ...
   │
   ├─ Section: "Your Playlists & Mixes"
   │    ├─ Header: Title + [‹] [›] carousel chevron buttons
   │    └─ ScrollArea::horizontal() ──► [Playlist Card 1] [Playlist Card 2] ...
   │
   └─ Section: "Artists You Might Like"
        ├─ Header: Title + [‹] [›] carousel chevron buttons
        └─ ScrollArea::horizontal() ──► [Circular Artist Card 1] [Artist Card 2] ...
```

## Detailed Technical Changes

### 1. Carousel Scroll Implementation
- Use `egui::ScrollArea::horizontal().id_salt("featured_tracks_scroll").horizontal_scroll_offset(offset)`
- Track scroll offset in egui temporary data memory (`ui.data_mut`):
  ```rust
  #[derive(Debug, Clone, Default)]
  pub struct CarouselState {
      pub offset: f32,
  }
  ```
- In the section header:
  - Add circular pill buttons: `‹` (left) and `›` (right).
  - Clicking `‹` scrolls left by one page (`offset -= 500.0`).
  - Clicking `›` scrolls right by one page (`offset += 500.0`).
  - Buttons dim or disable when at edges.
- Cards maintain fixed sizes:
  - Track card: `180px x 246px`, cover `156px x 156px` with 6px corner radius.
  - Playlist card: `180px x 246px`, cover `156px x 156px` with 6px corner radius.
  - Artist card: `160px x 210px`, circular avatar `136px` diameter with "👤" or custom image.

### 2. Interaction & Visuals
- Preserves Spotify card hover elevation and floating green circular play button (`#1ed760`) with play triangle.
- Clicking cards navigates to the respective `Route::Playlist` or plays the track / artist.
