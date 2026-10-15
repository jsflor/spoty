# Tasks: volume-and-scrubber-polish

## 1. SpotifySlider Hover Tooltip Extension (`crates/ui/src/widgets/slider.rs`)

- [x] 1.1 Implement `SliderTooltip` enum and builder methods (`tooltip_time`, `tooltip_percent`) in `SpotifySlider`.
- [x] 1.2 Paint elevated floating tooltip badge directly above cursor on hover and drag with exact Spotify Encore styling.
- [x] 1.3 Write unit tests in `crates/ui/src/widgets/slider.rs` for tooltip value calculation. Verify with `cargo test -p spoty-ui`.

## 2. Audio Worker Volume Restoration (`crates/audio/src/worker.rs`)

- [x] 2.1 Update `AudioWorker` to track `previous_volume` and restore previous volume or 50% default upon unmuting.
- [x] 2.2 Automatically clear mute status when `SetVolume` is called with volume > 0.
- [x] 2.3 Verify audio tests with `cargo test -p spoty-audio`.

## 3. Player Bar Integration (`crates/ui/src/app.rs`)

- [x] 3.1 Attach `.tooltip_time()` to timeline scrub slider in `player_bar`.
- [x] 3.2 Attach `.tooltip_percent()` to volume slider in `player_bar`.
- [x] 3.3 Ensure tabular monospace styling for elapsed and remaining time labels.
- [x] 3.4 Verify compilation with `cargo check -p spoty-ui`.

## 4. Verification and Quality Gates

- [x] 4.1 Run full workspace test suite `cargo test --workspace`.
- [x] 4.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
