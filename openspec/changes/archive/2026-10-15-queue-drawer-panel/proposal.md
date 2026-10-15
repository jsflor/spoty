# Proposal: queue-drawer-panel

## Why

Spoty maintains an active playback queue in `spoty-core` (`Queue`), but provides no visual interface to view upcoming tracks, remove unwanted songs from the queue, or clear the upcoming tracklist. Spotify desktop features a persistent queue toggle button (`≡`) in the bottom player bar that opens a slide-out "Now Playing" and "Next Up" panel. Adding this capability allows users to inspect their queue, skip directly to upcoming songs, and manage upcoming playback in real time.

## What Changes

1. **Queue Core Primitives (`crates/core/src/queue.rs`)**:
   - Add `remove_at(&mut self, index: usize) -> Option<Track>` to remove a specific queued song.
   - Add `clear_upcoming(&mut self)` to remove all upcoming tracks while keeping the active track playing.
   - Add `upcoming_tracks(&self) -> Vec<(usize, &Track)>` supporting both sequential and shuffle queue orders.
2. **Player Bar Queue Toggle (`crates/ui/src/widgets/icons.rs`, `crates/ui/src/app.rs`)**:
   - Add a vector Queue icon button (`≡`) to the right controls section of the 72px bottom player bar.
   - Highlights in Spotify Green (`#1ed760`) when the drawer is open.
3. **Queue Drawer Panel (`crates/ui/src/app.rs`)**:
   - Implement an elevated right side panel (`egui::SidePanel::right`) using "Cards on Canvas" aesthetics.
   - Display "Now Playing" section with current track details and cover artwork.
   - Display "Next in Queue" section with upcoming tracks, track numbers/thumbnails, duration, click-to-jump playback, and per-track `✕` removal buttons.
   - Add a "Clear Queue" action in the header to purge upcoming tracks.

## Capabilities

### New Capabilities
- `ui/queue-drawer`: Slide-out queue drawer toggled from the player bar, showing Now Playing track details, upcoming tracks with remove and clear actions, and click-to-play jumping.

### Modified Capabilities
- `audio/queue-management`: Extended with queue inspection, track removal, and upcoming track truncation.

## Impact

- Affects `crates/core/src/queue.rs` and its unit tests.
- Affects `crates/ui/src/widgets/icons.rs`, `crates/ui/src/widgets/mod.rs`, and `crates/ui/src/app.rs`.
- Zero breaking changes to storage providers or audio engine workers.
