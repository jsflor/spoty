# Design: logarithmic-volume-and-crossfade

## Acoustic Math & Crossfade Flow

### 1. Volume Curve Transformation
Instead of feeding linear slider values directly to the OS audio sink:
$$\text{Gain}(v) = v^3 \quad \text{for } v \in [0.0, 1.0]$$
- At $v = 0.5$ (50% slider): $\text{Gain} = 0.125$ (approximately $-18\text{ dB}$ attenuation), corresponding to a comfortable, naturally audible medium listening level.
- At $v = 1.0$ (100% slider): $\text{Gain} = 1.0$ ($0\text{ dB}$ full volume).
- At $v = 0.1$ (10% slider): $\text{Gain} = 0.001$ (quiet, whisper-level volume).

### 2. Dual-Sink Crossfading Architecture
```text
[Track A Finishing (Last N Seconds)] ──► Sink A: Gain fades 1.0 ──► 0.0
                                                                   │
[Track B Starting (First N Seconds)] ──► Sink B: Gain fades 0.0 ──► 1.0
                                                                   │
                                                                   ▼
                                                            Summed Hardware Output
```
- In `crates/audio/src/worker.rs`:
  - When `crossfade_secs > 0` and remaining playback time $\le \text{crossfade\_secs}$:
    - Pre-load and start the next track on a secondary `Sink`.
    - Linearly interpolate volumes: $\text{Sink A} = (1 - t)$, $\text{Sink B} = t$.
    - Once Track A finishes, drop Sink A and promote Sink B to primary.

### 3. User Configuration (`crates/ui/src/config.rs`)
- Add field `pub crossfade_secs: u32` to `SpotyConfig` (range: `0..=12`).
- Add slider in `SettingsView` under Playback preferences.
