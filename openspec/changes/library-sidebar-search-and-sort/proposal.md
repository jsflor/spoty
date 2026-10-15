# Proposal: library-sidebar-search-and-sort

## Why

In large music libraries with dozens of playlists, artists, and saved collections, finding a specific playlist or artist in Spoty's sidebar requires endless scrolling. Spotify desktop addresses this by placing an inline search filter input ("Search in Your Library") right below the category chips, paired with a sort dropdown menu ("Recents", "Recently Added", "Alphabetical (A–Z)"). Adding sidebar filtering and sorting transforms the library into an efficient, instantly searchable navigator.

## What Changes

1. **Sidebar Filter Search Input (`crates/ui/src/app.rs`)**:
   - Add a compact text search box (`🔍`) directly above the sidebar list.
   - Real-time filtering matching playlist titles and artist names as the user types.
2. **Library Sort Dropdown Menu**:
   - Add a sort menu button with options:
     - **"Recents"**: Default usage order.
     - **"Alphabetical (A–Z)"**: Sorts items alphabetically by title/name.
     - **"Recently Added"**: Sorts custom playlists by newest creation timestamp.
3. **Sidebar Category Chip Alignment**:
   - Seamless integration with the existing "Playlists", "Songs", and "Artists" filter chips.

## Capabilities

### New Capabilities
- `ui/library-sidebar-filter`: Real-time text search filtering and sorting dropdown ("Recents", "Alphabetical", "Recently Added") for the Library sidebar.

## Impact

- Affects `crates/ui/src/app.rs`.
- Zero changes to storage backends or audio playback.
