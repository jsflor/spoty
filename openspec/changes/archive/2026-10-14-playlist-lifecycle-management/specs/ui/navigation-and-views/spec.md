# ui/navigation-and-views Specification

## ADDED Requirements

### Requirement: Custom Playlist Deletion and Confirmation
The UI SHALL provide a deletion mechanism for custom playlists via sidebar context menus and the playlist detail options button (`•••`), displaying a confirmation dialog prior to permanent removal.

#### Scenario: Deleting a playlist from sidebar
- **GIVEN** a custom playlist visible in the library sidebar
- **WHEN** the user right-clicks the playlist and selects "Delete playlist", then confirms the deletion in the modal dialog
- **THEN** the playlist is deleted from the database and sidebar, and if the user was currently viewing that playlist, the view redirects to `Home`

### Requirement: Custom Playlist Renaming
The UI SHALL provide an inline or dialog interface allowing users to rename existing custom playlists.

#### Scenario: Renaming a playlist
- **WHEN** the user selects "Rename" from the playlist context menu or options menu
- **THEN** the user can edit the playlist title, and saving updates both SQLite and the UI immediately
