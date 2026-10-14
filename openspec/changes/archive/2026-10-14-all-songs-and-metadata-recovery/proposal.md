# Proposal: all-songs-and-metadata-recovery

## Why

When users import music files that lack ID3 tags or embedded artist metadata, those tracks fall back to "Unknown Artist" and become extremely difficult or impossible to locate in Spoty. Currently, Spoty only provides browsing through playlists and artists, while leaving `library.artists` and `library.albums` completely empty across OpenDAL, Subsonic, and SQLite storage backends. Users have no central location to view all imported songs, search flat track lists, or browse their favorited "Liked Songs".

## What Changes

1. **Storage & Metadata Improvements (`crates/storage/src/`)**:
   - Enhance metadata extraction with directory-aware heuristics: if ID3 tags are missing, inspect parent directory paths (e.g. `Music/<Artist>/<Album>/<Track>.mp3`) to infer artist and album names before defaulting to "Unknown Artist".
   - Automatically aggregate unique `Artist` and `Album` entities across all tracks in `Library`, populating `library.artists` and `library.albums` dynamically for OpenDAL, Subsonic, and SQLite cache.
2. **Navigation & Routes (`crates/ui/src/navigation.rs`)**:
   - Add `Route::AllSongs` and `Route::LikedSongs` to the application routing system.
3. **Sidebar Library Integration (`crates/ui/src/app.rs`)**:
   - Pin "Liked Songs" (with Spotify purple gradient and heart icon) and "All Songs" at the top of the sidebar.
   - Make category filter chips ("Playlists", "Songs", "Artists") interactive, allowing quick switching and filtering.
4. **Dedicated "All Songs" / "Liked Songs" View (`crates/ui/src/views/songs.rs`)**:
   - Implement a full-featured table view for all tracks with column sorting (by Index, Title, Artist, Album, Duration).
   - Provide an in-view instant filter search box to quickly filter songs by title, artist, album, or file path.
   - For "Liked Songs", render Spotify's signature purple gradient hero banner (`#450af5` to `#8e8ee5`) and list all user-favorited tracks.

## Capabilities

### New Capabilities
- `ui/all-songs-view`: Flat table view for all imported tracks with column header sorting, inline search filtering, and Spotify-styled "Liked Songs" variant.

### Modified Capabilities
- `storage/metadata-tag-extraction`: Extended with directory-path heuristics (`Artist/Album/Track`) when embedded tags are missing.
- `storage/provider-abstraction`: Automatic aggregation of unique `Artist` and `Album` models from scanned tracks.
- `ui/navigation-and-views`: Upgraded navigation stack and library sidebar with pinned items and interactive filter chips.

## Impact

- Affects `crates/storage/src/metadata.rs`, `crates/storage/src/provider.rs`, `crates/storage/src/opendal_provider.rs`, `crates/storage/src/db.rs`.
- Affects `crates/ui/src/navigation.rs`, `crates/ui/src/app.rs`, and creates `crates/ui/src/views/songs.rs`.
- Zero breaking changes to audio decoding or core playback engines.
