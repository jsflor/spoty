# ui/audio-visualizer Specification

## Purpose
Computes real-time audio sample amplitude levels and renders an animated audio spectrum equalizer visualizer in the player interface.

## Requirements

### Requirement: Audio Level Streaming
The playback engine SHALL periodically measure audio sample levels during active playback and dispatch level updates to the UI thread.

#### Scenario: Active playback level calculation
- **WHEN** audio is playing through the playback sink
- **THEN** the audio engine emits level updates containing frequency/amplitude bins at least 10 times per second

### Requirement: Animated Spectrum Visualizer Widget
The UI SHALL render an animated spectrum bar widget that visualizes the current audio amplitudes using the Spotify Encore color palette.

#### Scenario: Rendering spectrum bars
- **WHEN** audio is currently playing
- **THEN** visualizer bars animate dynamically according to incoming audio amplitude levels
