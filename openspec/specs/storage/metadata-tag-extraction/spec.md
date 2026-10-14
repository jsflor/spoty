# storage/metadata-tag-extraction Specification

## Purpose
Extracts audio tag metadata and embedded album cover art pictures from audio file bytes using pure-Rust parsers.

## Requirements

### Requirement: Embedded Audio Tag Parsing
The system SHALL parse ID3, Vorbis, and FLAC tags from audio files, extracting title, artist, album name, and duration, falling back gracefully to filename heuristics if tags are absent.

#### Scenario: Extracting tags from audio file
- **WHEN** an audio file with ID3 or Vorbis tags is scanned
- **THEN** the system populates the track's title, artist, album, and duration from the embedded tags

### Requirement: Embedded Cover Art Extraction
The system SHALL extract embedded front cover art picture bytes (`APIC` / `METADATA_BLOCK_PICTURE`) when present in audio tags.

#### Scenario: Audio file containing album art
- **WHEN** an audio track containing an embedded picture is scanned
- **THEN** its cover art bytes are extracted and made available for UI rendering
