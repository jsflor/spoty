## Context

See `proposal.md` for motivation. We are adding an embedded SQLite storage engine (`rusqlite`) for persistent track metadata caching, user custom playlists, and favorites.

## Goals / Non-Goals

**Goals:**
- Zero runtime C dependency using `rusqlite` bundled compilation.
- Database location at `~/.config/spoty/library.db` (or `./library.db`).
- Schema with `tracks`, `playlists`, `playlist_tracks`, and `favorites` tables.
- Thread-safe `Arc<Mutex<LibraryDatabase>>` connection wrapper.
- Heart/Favorite button in the player bar and track rows.

**Non-Goals:**
- Multi-user access control or external Postgres/MySQL network database connections.
- Cloud replication of the local SQLite file.

## Decisions

### 1. Embedded SQLite with Bundled Engine
- **Decision:** Use `rusqlite` with `features = ["bundled"]`.
- **Rationale:** Compiles a standalone SQLite C amalgam into the Rust binary, ensuring 100% portability across macOS, Linux, and Windows without requiring system `libsqlite3`.

### 2. Startup Pipeline
- **Decision:** Load cached tracks synchronously from SQLite on startup, then trigger background provider sync to discover new/modified files.
- **Rationale:** Provides zero perceived latency for the user—UI shows library in under 50ms, while background worker updates in the background.

## Risks / Trade-offs

- **[Risk]** Database schema evolution across app versions.
  → **Mitigation**: Use `CREATE TABLE IF NOT EXISTS` with automated migration guards.
