# audio/queue-management Specification

## ADDED Requirements

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
