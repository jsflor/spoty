# Tasks: audio-sink-adapter-seam

## 1. Sink Seam and Adapters (`crates/audio/src/sink.rs`)

- [x] 1.1 Define `AudioSink` trait with `play`, `pause`, `stop`, `set_volume`, `is_empty`, and `append_source`.
- [x] 1.2 Implement `RodioSink` production adapter wrapping `rodio::OutputStream` and `rodio::Sink`.
- [x] 1.3 Implement `MockSink` and `NullSink` adapters for deterministic testing and headless execution.
- [x] 1.4 Export sink module in `crates/audio/src/lib.rs`. Verify with `cargo test -p spoty-audio`.

## 2. Reclaim AudioEngine Core (`crates/audio/src/engine.rs`)

- [x] 2.1 Refactor `AudioEngine` to own `Box<dyn AudioSink>` and handle all state transitions deterministically.
- [x] 2.2 Move volume restoration, mute toggling, seek positioning, and stream loading into `AudioEngine`.
- [x] 2.3 Refactor `AudioWorker` to delegate command processing directly to `AudioEngine`.
- [x] 2.4 Verify with `cargo test -p spoty-audio`.

## 3. Fast Deterministic Tests (`crates/audio/tests/`)

- [x] 3.1 Rewrite `crates/audio/tests/controller_test.rs` to run deterministic assertions against `MockSink`.
- [x] 3.2 Remove all `std::thread::sleep` delays from `crates/audio/tests/`.
- [x] 3.3 Verify tests pass in <50ms with `cargo test -p spoty-audio`.

## 4. Verification and Quality Gates

- [x] 4.1 Run workspace test suite `cargo test --workspace`.
- [x] 4.2 Verify formatting with `cargo fmt --all -- --check` and compiler checks with `cargo check --workspace`.
