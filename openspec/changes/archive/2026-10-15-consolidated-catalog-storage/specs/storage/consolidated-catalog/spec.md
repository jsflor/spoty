# storage/consolidated-catalog Specification

## Purpose
Defines the consolidated catalog module that unifies in-memory catalog representations, instant favorites indexing, and SQLite database persistence behind a single cohesive interface.

## Requirements

### Requirement: Zero SQLite Queries on Render Loop
The catalog module SHALL service track favorite status queries in O(1) time from an in-memory index without opening SQLite connections or issuing SQL statements during UI rendering passes.

#### Scenario: Querying favorite status during UI frame
- **GIVEN** a catalog initialized with favorite tracks
- **WHEN** the UI calls `catalog.is_favorite(&track_id)` on 100 visible tracks in a single frame
- **THEN** all queries resolve from in-memory cache with zero disk I/O or SQLite lock contention

### Requirement: Atomic State and Disk Synchronization
The catalog module SHALL guarantee that all playlist and favorite mutations update the persistent SQLite database and the cached in-memory representations atomically.

#### Scenario: Adding track to custom playlist
- **GIVEN** an existing custom playlist in the catalog
- **WHEN** caller invokes `catalog.add_track_to_playlist(&playlist_id, &track_id)`
- **THEN** the track association is persisted to the `playlist_tracks` SQLite table
- **AND** the in-memory playlist representation immediately contains the track ID without requiring manual caller reloads

#### Scenario: Toggling track favorite
- **GIVEN** an unfavorited track ID
- **WHEN** caller invokes `catalog.toggle_favorite(&track_id)`
- **THEN** an entry is inserted into the SQLite `favorites` table
- **AND** `catalog.is_favorite(&track_id)` immediately returns `true`
- **AND** subsequent calls return `false` upon removal from both cache and disk

### Requirement: Non-blocking Provider Synchronization
The catalog module SHALL ingest and refresh library collections from external music providers asynchronously without blocking the desktop GUI rendering thread.

#### Scenario: Switching active provider
- **WHEN** user selects a new music provider
- **THEN** catalog fetches and indexes the catalog on background tasks
- **AND** UI frames continue rendering without interruption
