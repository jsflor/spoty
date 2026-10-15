# audio/playback-engine Specification

## ADDED Requirements

### Requirement: Perceptual Decibel Gain Scaling
The audio engine SHALL convert linear volume slider inputs into a logarithmic/perceptual acoustic gain curve ($G = V^3$) to provide smooth loudness adjustments matching human ear perception.

#### Scenario: Setting 50% volume
- **WHEN** user sets the volume slider to 50%
- **THEN** audio output power is attenuated logarithmically by approximately $-18\text{ dB}$ (gain: 0.125)

### Requirement: Track Crossfade Transitions
When crossfading is configured ($N > 0$ seconds), the audio engine SHALL begin playback of the subsequent track $N$ seconds prior to the end of the current track, fading the outgoing track to zero while fading the incoming track to full volume.

#### Scenario: Smooth transition between consecutive tracks
- **GIVEN** crossfade is set to 4 seconds
- **WHEN** the active track has 4 seconds remaining
- **THEN** the next track begins playing quietly, ramping up volume as the active track fades out
