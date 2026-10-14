## Why

As libraries grow to thousands of tracks across local directories and cloud buckets, scanning and re-extracting metadata on every app launch degrades startup speed. An embedded SQLite database (`library.db`) provides instant sub-millisecond startup, incremental caching, user-created playlists, and persistent track favorites (`♡`).

## What Changes

- Add embedded SQLite (`rusqlite` with `bundled` feature) to `spoty-storage`.
- Implement `LibraryDatabase` managing schema migrations, track caching, favorites tracking, and custom playlists.
- Update `SpotyApp` to load cached library state instantly on boot and write scanned tracks to the SQLite database.
- Add an interactive favorite toggle (`♡` / `💚`) in the player bar and track rows.
- Add a "+ Create Playlist" button in the library sidebar to create and persist user playlists.

## Capabilities

### New Capabilities
- `storage/sqlite-cache-and-favorites`: Embedded SQLite storage layer for track caching, custom playlist creation, and track favorites persistence.

### Modified Capabilities
<!-- None -->

## Impact

- Adds `rusqlite` dependency to `spoty-storage`.
- Drastically reduces startup latency by loading cached libraries instantly from disk.
- Enables user-created playlists and favorites without external servers.
