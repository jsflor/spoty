# ui/album-view Specification

## Purpose
Provides an album details view presenting the album hero header, release information, one-click full playback, and ordered tracklist.

## Requirements

### Requirement: Album Detail Presentation
The UI SHALL render an album view at `Route::Album(AlbumId)` with full-bleed hero banner, cover art, release year, artist link, and total duration.

#### Scenario: Navigating to album
- **WHEN** user selects an album card or clicks an album name link
- **THEN** the central viewport displays the album details with its full tracklist

### Requirement: Album Tracklist Ordering and Playback
The album view SHALL list all album tracks in track-index order and provide an action bar play button to play the entire album from track 1.

#### Scenario: Playing album
- **WHEN** user clicks the circular play button in the album header
- **THEN** the audio engine queues and starts playback of the album's first track
