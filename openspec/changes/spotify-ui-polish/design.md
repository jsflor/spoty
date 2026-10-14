## Context

Spoty's desktop interface is written in Rust using `egui` and `eframe`. While functionally sound, its default widget styling and simple flat layout lack the visual polish and design fidelity of Spotify's desktop client. We extracted Spotify's production Encore tokens, layout rules, and CSS structures from `/Users/sebastian.flor/Applications/Spotify.app/Contents/Resources/Apps/xpui.spa`.

See `proposal.md` for overall motivation and `specs/` for behavioral requirements.

## Goals / Non-Goals

**Goals:**
- Implement the "Cards on Canvas" architecture with true black background (`#000000`) and elevated rounded surfaces (`#121212`, 8px radius).
- Build a custom hover-responsive 4px timeline progress slider and volume slider in `crates/ui/src/widgets/` that turns green on hover and exposes a circular scrub thumb.
- Replace system emoji controls with custom-painted vector shapes (Play, Pause, Shuffle, Repeat, Heart, Volume).
- Elevate `HomeView` and `PlaylistView` to match Spotify desktop typography, spacing, card elevations, and full-bleed hero gradients.

**Non-Goals:**
- Modifying backend storage providers (OpenDAL, Subsonic, S3, Spotify API) or core audio playback logic.
- Implementing animated canvas video backgrounds or complex canvas canvas mini-players.

## Decisions

### 1. Encore Design Token Centralization
- **Choice**: Store official Encore color constants (`COLOR_CANVAS`, `COLOR_BG_BASE`, `COLOR_BG_ELEVATED`, `COLOR_BG_HIGHLIGHT`, `COLOR_TEXT_BASE`, `COLOR_TEXT_SUBDUED`, `COLOR_SPOTIFY_GREEN`, `COLOR_GREEN_HOVER`) in `crates/ui/src/theme.rs`.
- **Rationale**: Keeps color values consistent across all views and widgets, matching the exact hex values extracted from Spotify's production CSS.
- **Alternatives considered**: Inline color definitions in individual views (rejected for inconsistency).

### 2. Custom Painter for Sliders (`SpotifySlider`)
- **Choice**: Implement a dedicated `SpotifySlider` widget using `egui::Painter` instead of `egui::Slider`.
- **Rationale**: `egui::Slider` enforces default thumb sizes, padding, and text labels that look like developer tools. A custom painter allows an exact 4px track height, smooth hover transition from subdued white to Spotify green (`#1ed760`), and a circular white thumb handle that appears on hover/drag.
- **Alternatives considered**: Customizing `egui::Visuals` for default slider (rejected because egui's default slider draws numeric labels and bulky frames).

### 3. Custom Vector Glyphs for Playback Controls
- **Choice**: Render playback controls using `egui::Painter` primitives (triangles, rectangles, arcs) and clean typography rather than OS emojis.
- **Rationale**: Emojis render inconsistently across macOS/Linux/Windows and cannot be dynamically colored (e.g., green active dot for shuffle/repeat).
- **Alternatives considered**: Bundling an external icon font file (rejected to avoid additional asset dependencies when vector primitives are simple and self-contained).

### 4. Cards on Canvas Floating Frames
- **Choice**: Render the root viewport with `#000000`, containing two major floating frames: left sidebar card (`#121212`, 8px corner radius) and central content card (`#121212`, 8px corner radius), with an 8px margin and gutter.
- **Rationale**: Matches Spotify's modern desktop client hierarchy.

## Risks / Trade-offs

- [Risk] Custom painter slider could miss corner-case drag events or micro-seeking accuracy → Mitigation: Handle drag state via `ui.interact(rect, id, egui::Sense::click_and_drag())` and clamp values strictly between `0.0` and `duration`.
- [Risk] Fixed layout heights could clip on small windows → Mitigation: Preserve `eframe::NativeOptions` min window size (740×540) and make the left sidebar resizable within 180px–360px.
