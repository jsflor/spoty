## 1. Real Audio Output Worker (`spoty-audio`)

- [x] 1.1 Implement `AudioWorker` running on a dedicated thread with `rodio::OutputStream` and `rodio::Sink`.
- [x] 1.2 Implement stream decoding using `rodio::Decoder` with cursor/buffered audio bytes.
- [x] 1.3 Implement `PlayerController` handle with non-blocking command dispatch and event subscription.
- [x] 1.4 Add unit tests for player controller commands and state reporting. Verify with `cargo test -p spoty-audio`.

## 2. OpenDAL Storage Provider (`spoty-storage`)

- [x] 2.1 Add `opendal` dependency to `spoty-storage` and workspace `Cargo.toml`.
- [x] 2.2 Implement `OpenDalProvider` implementing `MusicProvider` with directory scanning and stream resolution.
- [x] 2.3 Implement audio file metadata extraction (filename heuristics and tag parsing).
- [x] 2.4 Add unit tests verifying `OpenDalProvider` scans a temporary directory with test audio files and streams them. Verify with `cargo test -p spoty-storage`.

## 3. Integration & Workspace Verification

- [x] 3.1 Wire `PlayerController` into `spoty-ui` so clicking play on demo tracks triggers real audio playback through the audio engine.
- [x] 3.2 Run full workspace tests and build verification with `cargo test --workspace`.
