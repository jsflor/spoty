# audio/queue-management Specification

## Purpose
Tracks playback queues, next and previous track selection, shuffle permutation, and automatic playlist advancement.

## Requirements

### Requirement: Queue Management and Track Progression
The system SHALL maintain a list of upcoming tracks and automatically play the next track in the queue when the currently playing track finishes.

#### Scenario: Continuous playback on track completion
- **WHEN** the current track finishes and repeat is not set to One
- **THEN** the queue advances to the next track and begins streaming it

### Requirement: Next and Previous Navigation
The system SHALL support explicit next track skipping and previous track seeking or skipping based on the current position.

#### Scenario: Skipping to next track
- **WHEN** user clicks the next button
- **THEN** the active track advances to the subsequent track in the queue

### Requirement: Dynamic Queue Append and Insertion
The `Queue` model SHALL support appending tracks to the end of the upcoming playlist and inserting tracks to play immediately following the active track.

#### Scenario: Appending a track to the queue
- **GIVEN** an active queue of 3 tracks
- **WHEN** `push_back(track)` is invoked
- **THEN** the queue contains 4 tracks, and the new track is placed at the end

#### Scenario: Inserting a track to play next
- **GIVEN** an active queue playing track index 1
- **WHEN** `insert_next(track)` is invoked
- **THEN** the new track is inserted at index 2, directly following the current track
