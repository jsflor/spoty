# ui/all-songs-view Specification

## Purpose
Provides a dedicated flat table view for all scanned tracks and favorited "Liked Songs", with interactive column sorting, real-time filtering, and one-click full playback.

## Requirements

### Requirement: Flat Track Listing
The UI SHALL render an "All Songs" view displaying every track in the loaded library within a tabular layout containing columns for `#`, `Title`, `Artist`, `Album`, and `Duration`.

#### Scenario: Browsing unorganized tracks
- **GIVEN** a library containing tracks with missing or inconsistent metadata
- **WHEN** the user navigates to `Route::AllSongs`
- **THEN** all tracks are listed in a scrollable table regardless of whether they belong to a playlist or artist

### Requirement: Interactive Column Sorting
The songs table SHALL allow clicking on any column header to sort the track list in ascending or descending order.

#### Scenario: Sorting by title
- **WHEN** the user clicks the "Title" column header
- **THEN** the tracks reorder alphabetically by title; clicking again reverses the sort order

### Requirement: Real-Time Table Filtering
The songs view SHALL provide an inline search filter input that instantly filters the displayed table rows by title, artist, or album.

#### Scenario: Searching within All Songs
- **WHEN** the user types "synth" in the view filter input
- **THEN** only rows matching "synth" in their title, artist, or album remain visible

### Requirement: Liked Songs Hero Banner and Aggregation
The UI SHALL render a dedicated "Liked Songs" view at `Route::LikedSongs` featuring Spotify's purple gradient hero banner and listing only tracks currently marked as favorites in the database.

#### Scenario: Viewing favorited tracks
- **WHEN** user clicks on "Liked Songs" from the sidebar
- **THEN** the central panel displays the purple gradient banner and all favorited tracks
