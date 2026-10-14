## 1. Metadata Tag & Picture Extraction (`spoty-storage`)

- [x] 1.1 Add `lofty` dependency to `Cargo.toml` and `spoty-storage/Cargo.toml`.
- [x] 1.2 Implement `extract_audio_metadata(&[u8], &str)` in `crates/storage/src/metadata.rs` extracting title, artist, album, duration, and picture data using `lofty`.
- [x] 1.3 Update `OpenDalProvider` to use `extract_audio_metadata` during directory scanning.
- [x] 1.4 Add unit tests verifying tag extraction and fallback heuristics on test audio bytes. Verify with `cargo test -p spoty-storage`.

## 2. Audio Visualizer Level Computation (`spoty-audio`)

- [x] 2.1 Add `PlayerEvent::VisualizerLevels([f32; 16])` to `spoty-audio::engine`.
- [x] 2.2 Implement real-time amplitude/frequency bin calculation in `spoty-audio::worker` during active playback loop.
- [x] 2.3 Add unit tests verifying visualizer level generation and event dispatch. Verify with `cargo test -p spoty-audio`.

## 3. UI Visualizer & Cover Art Rendering (`spoty-ui`)

- [x] 3.1 Implement `VisualizerWidget` in `crates/ui/src/widgets/visualizer.rs` rendering animated spectrum bars in Spotify green (`#1ed760`).
- [x] 3.2 Add the visualizer widget to the bottom persistent player bar next to the volume slider.
- [x] 3.3 Render album cover art thumbnails in the bottom player bar and playlist cards.
- [x] 3.4 Run full workspace test suite and build verification with `cargo test --workspace`.
