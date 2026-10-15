# audio/sink-abstraction Specification

## Purpose
Defines the hardware output abstraction seam separating the audio playback engine's deterministic state logic from physical operating system audio devices.

## Requirements

### Requirement: Swappable Audio Output Adapters
The audio engine SHALL accept any backend satisfying the `AudioSink` trait, allowing audio playback logic to run against physical hardware in production or in-memory sinks in test environments.

#### Scenario: Running in production environment
- **GIVEN** operating system audio hardware is present
- **WHEN** player controller initializes
- **THEN** it instantiates `RodioSink` and outputs audio samples via system sound devices

#### Scenario: Running in headless test environment
- **GIVEN** an automated test or headless CI environment
- **WHEN** audio engine initializes with `MockSink`
- **THEN** commands execute deterministically without opening system audio devices
- **AND** assertions complete without thread sleep delays

### Requirement: Deterministic Volume and Mute Control
Setting volume levels and toggling mute status SHALL alter the target sink's volume and update the authoritative playback state without temporal lag.

#### Scenario: Muting audio through sink seam
- **GIVEN** audio is playing at volume 0.7
- **WHEN** user toggles mute
- **THEN** the sink's output volume is set to 0.0
- **AND** the engine records the 0.7 level for restoration upon unmute
