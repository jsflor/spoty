# Design: Vector Icons and Audio Seek Engine Fix

## Technical Architecture

### 1. Vector Icon Primitives (`spoty-ui::widgets::icons`)
Rather than relying on font glyphs (which fail when unicode code points are not present in bundled fonts), we implement vector drawing helpers using `egui::Painter`:

- `draw_home_icon(painter: &Painter, center: Pos2, size: f32, color: Color32)`:
  - Renders a peaked roof triangle with `Shape::convex_polygon` and a body rectangle with subtle proportions.
- `draw_plus_icon(painter: &Painter, center: Pos2, size: f32, stroke_width: f32, color: Color32)`:
  - Renders two perpendicular rounded rectangles (`Vec2(size, stroke_width)` and `Vec2(stroke_width, size)`).
- `shuffle_button(ui: &mut Ui, is_shuffle: bool)`:
  - Renders two crossed arrows using `painter.line_segment` and small triangle arrowheads at the right ends.
- `draw_close_icon(painter: &Painter, center: Pos2, size: f32, stroke_width: f32, color: Color32)`:
  - Renders two diagonal line segments crossing at `center`.
- `draw_sort_arrow(painter: &Painter, center: Pos2, ascending: bool, size: f32, color: Color32)`:
  - Renders a compact upward or downward pointing equilateral triangle.

### 2. Audio Seek Fallback (`spoty-audio::worker`)
`rodio::Decoder` wraps underlying decoders (Symphonia, minimp3, etc.) whose `try_seek` returns `SeekError::NotSupported` for most container and codec formats.

When `PlayerCommand::Seek(pos)` is handled in `AudioWorker`:
1. Clamp target seconds: `let target_secs = pos.as_secs_f64().clamp(0.0, self.state.duration_secs);`
2. Update internal state: `self.state.position_secs = target_secs;`
3. If `sink_opt` exists:
   - Try `sink.try_seek(Duration::from_secs_f64(target_secs))`.
   - If `Err(_)`, check `current_bytes`. If present:
     - Call `sink.stop()`.
     - Create a new decoder from `Cursor::new(bytes.clone())`.
     - Skip samples via `rodio::Source::skip_duration(target_dur)`.
     - Append skipped source to sink and re-apply volume and play/pause state.
4. Reset `last_tick = Instant::now();` so elapsed tick timing restarts from the exact seek position.
5. Call `self.emit_state();` to notify the UI immediately.

### 3. Progress Slider Click & Drag Handling (`spoty-ui::widgets::slider` & `SpotyApp`)
- In `SpotifySlider::show`:
  - Detect interaction if `response.dragged()`, `response.clicked()`, or `(response.hovered() && ui.input(|i| i.pointer.primary_down()))`.
  - Resolve pointer via `response.interact_pointer_pos().or_else(|| response.hover_pos())`.
  - Recalculate normalized slider value and mark response changed.
- In `SpotyApp`:
  - When slider changes, dispatch `self.controller.seek(Duration::from_secs_f64(pos))` so the audio engine jumps immediately.
