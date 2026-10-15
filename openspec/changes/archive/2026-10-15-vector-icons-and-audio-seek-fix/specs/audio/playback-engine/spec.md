# Specification: Audio Playback Engine Seeking

## MODIFIED REQUIREMENTS

### Requirement: Playback Position Seeking
The audio playback engine SHALL seek directly to the requested timeline position on command. If the sink's underlying decoder does not natively support seeking, the audio worker SHALL re-open the audio source from cached bytes and skip samples up to the target duration before appending to the sink.

#### Scenario: User seeks track to target position
- GIVEN audio is loaded and playing or paused
- WHEN a `PlayerCommand::Seek(position)` is sent to the worker
- THEN the worker SHALL update `state.position_secs` to the requested position
- AND the audio sink SHALL output audio starting from the requested timestamp without remaining at the previous position
- AND the internal timing reference `last_tick` SHALL be reset to prevent time drifting.
