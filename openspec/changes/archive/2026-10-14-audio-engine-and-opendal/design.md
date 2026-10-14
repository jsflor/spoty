## Context

In Milestone 1, we defined `AudioEngine` state models and the `MusicProvider` trait. In Milestone 2, we implement the real audio pipeline and the OpenDAL storage integration.

## Goals / Non-Goals

**Goals:**
- Dedicated `rodio::OutputStream` worker thread running an event loop decoupled from UI framerate.
- Thread-safe `PlayerController` handle providing non-blocking calls and an event receiver.
- `OpenDalProvider` using Apache OpenDAL (`services::Fs` with readiness for `services::S3`) to list audio files and stream bytes.
- Lightweight audio tag extraction (fallback to filename when tags are missing).

**Non-Goals:**
- UI slider rendering (this will consume the controller in Milestone 3).
- Remote Spotify Connect protocol handshake (deferred to future milestone).

## Decisions

### 1. Dedicated OS Audio Thread vs Tokio Task
- **Decision:** Run the audio output sink on a dedicated `std::thread` rather than an async Tokio worker.
- **Rationale:** Audio output requires real-time guarantees. Mixing audio sink buffers on a shared async runtime thread pool can lead to audio stutter when disk I/O or network tasks saturate workers.
- **Alternatives Considered:** Tokio blocking task (rejected: unpredictable scheduling jitter).

### 2. Message Passing Protocol
- **Decision:** Use `crossbeam_channel` (or `std::sync::mpsc`) for command sending (`PlayerCommand`) and progress event broadcasting.
- **Rationale:** Simple, zero-allocation message queues with bounded latency.

### 3. OpenDAL Integration
- **Decision:** Use OpenDAL `Operator` with `services::Fs` as primary backend.
- **Rationale:** OpenDAL abstractly represents files as entries with `reader().await`, making it trivial to point to S3 or R2 later by changing the builder configuration.

## Risks / Trade-offs

- **[Risk]** Headless CI environments having no audio hardware (`NoDevice` error from CPAL).
  → **Mitigation**: Gracefully handle missing audio device in tests or fallback to a dummy/null sink when audio hardware is unavailable.
- **[Risk]** OpenDAL pulling large async dependencies.
  → **Mitigation**: Enable only necessary OpenDAL features (`services-fs`).
