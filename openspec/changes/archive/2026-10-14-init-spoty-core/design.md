## Context

See `proposal.md` for motivation. We are bootstrapping a multi-crate Rust workspace to deliver a self-hosted Spotify desktop client using `egui`.

## Goals / Non-Goals

**Goals:**
- Establish a workspace structure (`core`, `storage`, `audio`, `ui`) with shared dependencies and unified compile targets.
- Implement zero-dependency core domain types that can be shared between headless servers and GUI clients.
- Define a generic `MusicProvider` trait that decouples catalog management and streaming from underlying storage technologies.
- Include a working local/embedded demo provider with playable audio files to verify compilation and testing immediately.

**Non-Goals:**
- Full UI presentation implementation (deferred to Milestone 3).
- Deep cloud integrations (OpenDAL S3/R2/Dropbox connection logic deferred to Milestone 4).
- Spotify DRM circumvention or proprietary media decryption (explicitly out of scope).

## Decisions

### 1. Cargo Workspace Architecture
- **Decision:** Split into `spoty-core`, `spoty-storage`, `spoty-audio`, and `spoty-ui`.
- **Rationale:** Keeps domain models independent of heavy GUI (`eframe`, `egui`) and audio hardware libraries (`cpal`), enabling reuse in headless or CLI modes.
- **Alternatives Considered:** Monolithic single crate (rejected: mixes UI and audio logic, slowing rebuilds).

### 2. Provider Abstraction via Async Traits
- **Decision:** Use `async-trait` for `MusicProvider` returning standard `AsyncRead` byte streams.
- **Rationale:** Providers like OpenDAL, Subsonic, and local disk are inherently asynchronous. Returning an `AsyncRead` stream decouples byte delivery from audio decoding.
- **Alternatives Considered:** Synchronous `std::io::Read` (rejected: blocks worker threads on network latency).

### 3. Serialization and Identifiers
- **Decision:** Use `serde` for all models, with strong ID wrapper types (`TrackId`, `AlbumId`, `PlaylistId`).
- **Rationale:** Prevents identifier confusion and simplifies saving/restoring local playlist caches.

## Risks / Trade-offs

- **[Risk]** Heavy audio dependencies across crates.
  → **Mitigation**: Confine `symphonia` and `rodio` strictly to `spoty-audio`; `core` and `storage` deal only with metadata and raw byte streams.
- **[Risk]** Large binary size with embedded demo audio files.
  → **Mitigation**: Use compact MP3/OGG royalty-free clips under 1MB for the embedded demo set.
