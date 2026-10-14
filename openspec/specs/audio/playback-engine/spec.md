# audio/playback-engine Specification

## Purpose
Manages the background audio thread, device sink output, stream decoding, timeline progression, and playback commands.

## Requirements

### Requirement: Non-blocking Playback Control
The audio subsystem SHALL accept playback commands (`Play`, `Pause`, `TogglePlayPause`, `Seek`, `SetVolume`, `LoadStream`) from the main/UI thread without blocking the calling thread.

#### Scenario: Dispatching play command
- **WHEN** the caller invokes `play()` on the player controller
- **THEN** the command is placed in the background worker channel and execution returns immediately

### Requirement: Real-time Timeline Progression
The playback engine SHALL track elapsed playback time and dispatch updated state events to listeners at regular intervals during active playback.

#### Scenario: Timeline position tracking
- **WHEN** audio is playing
- **THEN** elapsed position increases monotonically according to real elapsed time

### Requirement: Track End Detection
The playback engine SHALL detect when the active audio stream reaches its end and emit a `TrackEnded` event.

#### Scenario: Track finishes playing
- **WHEN** the audio sink drains all samples of the current track
- **THEN** the engine emits `PlayerEvent::TrackEnded` and enters the `Stopped` state
