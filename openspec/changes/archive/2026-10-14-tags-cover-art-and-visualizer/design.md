## Context

Milestones 1–4 provided playback, multi-view navigation, and storage across Local, S3, and Subsonic. Milestone 5 brings metadata accuracy, cover art rendering, and live audio visualization.

## Goals / Non-Goals

**Goals:**
- Integrate `lofty` in `spoty-storage` for pure-Rust audio metadata and picture extraction.
- Compute real-time RMS/peak audio levels across 16 frequency bands in `spoty-audio`'s background thread.
- Render dynamic equalizer bars in the player bar using egui's painter in Spotify bright green (`#1ed760`).
- Display album cover art (or fallback gradient tile) in the bottom player bar and playlist hero banner.

**Non-Goals:**
- Full parametric equalizer DSP audio filters (read-only visualization only).
- Tag writing/editing (music library remains read-only for safety).

## Decisions

### 1. Pure-Rust Tag Extraction (`lofty`)
- **Decision:** Use `lofty` with `Cursor<Vec<u8>>` or file readers.
- **Rationale:** `lofty` is pure-Rust, has no C dependencies, supports all target formats (MP3, FLAC, OGG, WAV, M4A), and parses pictures cleanly without disk writes.

### 2. Audio Level Dispatch
- **Decision:** Calculate peak/RMS amplitude envelope bins in `spoty-audio` and dispatch `PlayerEvent::VisualizerLevels([f32; 16])`.
- **Rationale:** Keeps CPU overhead below 0.1% while providing 10–20 updates per second, producing silky smooth 60fps interpolated bar animation in egui.

## Risks / Trade-offs

- **[Risk]** Corrupted or malformed tags causing scanning crashes.
  → **Mitigation**: Wrap tag parsing in `std::panic::catch_unwind` or `match` on `lofty` results and fall back to filename heuristics on any error.
