# ui/navigation-and-views Specification

## Purpose
Provides multi-view routing, navigation history stack, playlist detail hero header, and settings view in `spoty-ui`.

## Requirements

### Requirement: Navigation History Stack
The UI SHALL maintain a forward and back navigation history stack allowing users to navigate between views using the back (`◀`) and forward (`▶`) buttons.

#### Scenario: Navigating back to previous view
- **WHEN** user clicks on a playlist card and then clicks the back button
- **THEN** the view switches back to the previous view and enables the forward button

### Requirement: Dedicated View Rendering
The UI SHALL render distinct views for `Home`, `Search`, `PlaylistDetail`, `AlbumDetail`, and `Settings`.

#### Scenario: Viewing playlist details
- **WHEN** user selects a playlist from the sidebar or home grid
- **THEN** the central panel displays the playlist hero banner with title, description, and the full track list table
