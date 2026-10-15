# Proposal: consolidated-catalog-storage

## Why

Currently, library state management in Spoty is fragmented across two disjoint modules in `crates/storage`: the dumb `Library` struct (holding raw vectors of tracks, playlists, albums, and artists) and `LibraryDatabase` (an SQLite DAO exposing 10 pass-through CRUD methods). The caller—specifically `SpotyApp` in `crates/ui/src/app.rs`—is forced to act as the synchronization glue:
1. **Dual-Write Fragility:** Whenever a playlist is created, renamed, deleted, or altered, `SpotyApp` must perform two separate mutations: executing an SQLite operation via `self.db`, and then finding and mutating the corresponding entity in `self.library.playlists`. If any caller forgets this dual mutation, in-memory state and disk state drift out of sync.
2. **Synchronous Database Queries Inside 60fps Render Loop:** On every frame, `SpotyApp` constructs an `is_fav_closure` that executes `SELECT EXISTS(SELECT 1 FROM favorites WHERE track_id = ?1)` against SQLite for every rendered track row in `PlaylistView`, `SongsView`, `SearchView`, and `HomeView`. Running SQLite queries repeatedly during immediate-mode rendering causes frame-time jitter and unnecessary mutex contention.
3. **Blocking Provider Ingestion:** When switching music providers, `SpotyApp` invokes `self.tokio_rt.block_on(provider.fetch_library())` directly on the UI thread, freezing window rendering during disk scans or remote network requests.

## What Changes

1. **Deep Catalog Module (`crates/storage/src/catalog.rs` / `Catalog`)**:
   - Collapse `LibraryDatabase`, in-memory library caches, and provider synchronization behind a single deep `Catalog` module.
2. **Instant In-Memory Favorites Index (`HashSet<TrackId>`)**:
   - Maintain a synchronized in-memory set of favorited track IDs. Queries such as `catalog.is_favorite(&track_id)` become instantaneous O(1) lookups with zero SQLite overhead during UI rendering.
3. **Atomic Single-Seam Mutations**:
   - Consolidate all playlist and favorites operations (`create_playlist`, `rename_playlist`, `delete_playlist`, `add_track_to_playlist`, `remove_track_from_playlist`, `toggle_favorite`) behind the `Catalog` interface. The implementation updates the SQLite database and internal in-memory collections atomically with locality.
4. **Non-blocking Provider Refresh**:
   - Provider library ingestion and track upserts are handled asynchronously within `Catalog`, returning updated snapshot notifications without halting the UI event loop.

## Capabilities

### New Capabilities
- `storage/consolidated-catalog`: Unified catalog subsystem combining fast in-memory indexing, provider synchronization, and SQLite persistence behind a small interface.

### Modified Capabilities
- `storage/sqlite-cache-and-favorites`: SQLite tables remain the persistence layer, but database connections and queries are encapsulated completely within the catalog module.
- `storage/playlist-crud-operations`: Playlist CRUD operations are executed through the unified catalog interface, ensuring atomic cache updates.

## Impact

- **Crates Affected:**
  - `crates/storage`: Creates `crates/storage/src/catalog.rs`, exposes `Catalog` as the primary interface, and encapsulates `LibraryDatabase`.
  - `crates/ui`: Removes direct `LibraryDatabase` dependencies and duplicate `Library` mutations in `crates/ui/src/app.rs`.
- **Breaking Changes:** Replaces `app.db` and manual `app.library` mutations with `app.catalog`. All view call sites receive an instant in-memory favorite predicate.
