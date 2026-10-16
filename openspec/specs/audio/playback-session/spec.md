# audio/playback-session Specification

## Purpose
Defines the unified playback session subsystem that coordinates audio stream ingestion, hardware sink control, and queue state progression behind a single non-blocking interface.

## Requirements

### Requirement: Unified Session Commands
The playback session SHALL expose a single non-blocking interface accepting commands for transport control (`Play`, `Pause`, `TogglePlayPause`, `Seek`, `SetVolume`, `ToggleMute`) and queue manipulation (`PlayTrack`, `PlayAll`, `Next`, `Prev`, `ToggleShuffle`, `CycleRepeatMode`, `EnqueueNext`).

#### Scenario: Dispatching transport command
- **GIVEN** an active playback session
- **WHEN** caller dispatches `session.toggle_play_pause()`
- **THEN** command is accepted non-blockingly without halting the caller
- **AND** the internal audio sink transitions between playing and paused states

### Requirement: Autonomous Queue Progression
The playback session SHALL autonomously advance to the subsequent track in the queue when the currently playing track reaches EOF, without requiring external manual event dispatch from the UI.

#### Scenario: Track finishes in sequential mode
- **GIVEN** an active queue with tracks `[Track A, Track B]` and repeat mode `All`
- **WHEN** `Track A` finishes playing
- **THEN** the session coordinator automatically starts streaming `Track B`
- **AND** emits a `SessionState` event reflecting `Track B` as current

#### Scenario: Track finishes with Repeat One enabled
- **GIVEN** an active track with repeat mode `One`
- **WHEN** the track finishes playing
- **THEN** the session restarts the same track from position 0.0 without changing the active queue index

### Requirement: Memory-Bounded Streaming
The playback session SHALL decode and stream audio tracks without loading the complete audio file into memory as a single contiguous buffer, maintaining resident memory usage below 50MB.

#### Scenario: Streaming high-resolution audio
- **WHEN** a track of 100MB duration is loaded
- **THEN** the audio worker decodes chunks incrementally
- **AND** application RAM consumption remains below 50MB

### Requirement: Guarded Track Selection
When `PlayTrack(TrackId)` is received, the playback session SHALL only initiate playback if the specified `TrackId` is found in the current queue. If the track is not present in the queue, the session SHALL NOT load or stream any leftover current track.

#### Scenario: Requesting playback of a track not in queue
- **GIVEN** an active playback session with queue `[Track A, Track B]`
- **WHEN** caller dispatches `session.play_track(Track C)`
- **THEN** the session does not load or stream `Track A` or any other track
- **AND** a warning is logged without altering playback state
