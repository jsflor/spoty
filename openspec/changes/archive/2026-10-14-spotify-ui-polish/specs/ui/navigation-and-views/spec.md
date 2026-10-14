## ADDED Requirements

### Requirement: Cards on Canvas Shell Architecture
The UI SHALL render using a "Cards on Canvas" shell architecture with a pure black background canvas (`#000000`) and floating elevated rounded cards (`#121212` with 8px corner radii) separating the sidebar navigation from the main content viewport.

#### Scenario: Rendering floating panels
- **WHEN** the application renders
- **THEN** the sidebar and central content panels appear as distinct rounded cards with an 8px gutter on the dark canvas

## MODIFIED Requirements

### Requirement: Dedicated View Rendering
The UI SHALL render distinct views for `Home`, `Search`, `PlaylistDetail`, `AlbumDetail`, and `Settings` styled according to Spotify Encore design conventions, including time-based greetings and quick-play cards on Home, and full-bleed gradient hero banners with interactive hoverable track tables on Playlist view.

#### Scenario: Viewing playlist details
- **WHEN** user selects a playlist from the sidebar or home grid
- **THEN** the central panel displays the playlist hero banner with title, description, and the full track list table
