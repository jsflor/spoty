## 1. Desktop Keyboard Shortcuts (`spoty-ui`)

- [x] 1.1 Implement `handle_keyboard_shortcuts` in `crates/ui/src/app.rs` checking `!ctx.wants_keyboard_input()`.
- [x] 1.2 Wire Space for Play/Pause, Arrows for Seek and Volume, M for Mute, S for Shuffle, and R for Repeat.
- [x] 1.3 Add unit tests verifying keyboard shortcut command mappings. Verify with `cargo test -p spoty-ui`.

## 2. macOS Application Packaging & CI/CD Pipeline

- [x] 2.1 Create macOS bundle packaging script `scripts/package-macos.sh` generating `Spoty.app` with `Info.plist`.
- [x] 2.2 Create GitHub Actions workflow `.github/workflows/ci.yml` matrix testing macOS and Linux.
- [x] 2.3 Run full workspace test suite and execute `scripts/package-macos.sh` to verify bundle creation. Verify with `cargo test --workspace`.
