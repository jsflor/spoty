# storage/playlist-crud-operations Specification

## Purpose
Extends the SQLite persistence layer with full playlist lifecycle management: deletion with cascading track association cleanup, renaming, and individual track removal.

## Requirements

### Requirement: Custom Playlist Deletion
The database SHALL provide an atomic operation to delete a playlist and all its associated track entries from the database.

#### Scenario: Deleting an existing playlist
- **GIVEN** an existing playlist with ID `pl_1` and 3 associated tracks in SQLite
- **WHEN** `delete_playlist(&PlaylistId::new("pl_1"))` is invoked
- **THEN** both the `playlists` entry and all 3 rows in `playlist_tracks` are deleted permanently

### Requirement: Removing Tracks from Playlist
The database SHALL provide an operation to remove a specific track from a playlist without deleting the track itself.

#### Scenario: Removing a track
- **GIVEN** a playlist containing tracks `trk_1` and `trk_2`
- **WHEN** `remove_track_from_playlist(&pl_id, &trk_1)` is invoked
- **THEN** `trk_1` is removed from `playlist_tracks` for that playlist, and `trk_2` remains intact

### Requirement: Renaming Playlists
The database SHALL provide an operation to update the title and optional description of an existing playlist.

#### Scenario: Renaming a playlist
- **GIVEN** a playlist with title "Old Title"
- **WHEN** `rename_playlist(&pl_id, "New Title", None)` is invoked
- **THEN** the playlist record in SQLite is updated with the new title
