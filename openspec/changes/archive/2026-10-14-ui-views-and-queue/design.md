## Context

See `proposal.md` for motivation. We are expanding the UI to support Spotify's distinct navigation architecture and a robust queue manager.

## Goals / Non-Goals

**Goals:**
- Implement an explicit `Route` enum in `spoty-ui` (`Home`, `Search`, `Playlist(PlaylistId)`, `Album(AlbumId)`, `Settings`).
- Implement a `NavigationHistory` struct with `history: Vec<Route>` and `history_index: usize`.
- Implement a `QueueManager` with `tracks: Vec<Track>`, `current_index: usize`, shuffle index mappings, and continuous next-track dispatch.
- Build Spotify's visual elements:
  - Time-of-day greeting ("Good morning / afternoon / evening") on Home.
  - Playlist hero header with large artwork and metadata.
  - Category browse cards on Search with vibrant pastel gradients.
  - Settings panel with directory picker for OpenDAL local folder scanning.

**Non-Goals:**
- Remote multiplayer listening session sync.
- Cloud sync of user play counts or history.

## Decisions

### 1. Route State Representation
- **Decision:** Use an explicit enum `Route` with typed IDs (`PlaylistId`, `AlbumId`) and a history stack.
- **Rationale:** Simple, deterministic, and makes `can_go_back()` and `can_go_forward()` trivial to evaluate.

### 2. Queue in UI vs Audio Engine
- **Decision:** Keep the playlist queue in the client layer (`QueueManager`), while the `AudioWorker` continues to manage the single active stream buffer and emits `TrackEnded`.
- **Rationale:** The client layer has full access to the active catalog, shuffle settings, user queue alterations, and provider streaming handles. When `TrackEnded` is received, the client triggers `controller.load_track(...)` for the next song.

## Risks / Trade-offs

- **[Risk]** Heavy allocations during rapid route navigation.
  → **Mitigation**: Store lightweight `TrackId` / `PlaylistId` in routes; resolve data from the cached `Library`.
