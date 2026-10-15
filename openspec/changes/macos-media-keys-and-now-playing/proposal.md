# Proposal: macos-media-keys-and-now-playing

## Why

Desktop music players are expected to integrate with the host operating system. Currently, Spoty only intercepts keyboard shortcuts when the application window is focused. On macOS, users expect physical media keys (Play/Pause, Next, Previous) to control playback regardless of which app is active, and expect the current track title, artist, and playback status to be displayed in the macOS Control Center and lock screen Now Playing widgets.

## What Changes

1. **Global macOS Media Keys Listener (`crates/ui/src/os/macos.rs` or background worker)**:
   - Listen for macOS media key event notifications (`NX_KEYTYPE_PLAY`, `NX_KEYTYPE_NEXT`, `NX_KEYTYPE_PREVIOUS`) using macOS CoreGraphics / IOKit event taps or lightweight cross-platform bindings.
   - Dispatch commands directly into `PlayerController` to toggle playback or skip tracks.
2. **macOS System Media Information Integration**:
   - Update macOS `MPNowPlayingInfoCenter` / system media info on playback state transitions (track title, artist, duration, playback rate).
   - Display active playback status in the macOS menu bar / Control Center now playing widget.

## Capabilities

### New Capabilities
- `ui/macos-integration`: Background hardware media keys listener and lock screen / Control Center Now Playing integration for macOS.

## Impact

- Affects `crates/ui/Cargo.toml` (target-specific macOS dependencies where applicable) and `crates/ui/src/app.rs`.
- Platform-isolated: conditionally compiled with `#[cfg(target_os = "macos")]` to ensure zero impact on Linux or Windows builds.
