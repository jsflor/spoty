# ui/navigation-and-views Specification

## ADDED Requirements

### Requirement: Track Context Menu
The UI SHALL provide a context menu on track rows and cards across all views with options to add the track to custom playlists, append the track to the playback queue, and toggle its favorite status.

#### Scenario: Adding a track to a custom playlist from context menu
- **GIVEN** a track visible in Search, Songs, or Home view
- **WHEN** the user right-clicks the track and selects a playlist under "Add to Playlist"
- **THEN** the track is appended to the custom playlist in SQLite and reflected immediately in the library
