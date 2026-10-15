# Design: volume-and-scrubber-polish

## Architecture & Interaction

```text
Cursor Hovers / Drags on Timeline
                 │
                 ▼
     [SpotifySlider::show]
                 │
                 ├─ Calculate normalized cursor position: (mouse.x - rect.left) / rect.width
                 ├─ Value preview = norm * max
                 │
                 ├─ If SliderTooltip::Time:
                 │    Format: "m:ss" (e.g. "2:45")
                 │
                 ├─ If SliderTooltip::Percent:
                 │    Format: "N%" (e.g. "80%")
                 │
                 ▼
     Render floating tooltip pill:
     - Center X: mouse.x (clamped within track bounds)
     - Center Y: rect.top() - 14.0px
     - Background: #282828 (Elevated Spotify dark)
     - Corner radius: 4px
     - Text: 11px bold white
```

## Detailed Component Specifications

### 1. `crates/ui/src/widgets/slider.rs`
- Add `SliderTooltip` enum:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum SliderTooltip {
      Time,
      Percent,
  }
  ```
- Builder methods:
  - `pub fn tooltip_time(mut self) -> Self`
  - `pub fn tooltip_percent(mut self) -> Self`
- When `is_hovered`:
  - Calculate `hover_x`: `mouse_pos.x.clamp(rect.left(), rect.right())`.
  - Calculate `hover_val`: `((hover_x - rect.left()) / rect.width()) as f64 * self.max`.
  - Format string according to `SliderTooltip`.
  - Paint a floating badge:
    - Text size: 10.5pt monospace.
    - Measure text bounds, pad with `[6.0, 3.0]`.
    - Bubble rect: centered at `hover_x`, top at `rect.top() - 20.0`.
    - Fill: `Color32::from_rgb(0x28, 0x28, 0x28)`, stroke: `1.0px` `#383838`, corner radius: `4px`.
    - Text color: `Color32::WHITE`.

### 2. Audio Engine / Worker Volume Restoration (`crates/audio/src/worker.rs`)
- Store `previous_volume: f32` in `AudioWorker`.
- When `PlayerCommand::ToggleMute`:
  - If `!self.state.is_muted` (muting):
    - `self.previous_volume = self.state.volume;`
    - `self.state.is_muted = true;`
    - `sink.set_volume(0.0);`
  - If `self.state.is_muted` (unmuting):
    - `self.state.is_muted = false;`
    - `let restore_vol = if self.previous_volume > 0.05 { self.previous_volume } else { 0.5 };`
    - `self.state.volume = restore_vol;`
    - `sink.set_volume(restore_vol);`
- When `PlayerCommand::SetVolume(vol)`:
  - If `self.state.is_muted` and `vol > 0.0`:
    - Automatically unmute (`self.state.is_muted = false;`).

### 3. Player Bar Integration (`crates/ui/src/app.rs`)
- Timeline slider uses `.tooltip_time()`.
- Volume slider uses `.tooltip_percent()`.
