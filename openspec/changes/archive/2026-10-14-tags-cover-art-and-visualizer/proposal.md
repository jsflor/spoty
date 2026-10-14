## Why

To deliver a true Spotify-like experience for local and cloud music libraries, tracks need accurate metadata (tags, durations, genres) and embedded album art extracted directly from audio files, paired with a real-time animated audio visualizer in the playback bar.

## What Changes

- Add pure-Rust audio tag parsing (`lofty`) to `spoty-storage` to extract ID3v2, Vorbis Comments, and FLAC tags (title, artist, album, genre, year, exact duration, and embedded cover art bytes).
- Enhance `OpenDalProvider` to parse tags from audio files when scanning.
- Add real-time audio amplitude and frequency level computation in `spoty-audio`'s background worker, dispatched to the UI via `PlayerEvent::VisualizerLevels`.
- Render a live, responsive audio equalizer spectrum bar visualizer in the player bar using egui's painter.
- Support embedded and URI-based album cover art rendering in the bottom bar and playlist hero banners.

## Capabilities

### New Capabilities
- `storage/metadata-tag-extraction`: Tag extraction for MP3, FLAC, OGG, WAV, and M4A audio files using `lofty`, including embedded picture retrieval and exact duration parsing.
- `ui/audio-visualizer`: Real-time audio sample amplitude calculation and animated frequency bar visualizer component in `spoty-ui`.

### Modified Capabilities
<!-- None -->

## Impact

- Adds `lofty` to `spoty-storage`.
- Extends `PlayerEvent` in `spoty-audio` with spectrum/level updates.
- Upgrades track listings and player bar with real cover art and live animation.
