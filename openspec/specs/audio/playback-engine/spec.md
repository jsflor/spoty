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

### Requirement: Volume Restoration on Unmute
The audio engine SHALL record the previous non-zero volume level when muting, and restore that level (or a minimum default of 50%) when unmuted.

#### Scenario: Unmuting restored volume
- **GIVEN** playback volume is set to 80%
- **WHEN** user toggles mute twice
- **THEN** volume is restored to 80% and audio resumes at that level

### Requirement: Auto-Unmute on Volume Slider Interaction
Setting a non-zero volume on a muted player SHALL automatically clear the muted state.

#### Scenario: Dragging volume while muted
- **GIVEN** player is currently muted
- **WHEN** user sets volume to 40%
- **THEN** mute state is set to false and audio begins playing at 40%

### Requirement: Playback Position Seeking
The audio playback engine SHALL seek directly to the requested timeline position on command. If the sink's underlying decoder does not natively support seeking, the audio worker SHALL re-open the audio source from cached bytes and skip samples up to the target duration before appending to the sink.

#### Scenario: User seeks track to target position
- **GIVEN** audio is loaded and playing or paused
- **WHEN** a `PlayerCommand::Seek(position)` is sent to the worker
- **THEN** the worker SHALL update `state.position_secs` to the requested position
- **AND** the audio sink SHALL output audio starting from the requested timestamp without remaining at the previous position
- **AND** the internal timing reference `last_tick` SHALL be reset to prevent time drifting

### Requirement: Audio Sink Hardware Seam
The playback engine SHALL decouple playback state transitions and stream decoding from OS audio hardware by delegating output operations to an `AudioSink` seam.

#### Scenario: Delegating playback commands to sink
- **GIVEN** an active `AudioEngine` configured with an `AudioSink`
- **WHEN** commands (`Play`, `Pause`, `Stop`, `SetVolume`) are processed
- **THEN** corresponding methods on the `AudioSink` are invoked and playback state is updated deterministically

### Requirement: Perceptual Decibel Gain Scaling
The audio engine SHALL convert linear volume slider inputs into a logarithmic/perceptual acoustic gain curve ($G = V^3$) to provide smooth loudness adjustments matching human ear perception.

#### Scenario: Setting 50% volume
- **WHEN** user sets the volume slider to 50%
- **THEN** audio output power is attenuated logarithmically by approximately $-18\text{ dB}$ (gain: 0.125)

### Requirement: Track Crossfade Transitions
When crossfading is configured ($N > 0$ seconds), the audio engine SHALL begin playback of the subsequent track $N$ seconds prior to the end of the current track, fading the outgoing track to zero while fading the incoming track to full volume.

#### Scenario: Smooth transition between consecutive tracks
- **GIVEN** crossfade is set to 4 seconds
- **WHEN** the active track has 4 seconds remaining
- **THEN** the next track begins playing quietly, ramping up volume as the active track fades out
