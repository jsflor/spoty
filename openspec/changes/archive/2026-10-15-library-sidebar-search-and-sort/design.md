# Design: library-sidebar-search-and-sort

## Architecture & Layout

```text
[Your Library Sidebar]
   ├─ Header: "Your Library" [＋ Create Playlist]
   ├─ Category Chips: [Playlists] [Songs] [Artists]
   ├─ Search & Sort Row:
   │    ├─ [🔍 Search in Your Library] (expands on click/focus)
   │    └─ [Sort by: Recents ▾] ──► Menu: "Recents", "Recently Added", "Alphabetical"
   └─ ScrollArea::vertical()
        ├─ Pinned Items: [Liked Songs] [All Songs]
        ├─ Filtered & Sorted Playlists
        └─ Filtered & Sorted Artists
```

## Detailed Component Specifications

### 1. State in `SpotyApp` (`crates/ui/src/app.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibrarySortOrder {
    #[default]
    Recents,
    Alphabetical,
    RecentlyAdded,
}

pub struct SpotyApp {
    ...
    sidebar_search_query: String,
    sidebar_sort_order: LibrarySortOrder,
}
```

### 2. Search & Sort Bar Rendering
- Located directly below the category chips and above the pinned rows.
- Search Input:
  - Compact pill edit with subtle magnifying glass icon.
  - Placeholder: `"Search in Your Library"`.
  - When non-empty, displays a small `✕` clear button.
- Sort Selector:
  - Menu button rendering current sort label (e.g. `Recents ▾`).
  - Options:
    - `"Recents"`
    - `"Recently Added"`
    - `"Alphabetical (A–Z)"`

### 3. List Filtering & Sorting
- Playlists and artists are filtered against `sidebar_search_query.to_lowercase()`.
- If `sidebar_sort_order == Alphabetical`:
  - Playlists sorted by `p.title.to_lowercase()`.
  - Artists sorted by `a.name.to_lowercase()`.
- If `sidebar_sort_order == RecentlyAdded`:
  - Playlists sorted by ID timestamp (newest first).
