# Proposal: volume-and-scrubber-polish

## Why

While Spoty has a persistent 72px player bar, the scrub timeline and volume sliders currently lack visual feedback present in Spotify desktop. Specifically, hovering over Spotify's progress bar reveals a floating timestamp tooltip (`mm:ss`) at the hover position, and hovering over the volume control shows the exact volume percentage. Furthermore, unmuting when volume is at zero leaves playback inaudible, and dragging the volume slider while muted does not automatically unmute.

## What Changes

1. **Floating Hover Tooltips on Sliders (`crates/ui/src/widgets/slider.rs`)**:
   - Extend `SpotifySlider` with `.tooltip_time()` and `.tooltip_percent()` modes.
   - When hovered or dragged, calculate the cursor offset position and render an elevated Spotify tooltip pill above the cursor displaying `mm:ss` (for timeline) or `N%` (for volume).
2. **Smooth Scrubbing Experience**:
   - Ensure the scrub thumb tracks the drag cursor position with subpixel smoothness and clear visual feedback.
3. **Smart Volume & Mute Restoration (`crates/audio/src/worker.rs`, `crates/ui/src/app.rs`)**:
   - Store previous non-zero volume so unmuting restores the user's prior listening level (defaulting to 50% if previous was 0).
   - Automatically clear muted state when the user interacts with the volume slider.
4. **Time Label Tabular Monospace Polish**:
   - Ensure current position and track duration timestamps render with tabular monospace formatting so numbers do not horizontally jitter during playback.

## Capabilities

### New Capabilities
- `ui/slider-tooltips`: Floating time and percentage tooltip indicators rendered on slider hover/drag.

### Modified Capabilities
- `ui/spotify-player-bar`: Interactive scrubber enhancements with tooltip previews and smart volume unmuting behavior.
- `audio/playback-engine`: Volume restoration logic ensuring unmuting always yields audible audio.

## Impact

- Affects `crates/ui/src/widgets/slider.rs`, `crates/ui/src/app.rs`, and `crates/audio/src/worker.rs`.
- Zero breaking changes to core domain models or storage providers.
