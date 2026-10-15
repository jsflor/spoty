# Proposal: audio-sink-adapter-seam

## Why

In `crates/audio`, playback logic and hardware control suffer from state duplication and an absent hardware seam:
1. **Dead Orphaned Module:** `AudioEngine` in `src/engine.rs` (105 lines) was written as a pure playback state machine during initial setup, but neither `AudioWorker` nor `PlayerController` ever invokes it. It exists purely to satisfy its own unit test, while `AudioWorker` reimplements an identical 200-line state machine inside its thread loop.
2. **Coupled Hardware & Thread Scheduling:** `AudioWorker` hardcodes direct calls to `rodio::OutputStream::try_default()` and `rodio::Sink::try_new()`. Thread scheduling, command channel polling, symphonia/rodio sample decoding, volume scaling, and state transitions are entangled in a monolithic 320-line loop.
3. **Flaky & Slow Tests Requiring Thread Sleeps:** Because there is no adapter seam between audio logic and the physical OS audio sink, testing `PlayerController` requires spawning real OS worker threads and sleeping for 150ms–300ms in wall-clock time (`controller_test.rs`, `volume_mute_test.rs`). In headless environments (CI runners, headless containers), tests rely on warning fallbacks or fail if audio drivers are unavailable.

## What Changes

1. **`AudioSink` Trait Seam (`crates/audio/src/sink.rs`)**:
   - Establish a clean seam representing the audio hardware output sink (`play`, `pause`, `stop`, `set_volume`, `is_empty`, `append_source`).
2. **Two Justifying Adapters**:
   - `RodioSink`: The production adapter managing live `rodio::OutputStream` and `rodio::Sink` devices.
   - `MockSink`: A deterministic, in-memory adapter that records commands and sample metrics without requiring real audio hardware or thread delays.
3. **Reclaim `AudioEngine` as the Deterministic Core**:
   - Consolidate state transitions, volume curves, mute toggles, and decoding into `AudioEngine`, driving the `AudioSink` trait through the seam.
   - Delete the duplicate state machine code from `AudioWorker`.
4. **Instant, Deterministic Tests**:
   - Rewrite audio unit and lifecycle tests to test `AudioEngine` directly against `MockSink`, eliminating all `thread::sleep` delays.

## Capabilities

### New Capabilities
- `audio/sink-abstraction`: Formal hardware abstraction seam separating deterministic playback state transitions from physical audio output devices.

### Modified Capabilities
- `audio/playback-engine`: Replaces monolithic hardware coupling with the `AudioSink` seam and consolidates state transitions into `AudioEngine`.

## Impact

- **Crates Affected:**
  - `crates/audio`: Introduces `src/sink.rs`, refactors `src/engine.rs` and `src/worker.rs`, and rewrites tests in `tests/`.
  - `crates/ui` and `crates/core`: Unaffected. Public `PlayerController` interface remains compatible.
- **Test Performance:** Replaces multi-second sleep-based audio tests with sub-millisecond deterministic assertions.
