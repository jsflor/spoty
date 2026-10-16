# audio/playback-session Specification Delta

## Requirements

### Requirement: Guarded Track Selection
When `PlayTrack(TrackId)` is received, the playback session SHALL only initiate playback if the specified `TrackId` is found in the current queue. If the track is not present in the queue, the session SHALL NOT load or stream any leftover current track.

#### Scenario: Requesting playback of a track not in queue
- **GIVEN** an active playback session with queue `[Track A, Track B]`
- **WHEN** caller dispatches `session.play_track(Track C)`
- **THEN** the session does not load or stream `Track A` or any other track
- **AND** a warning is logged without altering playback state
