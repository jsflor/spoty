## Context

Milestones 1–7 completed the audio pipeline, storage backends (Local, S3/R2, Subsonic, Spotify), UI views, and database caching. Milestone 8 finalizes desktop ergonomics (hotkeys) and distribution packaging.

## Goals / Non-Goals

**Goals:**
- Implement global keyboard shortcuts in `SpotyApp` using `egui::Context::input`.
- Ensure text inputs (Search, Settings forms) do not trigger playback hotkeys when focused.
- Create `scripts/package-macos.sh` to package `target/dist/Spoty.app`.
- Add `.github/workflows/ci.yml` running cargo tests and lints across macOS and Linux.

**Non-Goals:**
- Apple Developer ID paid notarization (unnotarized bundle can be run locally or codesigned with ad-hoc certificates).

## Decisions

### 1. Egui Context Input Interception
- **Decision:** Check `ctx.wants_keyboard_input()` before processing single-key hotkeys.
- **Rationale:** Prevents typing spaces or letters into the search or settings boxes from inadvertently pausing music or toggling shuffle.

### 2. Standalone App Bundle Shell Script
- **Decision:** Use a clean shell script `scripts/package-macos.sh` that builds `cargo build --release -p spoty-ui` and assembles `Spoty.app/Contents/MacOS/spoty` and `Info.plist`.
- **Rationale:** Requires zero third-party cargo plugins and runs natively on any Mac with Xcode command-line tools.

## Risks / Trade-offs

- **[Risk]** Hotkey conflicts with OS window manager.
  → **Mitigation**: Use standard application-level shortcuts that only fire when the Spoty window is active.
