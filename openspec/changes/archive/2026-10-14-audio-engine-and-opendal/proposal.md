## Why

To transform the initial skeleton into a functioning music player, we need physical audio device output via `rodio`/`cpal` in a dedicated background thread, as well as an OpenDAL storage provider that can scan directories and stream real local or cloud audio files.

## What Changes

- Implement a non-blocking `PlayerController` in `spoty-audio` that communicates with a background audio worker thread via crossbeam/tokio channels.
- Integrate `rodio::OutputStream` and `rodio::Sink` for low-latency playback, seek, volume, and automatic track-end detection.
- Add `opendal` dependency and implement `OpenDalProvider` in `spoty-storage` supporting local filesystem scanning with metadata extraction.
- Expose thread-safe playback state and progress ticker for real-time UI synchronization.

## Capabilities

### New Capabilities
- `audio/playback-engine`: Real-time audio stream playback, background sink management, volume/seek control, and progress event dispatch.
- `storage/opendal-provider`: Generic storage backend implementation using Apache OpenDAL capable of recursive scanning, metadata extraction, and streaming.

### Modified Capabilities
<!-- None: additive implementation -->

## Impact

- Adds `opendal` and `lofty` (metadata tag parser) dependencies to `spoty-storage`.
- Extends `spoty-audio` with real background hardware audio playback.
- Enables the UI to trigger real sound output and receive accurate timeline positions.
