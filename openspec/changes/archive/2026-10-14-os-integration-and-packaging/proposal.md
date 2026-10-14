## Why

To provide a first-class desktop user experience, Spoty needs standard desktop keyboard shortcuts (Space for Play/Pause, arrows for seek/volume, M for mute, S for shuffle, R for repeat) and a turnkey macOS application bundle (`Spoty.app`) packaging pipeline for distribution.

## What Changes

- Add comprehensive desktop keyboard navigation in `spoty-ui` (Space, Left/Right for seeking/skipping, Up/Down for volume, Mute, Shuffle, Repeat).
- Add packaging scripts (`scripts/package-macos.sh`) to build a standalone `Spoty.app` application bundle with `Info.plist` and application icons.
- Add GitHub Actions CI/CD matrix (`.github/workflows/ci.yml`) to automatically test and build Spoty across macOS, Linux, and Windows.

## Capabilities

### New Capabilities
- `ui/keyboard-shortcuts`: Full desktop keyboard shortcut handling for playback control, seeking, volume adjustment, and search focus.
- `dist/macos-packaging`: Standalone macOS application bundle (`Spoty.app`) generation with metadata and multi-platform CI/CD workflow.

### Modified Capabilities
<!-- None -->

## Impact

- Enables keyboard-driven desktop playback.
- Creates `scripts/package-macos.sh` and `.github/workflows/ci.yml`.
- Produces a drag-and-drop macOS `Spoty.app` bundle ready to run from `/Applications`.
