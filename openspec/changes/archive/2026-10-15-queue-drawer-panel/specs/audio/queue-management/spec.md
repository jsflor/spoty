# audio/queue-management Specification

## ADDED Requirements

### Requirement: Queue Removal and Truncation
The `Queue` model SHALL support removing items by index and clearing all items after the active track index.

#### Scenario: Removing an upcoming track
- **GIVEN** a queue of 3 tracks
- **WHEN** `remove_at(1)` is invoked
- **THEN** track 1 is removed from the queue, leaving 2 tracks

#### Scenario: Clearing upcoming queue items
- **GIVEN** a queue with current track at index 0 and 5 subsequent tracks
- **WHEN** `clear_upcoming()` is invoked
- **THEN** only track 0 remains in the queue
