# core/domain-model Specification

## Purpose
Defines the canonical, serializable data structures and playback state primitives for tracks, albums, playlists, and artists across the application.

## Requirements

### Requirement: Track Model Representation
The system SHALL represent audio tracks with strongly-typed identifiers, title, artist, album, duration in seconds, file format, and cover artwork location.

#### Scenario: Valid track instantiation
- **WHEN** track metadata is parsed from an audio stream or catalog
- **THEN** the track model contains title, artist name, duration, and a unique track identifier

### Requirement: Playback State Management
The system SHALL model discrete playback states including Stopped, Playing, Paused, and Buffering, alongside seek position and volume levels.

#### Scenario: Playback state transition
- **WHEN** playback is paused
- **THEN** state records paused status and preserves the current timeline offset in seconds
