# ui/spotify-player-bar Specification

## Purpose

Provides a persistent 72px–80px Spotify desktop-grade playback bar with a 3-section balanced layout, custom hover-responsive progress and volume sliders with interactive scrub handles, and vector playback control icons.

## Requirements

### Requirement: Three-Section Balanced Player Layout
The player bar SHALL render as a fixed persistent bottom panel divided into three distinct proportional sections: Track Details on the left (30%), Playback Controls & Progress Timeline in the center (40%), and Audio Visualizer & Volume Controls on the right (30%).

#### Scenario: Rendering track details
- **WHEN** a track is actively playing or paused
- **THEN** the left section displays the track cover art (44×44 to 48×48 with 4px corner radius), track title in bold white, artist name in subdued text, and an interactive favorite toggle button

### Requirement: Custom Hover-Responsive Progress Scrubbing
The player timeline SHALL display a slim progress bar that visually highlights in Spotify green upon hover and reveals a circular scrub thumb at the current playhead position.

#### Scenario: Scrubbing playback position
- **WHEN** the user hovers over or drags along the timeline progress bar
- **THEN** the progress bar turns Spotify green (`#1ed760`), renders a white circular scrub thumb, and seeks playback to the clicked time offset

### Requirement: Vector Playback Controls
Playback control buttons (Play/Pause, Previous, Next, Shuffle, Repeat) SHALL be rendered with crisp vector shapes and visual state indicators.

#### Scenario: Toggle shuffle mode
- **WHEN** the user clicks the shuffle button
- **THEN** the shuffle state toggles and displays an active green accent indicator
