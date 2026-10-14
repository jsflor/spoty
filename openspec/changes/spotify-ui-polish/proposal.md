## Why

Spoty's audio engine and storage providers work reliably, but the UI currently relies on generic egui panels, standard sliders, and emoji icons. Aligning Spoty with the real Spotify desktop experience ("Cards on Canvas" architecture, Encore design system tokens, dedicated 72px player bar with custom hover-responsive sliders, rich playlist hero gradient, and quick-play cards) elevates Spoty into a modern, polished native music player.

## What Changes

- Apply Spotify's "Cards on Canvas" layout architecture: pure black canvas (`#000000`) with 8px panel gaps, housing elevated rounded cards (`#121212`, 8px radius) for the sidebar and main central content.
- Replace top navigation with circular pill buttons `( < )` `( > )` and a rounded search pill input.
- Replace the player bar with a Spotify-authentic 72px persistent bar featuring a 3-section balanced layout (Left: 30%, Center: 40%, Right: 30%), custom slim 4px hover-responsive progress and volume sliders with interactive scrub thumbs, and vector control buttons.
- Polish `HomeView` with a dynamic time-based greeting, 2-column quick play cards with hover green play buttons, and hover-elevated featured cards.
- Polish `PlaylistView` with a full-bleed gradient hero banner, 48px circular green play button, and an interactive track list table with hover row highlights and track numbers transforming into play icons on hover.
- Update `theme.rs` with exact Spotify Encore design tokens extracted from production Spotify assets.

## Capabilities

### New Capabilities
- `ui/spotify-player-bar`: Dedicated 72px persistent bottom player bar with a balanced 3-section layout, custom hover-responsive progress/volume sliders with circular scrub handles, and vector playback control icons.

### Modified Capabilities
- `ui/navigation-and-views`: Upgrades the shell architecture to Spotify's "Cards on Canvas" structure with rounded floating panels, and enriches `HomeView` and `PlaylistView` with Spotify-styled cards, gradient banners, and interactive table rows.

## Impact

- Affects `crates/ui/src/theme.rs`, `crates/ui/src/app.rs`, `crates/ui/src/views/home.rs`, `crates/ui/src/views/playlist.rs`, and adds custom slider and widget painters in `crates/ui/src/widgets/`.
- No breaking changes to core, storage, or audio crates.
