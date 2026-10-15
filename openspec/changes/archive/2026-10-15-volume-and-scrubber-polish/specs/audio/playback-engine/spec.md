# audio/playback-engine Specification

## ADDED Requirements

### Requirement: Volume Restoration on Unmute
The audio engine SHALL record the previous non-zero volume level when muting, and restore that level (or a minimum default of 50%) when unmuted.

#### Scenario: Unmuting restored volume
- **GIVEN** playback volume is set to 80%
- **WHEN** user toggles mute twice
- **THEN** volume is restored to 80% and audio resumes at that level

### Requirement: Auto-Unmute on Volume Slider Interaction
Setting a non-zero volume on a muted player SHALL automatically clear the muted state.

#### Scenario: Dragging volume while muted
- **GIVEN** player is currently muted
- **WHEN** user sets volume to 40%
- **THEN** mute state is set to false and audio begins playing at 40%
