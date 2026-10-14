## 1. Cargo Workspace Setup

- [x] 1.1 Create root `Cargo.toml` defining the workspace members (`spoty-core`, `spoty-storage`, `spoty-audio`, `spoty-ui`) and shared workspace dependencies. Verify with `cargo metadata`.
- [x] 1.2 Scaffold each crate directory (`crates/core`, `crates/storage`, `crates/audio`, `crates/ui`) with its `Cargo.toml` and placeholder entrypoint. Verify with `cargo check --workspace`.

## 2. Core Domain Models (`spoty-core`)

- [x] 2.1 Implement strongly-typed identifiers (`TrackId`, `AlbumId`, `ArtistId`, `PlaylistId`) with `Display`, `From`, `Serialize`, `Deserialize`, and `Hash`.
- [x] 2.2 Implement domain structs: `Track`, `Album`, `Artist`, `Playlist`, and `PlaybackState`.
- [x] 2.3 Add unit tests for serialization, format validation, and state representation. Verify with `cargo test -p spoty-core`.

## 3. Storage & Music Provider Abstraction (`spoty-storage`)

- [x] 3.1 Define the asynchronous `MusicProvider` trait in `crates/storage/src/provider.rs` with `fetch_library`, `search`, and `stream_track`.
- [x] 3.2 Implement a `DemoProvider` that serves bundled in-memory royalty-free demo tracks with embedded metadata.
- [x] 3.3 Add unit tests verifying `DemoProvider` returns expected tracks and streams valid audio bytes. Verify with `cargo test -p spoty-storage`.

## 4. Audio Engine & UI Placeholders (`spoty-audio` & `spoty-ui`)

- [x] 4.1 Scaffold `spoty-audio` with audio engine interfaces and placeholder types.
- [x] 4.2 Scaffold `spoty-ui` with basic `eframe` / `egui` app skeleton.
- [x] 4.3 Run full workspace build and test suite. Verify with `cargo test --workspace`.
