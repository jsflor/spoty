# Proposal: home-horizontal-carousels

## Why

The Home view currently uses `ui.horizontal_wrapped(|ui| ...)` for featured tracks. When cards exceed the panel width, they wrap into multiple lines vertically, breaking Spotify's standard desktop carousel UX. In Spotify desktop, content sections ("Featured & Made For You", "Your Playlists", "Artists") are rendered as compact, single-row horizontally scrollable carousels. Users cannot currently scroll horizontally through tracks or playlists on Home, leading to unnecessary vertical bloat.

## What Changes

1. **Horizontal Carousel Architecture (`crates/ui/src/views/home.rs`)**:
   - Replace vertical card wrapping with dedicated `egui::ScrollArea::horizontal()` carousels for content sections.
   - Support horizontal mouse wheel scrolling, trackpad gestures, and interactive left/right carousel chevron buttons (`‹` / `›`) in section headers.
2. **Multi-Section Content Carousels**:
   - Section 1: "Featured Tracks" (horizontal carousel of track cards with 156x156 artwork, title, artist, and hover green circle play button).
   - Section 2: "Your Playlists & Mixes" (horizontal carousel of playlist cards with artwork, title, and track count).
   - Section 3: "Popular Artists" (horizontal carousel of circular artist cards with artist avatar and name).
3. **Smooth Scroll State**:
   - Provide persistent scroll offset state per carousel so users can navigate forward and backward using header chevron buttons.

## Capabilities

### New Capabilities
- `ui/home-carousels`: Single-row horizontally scrollable carousels on Home with header navigation buttons and smooth trackpad/wheel scrolling for tracks, playlists, and artists.

### Modified Capabilities
- `ui/navigation-and-views`: Enhances `HomeView` layout from wrapped vertical grids to multi-section horizontal carousels adhering to Spotify Encore design patterns.

## Impact

- Affects `crates/ui/src/views/home.rs`.
- Zero breaking changes to `spoty-core`, `spoty-storage`, or `spoty-audio`.
