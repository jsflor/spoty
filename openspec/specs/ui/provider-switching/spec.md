# ui/provider-switching Specification

## Purpose
Specifies the behavior of storage provider transitions and catalog synchronization within the Spoty user interface.

## Requirements

### Requirement: Provider Switching Playback Reset
When the active music provider is changed, the UI SHALL pause current playback, reset the session queue, and update the audio stream resolver to match the new provider.

#### Scenario: Switching music providers during playback
- **GIVEN** an active playback session streaming audio from Provider A
- **WHEN** user switches to Provider B
- **THEN** playback is paused
- **AND** the queue is cleared
- **AND** the audio stream resolver is updated to Provider B

### Requirement: Asynchronous Queue Synchronization
Upon successful synchronization of a newly selected provider's library into the catalog, the UI SHALL update the playback session queue with the catalog's tracks.

#### Scenario: Background sync completes
- **GIVEN** a newly mounted provider undergoing asynchronous catalog sync
- **WHEN** `sync_provider` finishes successfully
- **THEN** the playback session queue is populated with the new provider's tracks

### Requirement: Out-of-Queue Track Playback
When a user selects a track for playback that is not present in the active playback session queue, the UI SHALL populate the queue with library tracks starting at the selected track and initiate playback.

#### Scenario: User clicks a track not currently in the queue
- **GIVEN** an active session queue lacking `Track X`
- **WHEN** the user selects `Track X` to play
- **THEN** the session queue is populated with catalog tracks starting at `Track X`
- **AND** playback of `Track X` begins immediately
