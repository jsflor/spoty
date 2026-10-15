# Design: consolidated-catalog-storage

## Architectural Overview

```text
┌────────────────────────────────────────────────────────┐
│                   crates/ui (Shell)                    │
│                                                        │
│   SpotyApp ──► catalog.is_favorite(&track_id)  [O(1)]   │
│            ──► catalog.toggle_favorite(&track_id)      │
│            ──► catalog.create_playlist(title, desc)    │
│            ──► catalog.library()                       │
└───────────────────────────┬────────────────────────────┘
                            │ Clean Seam: Unified Interface
                            ▼
┌────────────────────────────────────────────────────────┐
│             crates/storage (Catalog Module)            │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                In-Memory Cache                   │  │
│  │  - Library (tracks, albums, artists, playlists)  │  │
│  │  - favorites_index: HashSet<TrackId>             │  │
│  │  - Fast O(1) lookups & filter predicates         │  │
│  └───────────┬──────────────────────────┬───────────┘  │
│              │ Atomic Mutations         │ Ingestion    │
│              ▼                          ▼              │
│  ┌───────────────────────┐  ┌───────────────────────┐  │
│  │    LibraryDatabase    │  │     MusicProvider     │  │
│  │    - SQLite tables    │  │    - Fetch library    │  │
│  │    - WAL journal mode │  │    - Stream tracks    │  │
│  │    - Transaction sync │  │    - Background sync  │  │
│  └───────────────────────┘  └───────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

## Seam & Interface Definition

```rust
#[derive(Clone)]
pub struct Catalog {
    inner: Arc<RwLock<CatalogInner>>,
    db: LibraryDatabase,
}

struct CatalogInner {
    library: Library,
    favorites: HashSet<TrackId>,
}

impl Catalog {
    pub fn open(db_path: impl AsRef<Path>) -> anyhow::Result<Self>;
    pub fn open_in_memory() -> anyhow::Result<Self>;

    // Pure in-memory queries (instant for 60fps rendering)
    pub fn library(&self) -> Library;
    pub fn is_favorite(&self, track_id: &TrackId) -> bool;
    pub fn get_favorites(&self) -> Vec<TrackId>;
    pub fn get_playlist(&self, id: &PlaylistId) -> Option<Playlist>;

    // Atomic mutations (persists to SQLite and updates in-memory cache)
    pub fn toggle_favorite(&self, track_id: &TrackId) -> anyhow::Result<bool>;
    pub fn create_playlist(&self, title: &str, description: Option<&str>) -> anyhow::Result<Playlist>;
    pub fn rename_playlist(&self, id: &PlaylistId, title: &str, description: Option<&str>) -> anyhow::Result<()>;
    pub fn delete_playlist(&self, id: &PlaylistId) -> anyhow::Result<()>;
    pub fn add_track_to_playlist(&self, playlist_id: &PlaylistId, track_id: &TrackId) -> anyhow::Result<()>;
    pub fn remove_track_from_playlist(&self, playlist_id: &PlaylistId, track_id: &TrackId) -> anyhow::Result<()>;

    // Provider synchronization
    pub async fn sync_provider(&self, provider: &dyn MusicProvider) -> anyhow::Result<()>;
}
```

## Key Technical Decisions

### 1. In-Memory Favorites Index (`HashSet<TrackId>`)
At startup, `Catalog` queries `LibraryDatabase::get_favorites()` once and populates an in-memory `HashSet<TrackId>`. 
- `is_favorite` acquires an `Arc<RwLock>` read lock and checks set membership in nanoseconds.
- No SQLite connections or locks are acquired during egui UI draw passes.
- `toggle_favorite` inserts or deletes from SQLite within a transaction and updates the in-memory set atomically, guaranteeing zero discrepancy.

### 2. Elimination of Dual-Write Boilerplate in UI
Previously, adding a track to a playlist required:
```rust
let _ = self.db.add_track_to_playlist(&pl_id, &trk_id);
if let Some(pl) = self.library.playlists.iter_mut().find(|p| p.id == pl_id) {
    if !pl.track_ids.contains(&trk_id) {
        pl.track_ids.push(trk_id);
    }
}
```
With `Catalog`:
```rust
let _ = self.catalog.add_track_to_playlist(&pl_id, &trk_id);
```
Both the SQLite `playlist_tracks` table and the cached `Playlist.track_ids` vector are updated together inside `Catalog`.

### 3. Asynchronous Provider Ingestion
Instead of blocking the UI thread with `tokio_rt.block_on(provider.fetch_library())`, `sync_provider` runs asynchronously on a worker task:
- Fetches new library entries.
- Upserts tracks into SQLite via batch transaction.
- Re-aggregates artists and albums.
- Atomically replaces the in-memory `Library` snapshot.
- Emits a refresh notification to trigger an egui repaint.

## Testability & Substitutability

Because `Catalog` implements `open_in_memory()`, unit tests can construct a fully functional catalog instance instantly without mocking, verifying both in-memory state and SQL persistence through the single public interface.
