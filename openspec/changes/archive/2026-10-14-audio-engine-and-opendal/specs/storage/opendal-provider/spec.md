## Purpose

Provides directory scanning, audio file indexing, metadata extraction, and stream acquisition using Apache OpenDAL.

## ADDED Requirements

### Requirement: OpenDAL Directory Catalog Scanning
The system SHALL scan the configured storage location for supported audio extensions (`.mp3`, `.flac`, `.wav`, `.ogg`, `.m4a`) and compile them into a queryable library.

#### Scenario: Scanning local audio directory
- **WHEN** `fetch_library()` is called on an OpenDAL provider initialized with a directory path
- **THEN** it discovers all audio files in that directory and returns them as `Track` models

### Requirement: Stream Resolution from OpenDAL
The system SHALL resolve a streaming byte reader for any track indexed through OpenDAL.

#### Scenario: Streaming an indexed track
- **WHEN** `stream_track()` is called with an indexed `TrackId`
- **THEN** an `AudioStream` reading raw bytes from the OpenDAL operator is returned
