# Tasks: logarithmic-volume-and-crossfade

## 1. Perceptual Volume Curve (`crates/audio/src/worker.rs`)

- [ ] 1.1 Implement perceptual cubic/logarithmic gain mapping in `AudioWorker::handle_command` for `SetVolume`.
- [ ] 1.2 Write unit tests verifying that 50% slider value produces approximately $-18\text{ dB}$ (0.125 gain) attenuation.

## 2. Crossfade Engine Implementation (`crates/audio/`)

- [ ] 2.1 Add crossfade configuration parameter to `PlayerController` and `AudioWorker`.
- [ ] 2.2 Implement pre-loading and dual-sink gain interpolation during track boundary transitions.
- [ ] 2.3 Write integration tests for crossfading in `crates/audio/tests/`.

## 3. Settings UI Integration (`crates/ui/src/`)

- [ ] 3.1 Add `crossfade_secs` field to `SpotyConfig` and persist to configuration file.
- [ ] 3.2 Add interactive crossfade slider (0 to 12s) to `SettingsView`.

## 4. Verification and Quality Gates

- [ ] 4.1 Run full workspace test suite `cargo test --workspace`.
- [ ] 4.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
