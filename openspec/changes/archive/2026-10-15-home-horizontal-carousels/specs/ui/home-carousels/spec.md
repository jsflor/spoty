# ui/home-carousels Specification

## Purpose
Provides single-row horizontally scrollable carousels on the Home screen for tracks, playlists, and artists, matching Spotify's desktop layout.

## Requirements

### Requirement: Single-Row Horizontal Carousels
The Home view SHALL render content sections as single-row horizontal carousels that scroll horizontally without wrapping vertically onto subsequent lines.

#### Scenario: Scrolling through featured items
- **GIVEN** a library with 20 tracks
- **WHEN** the user views the "Featured & Made For You" section on Home
- **THEN** tracks are laid out in a single horizontal row that scrolls horizontally via mouse wheel, trackpad gestures, or header scroll buttons

### Requirement: Carousel Navigation Controls
Each carousel section header SHALL provide left (`‹`) and right (`›`) navigation buttons to page the horizontal scroll position.

#### Scenario: Paging carousel
- **WHEN** the user clicks the right chevron button (`›`) on a carousel header
- **THEN** the carousel smoothly advances its horizontal scroll offset forward
