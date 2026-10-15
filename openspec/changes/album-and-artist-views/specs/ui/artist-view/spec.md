# ui/artist-view Specification

## Purpose
Provides an artist profile page showcasing the artist's portrait banner, top popular tracks, and discography albums.

## Requirements

### Requirement: Artist Profile Presentation
The UI SHALL render an artist profile page at `Route::Artist(ArtistId)` with a verified artist header, artist portrait, and one-click artist playback.

#### Scenario: Navigating to artist profile
- **WHEN** user clicks on an artist in the sidebar, Home carousel, or track row
- **THEN** the central panel displays the artist page with their popular tracks and albums

### Requirement: Artist Popular Tracks and Discography
The artist view SHALL list the artist's top tracks with playback and context menu controls, and display an album carousel linking to individual album pages.

#### Scenario: Selecting an album from artist page
- **WHEN** user clicks an album card in the artist's discography section
- **THEN** the view transitions to that album's detail page
