# Proposal: playback-session-pipeline

## Why

In the current architecture, the desktop UI module (`crates/ui/src/app.rs`) acts as an ad-hoc audio coordinator rather than a pure presentation shell. When a track plays, the UI thread spawns a Tokio task to download the entire audio file into an uncompressed in-memory byte buffer (`Vec<u8>`), clones those bytes into `PlayerController.load_track`, and manages queue transitions manually:
1. **Memory Budget Violation:** Buffering entire audio files in RAM directly breaches Spoty's strict `<50MB RAM` invariant when streaming high-bitrate FLAC or WAV files.
2. **State Duplication & Synchronization Leaks:** Playback status, shuffle mode, and repeat settings are duplicated across `spoty-core` (`Queue`), `spoty-audio` (`AudioWorker` and `cached_state`), and `spoty-ui`. The UI must manually sync shuffle order between `Queue` and `PlayerController`.
3. **Porous Seam:** When a track finishes, `AudioWorker` emits `PlayerEvent::TrackEnded`, which the UI catches, inspects repeat mode from cached state, calls `queue.next()`, and re-triggers the Tokio download sequence. This creates a distributed, fragile state machine where playback sequencing is untestable without running the full GUI app.

## What Changes

1. **Deep Playback Session Module (`crates/audio/src/session.rs` / `PlaybackSession`)**:
   - Create a deep `PlaybackSession` module coordinating queue progression, audio streaming, and sink control behind a single non-blocking command interface (`play`, `pause`, `toggle_play_pause`, `seek`, `next`, `prev`, `load_queue`, `set_shuffle`, `cycle_repeat`).
   - The session encapsulates track stream acquisition by accepting an abstract stream provider reference, feeding stream readers directly to the audio decoder without buffering complete files into UI memory.
2. **Automated Track Progression Behind the Seam**:
   - When a track reaches EOF, track advancement, repeat-one looping, and shuffle traversal occur internally within the session module.
   - The UI receives consolidated `SessionEvent::StateChanged` events without acting as the queue scheduler.
3. **Elimination of Duplicated State**:
   - Consolidate playback status, timeline position, queue indices, shuffle flag, and repeat mode into an authoritative `SessionState` snapshot.
4. **Shell Simplification (`crates/ui/src/app.rs`)**:
   - Replace manual Tokio byte-buffering tasks and queue-audio synchronization logic in `SpotyApp` with direct calls to `PlaybackSession`.

## Capabilities

### New Capabilities
- `audio/playback-session`: Unified queue and playback management orchestrating track progression, stream ingestion, and timeline tracking behind a single interface.

### Modified Capabilities
- `audio/playback-engine`: Offloads track transition and queue coordination logic to the session module; decodes audio streams directly from readers rather than holding static byte buffers.
- `audio/queue-management`: Integrates queue state directly into the unified session state machine.

## Impact

- **Crates Affected:**
  - `crates/audio`: Introduces `session.rs` and refactors `PlayerController` / `AudioWorker` to work with streaming readers.
  - `crates/core`: `Queue` remains a pure deterministic domain struct but is managed directly within `PlaybackSession`.
  - `crates/ui`: Removes over 200 lines of manual streaming and synchronization code in `crates/ui/src/app.rs`.
- **Breaking Changes:** Replaces `PlayerController::load_track(track, Vec<u8>)` with streaming session commands. Existing integration tests in `crates/ui/tests/` will be updated to interact with `PlaybackSession`.
