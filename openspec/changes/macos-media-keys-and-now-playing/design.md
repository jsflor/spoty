# Design: macos-media-keys-and-now-playing

## Architecture

```text
[macOS System Events]
        │
        ├─ Hardware Media Keys (Play/Pause, Next, Prev)
        │           │
        │           ▼
        │   [crates/ui/src/os/media_keys.rs]
        │           │
        │           ▼
        │   PlayerController (play_pause, next, prev)
        │
        └─ Active Track Events (Track Changed, Play, Pause, Seek)
                    │
                    ▼
            [crates/ui/src/os/now_playing.rs]
                    │
                    ▼
            macOS MPNowPlayingInfoCenter / Control Center Widget
```

## Detailed Component Specifications

### 1. Platform Isolation
- All macOS-specific code is isolated in `crates/ui/src/os/` gated by `#[cfg(target_os = "macos")]`.
- Provide no-op stub implementations for non-macOS platforms to preserve cross-platform compilation.

### 2. Media Keys Handling
- Register a background event listener or Cocoa delegate for system media keys.
- Maps:
  - `Play/Pause`: calls `controller.toggle_play_pause()`
  - `Next`: calls `app.play_next()`
  - `Previous`: calls `app.play_prev()`

### 3. Now Playing Info
- Synchronize track metadata when `PlayerEvent::StateChanged` is received:
  - `MPMediaItemPropertyTitle`: `track.title`
  - `MPMediaItemPropertyArtist`: `track.artist`
  - `MPMediaItemPropertyPlaybackDuration`: `state.duration_secs`
  - `MPNowPlayingInfoPropertyElapsedPlaybackTime`: `state.position_secs`
  - `MPNowPlayingInfoPropertyPlaybackRate`: `1.0` if playing, `0.0` if paused
