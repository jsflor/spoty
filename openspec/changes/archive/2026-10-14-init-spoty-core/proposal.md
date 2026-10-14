## Why

Users who want a self-hosted, lightweight streaming music player need a flexible open-source solution that decouples music storage (Local filesystem, S3, R2, Dropbox, Google Drive, Raspberry Pi home servers) from the player interface while matching Spotify's desktop UI and UX. This initial milestone establishes the Cargo workspace architecture, foundational domain models, and pluggable storage provider abstraction.

## What Changes

- Initialize a modular Cargo workspace with `crates/core`, `crates/storage`, `crates/audio`, and `crates/ui`.
- Define core domain models in `crates/core` for `Track`, `Album`, `Artist`, `Playlist`, and `PlaybackState`.
- Implement a pluggable `MusicProvider` async trait in `crates/storage` to support multiple backends.
- Set up a demo asset bundler with royalty-free tracks and album artwork for out-of-the-box local testing.

## Capabilities

### New Capabilities
- `core/domain-model`: Canonical domain representations for music entities, metadata parsing, and playback state models.
- `storage/provider-abstraction`: The asynchronous `MusicProvider` trait defining catalog discovery, metadata retrieval, and audio stream resolution across self-hosted and third-party backends.

### Modified Capabilities
<!-- None: greenfield project -->

## Impact

- Creates the root `Cargo.toml` and crate structure.
- Establishes zero-copy and serializable domain interfaces shared across all subsystems.
- Sets up test audio assets for local development and CI verification without requiring external cloud accounts.
