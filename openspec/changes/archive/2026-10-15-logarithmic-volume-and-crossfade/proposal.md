# Proposal: logarithmic-volume-and-crossfade

## Why

Human hearing perceives loudness logarithmically according to the Weber-Fechner law. Linear attenuation causes audio to sound whisper-quiet across the lower 50% of the volume slider, followed by rapid, disproportionate loudness gains near 100%. In addition, Spotify desktop provides a configurable crossfade setting (0–12 seconds) that smoothly blends the tail of a finishing track with the head of the upcoming track, avoiding jarring audio cutoffs. Implementing logarithmic volume curves and audio crossfading delivers acoustic fidelity matching production desktop music clients.

## What Changes

1. **Perceptual (Logarithmic / Cubic) Volume Curve (`crates/audio/src/worker.rs`)**:
   - Transform slider positions ($V \in [0, 1]$) to perceptual audio gain via cubic/exponential curve: $G = V^3$ (or standard decibel curve).
   - Ensures natural, smooth volume increases across the entire 0%–100% range.
2. **Configurable Crossfade Engine (`crates/audio/src/`, `crates/ui/src/views/settings.rs`)**:
   - Add a crossfade duration setting (0 to 12 seconds, default: 0 / off).
   - When a track reaches its final $N$ seconds, smoothly attenuate its output while beginning decoding and ramping up the volume of the next queued track.
3. **Settings View Controls**:
   - Add an interactive slider in `SettingsView` allowing users to configure crossfade seconds.

## Capabilities

### Modified Capabilities
- `audio/playback-engine`: Enhanced with logarithmic audio gain scaling and multi-second crossfade blending between sequential tracks.

## Impact

- Affects `crates/audio/src/engine.rs`, `crates/audio/src/worker.rs`, `crates/ui/src/config.rs`, and `crates/ui/src/views/settings.rs`.
- Zero breaking changes to storage providers or UI themes.
