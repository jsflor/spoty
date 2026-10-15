## Why

Spoty's current `DemoProvider` synthesizes rudimentary pure-tone sine waves (440Hz, 523Hz, 659Hz) rather than authentic musical compositions. To deliver an authentic Spotify desktop experience out of the box without requiring external cloud accounts or internet streaming, Spoty needs authentic, high-fidelity royalty-free music bundled directly with the application while remaining well within the strict `<50MB RAM` boundary.

## What Changes

- Bundle authentic, high-quality Creative Commons (CC-BY 3.0 / 4.0) demo tracks by composer Kevin MacLeod (incompetech.com) encoded in compact Ogg Vorbis format.
- Replace synthetic sine wave generation in `DemoProvider` with real compressed audio bytes embedded via `include_bytes!`.
- Provide accurate track metadata (real titles, artist attribution, album names, durations, and album artwork).
- Retain curated "Spoty Favorites" demo playlist and catalog search across all demo tracks.
- Adhere strictly to the `<50MB RAM` memory constraint by keeping total bundled audio under 5MB.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `storage/provider-abstraction`: Update `Requirement: Demo Audio Asset Resolution` to specify bundled Creative Commons royalty-free music assets with accurate metadata, durations, and audio stream decoding.

## Impact

- `crates/storage/src/demo.rs`: Updated `DemoProvider` catalog and byte storage to embed royalty-free Ogg Vorbis audio tracks.
- `crates/storage/assets/demo/`: Directory added to store CC-BY audio files and license documentation.
- `crates/storage/tests/`: Unit and integration tests in `crates/storage` verified for decoding and streaming.
- `crates/audio`: Verified that `rodio` audio worker decodes embedded Ogg Vorbis audio seamlessly.
