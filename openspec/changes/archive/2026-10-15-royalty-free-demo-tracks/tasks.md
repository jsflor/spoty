## 1. Asset Staging & Licensing (`crates/storage/assets/demo/`)

- [x] 1.1 Stage the 3 CC-BY audio files (`consort_for_brass.ogg`, `the_rule.ogg`, `monkeys_spinning_monkeys.ogg`) into `crates/storage/assets/demo/`. Verify files exist and are valid Ogg Vorbis audio.
- [x] 1.2 Create `crates/storage/assets/demo/LICENSE.txt` documenting the Creative Commons attribution for Kevin MacLeod (`incompetech.com`) and source links. Verify file exists.

## 2. Demo Provider Implementation (`crates/storage/src/demo.rs`)

- [x] 2.1 Update `crates/storage/src/demo.rs` to embed audio bytes via `include_bytes!` and configure `Track`, `Album`, `Artist`, and `Playlist` models with accurate metadata and `AudioFormat::Ogg`.
- [x] 2.2 Remove obsolete `generate_pcm_wav` helper function and verify compilation with `cargo check -p spoty-storage`.

## 3. Testing & Quality Gates

- [x] 3.1 Update and expand unit tests in `crates/storage/src/demo.rs` to verify track listing, metadata correctness, and audio stream decoding. Verify with `cargo test -p spoty-storage`.
- [x] 3.2 Run full workspace quality gates with `cargo test --workspace` and `cargo fmt --all -- --check`.
