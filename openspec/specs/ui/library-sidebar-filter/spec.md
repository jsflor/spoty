# ui/library-sidebar-filter Specification

## Purpose
Provides inline real-time search filtering and sorting options for items displayed within the Library sidebar.

## Requirements

### Requirement: Library Sidebar Search Filtering
The sidebar SHALL provide a search text input that immediately filters visible playlists and artists matching the entered search query.

#### Scenario: Filtering playlists in sidebar
- **GIVEN** a library with playlists "Synthwave Essentials" and "Rock Classics"
- **WHEN** user types "synth" in the sidebar search box
- **THEN** only "Synthwave Essentials" remains visible in the sidebar list

### Requirement: Library Sorting Order
The sidebar SHALL provide a sort dropdown allowing the user to sort items by Recents, Alphabetical (A–Z), and Recently Added.

#### Scenario: Sorting playlists alphabetically
- **WHEN** user selects "Alphabetical" from the sidebar sort menu
- **THEN** playlists are displayed sorted in ascending alphabetical order by title
