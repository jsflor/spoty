## Purpose

Provides desktop keyboard shortcut handling for playback control, seeking, volume adjustment, and search navigation.

## ADDED Requirements

### Requirement: Playback Hotkey Controls
The system SHALL handle keyboard hotkeys when text input fields are not active:
- Spacebar toggles Play/Pause.
- Left Arrow seeks backward 5 seconds; Shift + Left Arrow skips to the previous track.
- Right Arrow seeks forward 5 seconds; Shift + Right Arrow skips to the next track.
- Up / Down Arrows increment / decrement audio volume by 5%.
- `M` toggles audio mute.
- `S` toggles shuffle.
- `R` cycles repeat modes.

#### Scenario: Pressing spacebar during playback
- **WHEN** audio is playing and user presses the spacebar (outside text input)
- **THEN** playback pauses immediately

#### Scenario: Pressing arrow keys to seek
- **WHEN** user presses Right Arrow during playback
- **THEN** the active track timeline position advances by 5 seconds
