# storage/sqlite-cache-and-favorites Specification

## Purpose
Provides embedded SQLite database persistence for caching track metadata, managing user custom playlists, and toggling favorited tracks.

## Requirements

### Requirement: Embedded SQLite Library Caching
The system SHALL persist scanned tracks to an embedded SQLite database and load cached tracks on application launch.

#### Scenario: Instant cached startup
- **WHEN** the application starts up
- **THEN** it immediately reads previously cached tracks from SQLite without waiting for storage scanning

### Requirement: Track Favorites Persistence
The system SHALL allow users to toggle favorite status on any track and persist the favorited set across sessions.

#### Scenario: Favoriting a track
- **WHEN** the user toggles the heart button on a track
- **THEN** the track ID is added to or removed from the favorites table and reflected immediately in the UI

### Requirement: User Custom Playlists
The system SHALL allow users to create named custom playlists and persist their track membership in SQLite.

#### Scenario: Creating a new playlist
- **WHEN** user clicks create playlist and provides a title
- **THEN** a new playlist record is created in SQLite and displayed in the library sidebar
