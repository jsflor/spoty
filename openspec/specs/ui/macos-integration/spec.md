# ui/macos-integration Specification

## Purpose
Integrates Spoty with macOS system-wide media playback hardware keys and the macOS Control Center Now Playing information center.

## Requirements

### Requirement: Global Hardware Media Keys
On macOS systems, the application SHALL respond to physical keyboard media keys (Play/Pause, Next Track, Previous Track) even when the application window is minimized or not in focus.

#### Scenario: Pressing Play/Pause while app is in background
- **GIVEN** Spoty is minimized and music is currently playing
- **WHEN** user presses the keyboard Play/Pause media key
- **THEN** audio playback pauses immediately

### Requirement: System Now Playing Metadata
The application SHALL broadcast active track details (title, artist, duration, playback status) to macOS system media controls.

#### Scenario: Viewing macOS Control Center media widget
- **WHEN** a track begins playing
- **THEN** the macOS Control Center and lock screen display the song title and artist
