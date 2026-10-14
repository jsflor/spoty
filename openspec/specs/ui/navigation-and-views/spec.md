# ui/navigation-and-views Specification

## Purpose
Provides multi-view routing, navigation history stack, playlist detail hero header, and settings view in `spoty-ui`.

## Requirements

### Requirement: Navigation History Stack
The UI SHALL maintain a forward and back navigation history stack allowing users to navigate between views using the back (`◀`) and forward (`▶`) buttons.

#### Scenario: Navigating back to previous view
- **WHEN** user clicks on a playlist card and then clicks the back button
- **THEN** the view switches back to the previous view and enables the forward button

### Requirement: Cards on Canvas Shell Architecture
The UI SHALL render using a "Cards on Canvas" shell architecture with a pure black background canvas (`#000000`) and floating elevated rounded cards (`#121212` with 8px corner radii) separating the sidebar navigation from the main content viewport.

#### Scenario: Rendering floating panels
- **WHEN** the application renders
- **THEN** the sidebar and central content panels appear as distinct rounded cards with an 8px gutter on the dark canvas

### Requirement: Dedicated View Rendering
The UI SHALL render distinct views for `Home`, `Search`, `PlaylistDetail`, `AlbumDetail`, and `Settings` styled according to Spotify Encore design conventions, including time-based greetings and quick-play cards on Home, and full-bleed gradient hero banners with interactive hoverable track tables on Playlist view.

#### Scenario: Viewing playlist details
- **WHEN** user selects a playlist from the sidebar or home grid
- **THEN** the central panel displays the playlist hero banner with title, description, and the full track list table

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
