# Design: queue-drawer-panel

## Architecture & Layout

```text
+--------------------------------------------------------------------------------+
|  Top Navigation & Search                                                       |
+----------------------+----------------------------------+----------------------+
|                      |                                  |  Queue Panel         |
|  Library Sidebar     |  Central Content                 |  (SidePanel::right)  |
|  (SidePanel::left)   |  (CentralPanel)                  |  - Now Playing       |
|                      |                                  |  - Next Up [Clear]   |
|                      |                                  |  - Track rows (✕)    |
+----------------------+----------------------------------+----------------------+
|  Player Bar: [Track details] | [Playback Controls] | [Visualizer] [≡] [Volume] |
+--------------------------------------------------------------------------------+
```

## Detailed Specifications

### 1. `crates/core/src/queue.rs`
- `remove_at(&mut self, index: usize) -> Option<Track>`:
  - If `index < self.tracks.len()`, remove track.
  - Adjust `current_index`:
    - If `Some(curr)` where `index < curr`, `self.current_index = Some(curr - 1)`.
    - If `index == curr`, keeps current index (pointing to next track) or `None` if queue empty.
  - If `self.shuffle`, regenerate `shuffle_order`.
- `clear_upcoming(&mut self)`:
  - If `Some(curr) = self.current_index`, truncate `self.tracks` to `curr + 1`.
  - If `self.shuffle`, regenerate `shuffle_order`.
- `upcoming_tracks(&self) -> Vec<(usize, &Track)>`:
  - Returns upcoming items ordered by playback order:
    - If shuffle: collects indices from `shuffle_order` following the current shuffle position.
    - If sequential: iterates `self.tracks` from `current_index + 1` to `self.tracks.len()`.

### 2. Player Bar Queue Toggle (`crates/ui/src/widgets/icons.rs`)
- `pub fn queue_button(ui: &mut egui::Ui, is_open: bool) -> egui::Response`:
  - 32x32 interactive button.
  - Draws 3 horizontal vector bars with crisp 1px corner radius.
  - Color is `COLOR_ACCENT_GREEN` when open/active, `COLOR_TEXT_BASE` on hover, `COLOR_TEXT_SUBDUED` when inactive.

### 3. Queue Drawer Panel (`crates/ui/src/app.rs`)
- State in `SpotyApp`:
  - `queue_drawer_open: bool` (default: `false`).
- Rendered via `egui::SidePanel::right("queue_drawer")`:
  - Width: `280.0px` (resizable `220.0..=400.0`).
  - Dark floating card frame: `COLOR_BG_BASE`, 8px corner radius, 12px inner margin.
  - **Header:**
    - Left: "Queue" heading.
    - Right: Close button (`✕`) and "Clear Queue" button (disabled if no upcoming tracks).
  - **Now Playing Section:**
    - Small cover art (40×40), bold title, artist, audio note icon (`♫`).
  - **Next In Queue Section:**
    - Vertically scrollable list of upcoming tracks.
    - Each row shows:
      - Cover thumbnail (32×32) or play icon on hover.
      - Title & Artist.
      - Duration.
      - Remove button (`✕`) on hover.
    - Clicking the row immediately jumps playback to that track (`self.play_track(track)`).
