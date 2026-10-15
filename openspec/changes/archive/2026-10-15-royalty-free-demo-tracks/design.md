## Context

`DemoProvider` in `crates/storage/src/demo.rs` was built using synthetic pure-tone PCM sine waves (`generate_pcm_wav`). While functional for verifying the audio pipeline, it lacks authentic musicality for first-time users. See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Bundle 3 authentic Creative Commons (CC-BY 3.0/4.0) musical compositions by Kevin MacLeod (`incompetech.com`).
- Store audio files in `crates/storage/assets/demo/` and embed them into `DemoProvider` using `include_bytes!`.
- Set accurate metadata (titles, artist attribution, album titles, duration in seconds, `AudioFormat::Ogg`).
- Include licensing attribution in `crates/storage/assets/demo/LICENSE.txt`.
- Keep total bundled asset size under ~4.2MB to strictly preserve the `<50MB RAM` constraint.

**Non-Goals:**
- Streaming demo tracks over the internet (offline bundled playback is required for instant, reliable out-of-the-box experience).
- Adding new audio codecs or dependencies (`rodio` already integrates `lewton` for Ogg Vorbis decoding).

## Decisions

### 1. Ogg Vorbis Format vs Uncompressed WAV
- **Decision:** Use Ogg Vorbis (`.ogg`) compressed format.
- **Rationale:** Uncompressed 44.1kHz stereo WAV files for 3 full tracks would consume >70MB, directly violating Spoty's `<50MB RAM` rule and bloating the Git repository. Ogg Vorbis provides pristine audio quality at ~180-200 kbps, requiring only ~4.1MB total for all 3 tracks.
- **Alternatives Considered:** MP3 (comparable size, but Ogg is fully open and natively supported by `lewton`/`symphonia` in Rodio).

### 2. Compile-Time Embedding via `include_bytes!`
- **Decision:** Embed audio bytes directly in `crates/storage` via `include_bytes!`.
- **Rationale:** Ensures zero runtime path dependency issues across tests, local cargo runs, and packaged macOS `.app` bundles.
- **Alternatives Considered:** Loading from filesystem path at runtime (brittle across dev, test, and release packaging).

### 3. Track Selection & Attribution
- **Decision:** Select 3 distinct tracks from composer Kevin MacLeod:
  1. *Consort for Brass* (Classical / Instrumental, 51s)
  2. *The Rule* (Orchestral / Theme, 57s)
  3. *Monkeys Spinning Monkeys* (Playful / Pizzicato, 2m 5s)
- **Rationale:** All three are covered by Creative Commons Attribution (CC-BY), allowing free redistribution and software embedding with proper attribution.

## Risks / Trade-offs

- **[Risk]** Slightly increased binary size (~4MB). → **Mitigation:** 4MB is very compact for a modern desktop application and completely negligible compared to Electron builds (>150MB).
- **[Risk]** Audio decoder compatibility with OGG in `spoty-audio`. → **Mitigation:** Verified `rodio::Decoder` with `lewton` integration in Cargo workspace. Unit tests will explicitly test decoding demo streams into valid sample sources.
