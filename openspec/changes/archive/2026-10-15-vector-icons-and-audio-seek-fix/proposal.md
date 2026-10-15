# Proposal: Vector Icons and Audio Seek Engine Fix

## Why
1. Several critical icons across the Spoty UI (Create Playlist `＋`, Home `⌂`, Sort Asc/Desc `▲`/`▼`, Shuffle `⇄`, Close Queue / Remove Track `✕`) fail to render or appear as blank glyphs because the embedded egui default fonts do not include extended Unicode characters.
2. Clicking or dragging on the Spotify player progress bar does not seek or advance the music. In `crates/audio/src/worker.rs`, `rodio::Sink::try_seek` is called on the audio decoder, but rodio's decoders for OGG Vorbis, MP3, and FLAC do not support `try_seek` and return `SeekError::NotSupported`. The error is silently discarded, leaving the audio sink playing at its original position without advancing. In addition, the slider interaction model drops clicks that lack a drag delta.

## What Changes
- Replace font-dependent Unicode characters with crisp, resolution-independent vector icons rendered via egui's `Painter` primitives:
  - Vector Home icon (`draw_home_icon`)
  - Vector Plus icon (`draw_plus_icon`)
  - Vector Shuffle crossed-arrows icon in `shuffle_button`
  - Vector Close / Remove 'X' icon (`draw_close_icon`)
  - Vector Sort Ascending / Descending arrows in `crates/ui/src/views/songs.rs`
- Fix audio seeking in `crates/audio/src/worker.rs`:
  - When `sink.try_seek` returns `Err(_)`, fall back to re-instantiating the decoder from cached audio bytes (`current_bytes`) and calling `decoder.skip_duration(target_duration)` before appending to the sink.
  - Reset `last_tick` on seek to ensure playback elapsed timer is accurately synchronized.
- Enhance `SpotifySlider` interaction and progress bar state handling:
  - Robust pointer coordinate resolution supporting press-down, click release, and drag.
  - Prevent slider thumb fighting with worker position ticks while scrubbing.

## Capabilities

### New / Modified Capabilities
- `ui/vector-icons`: Standalone vector rendering functions for UI action icons that render cleanly across all display densities and font configurations.
- `audio/playback-engine`: Reliable seeking mechanism across compressed audio formats (Ogg, MP3, FLAC) via sample stream skipping fallback.
- `ui/spotify-player-bar`: Interactive progress scrubbing and click-to-seek functionality.

## Impact
- `crates/ui`: Fixes missing visual elements in TopBar, Library Sidebar, Queue Drawer, Songs View, and Player Bar.
- `crates/audio`: Enables responsive audio timeline scrubbing and seeking without UI or thread blocking.
