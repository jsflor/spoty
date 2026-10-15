use crate::db::LibraryDatabase;
use crate::provider::{Library, MusicProvider};
use spoty_core::{Playlist, PlaylistId, Track, TrackId};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, RwLock};

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
    /// Opens or creates the SQLite database and initializes the in-memory catalog and favorites index.
    pub fn open(db_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = LibraryDatabase::open_or_create(db_path)?;
        let library = db.load_library().unwrap_or_default();
        let fav_list = db.get_favorites().unwrap_or_default();
        let favorites = fav_list.into_iter().collect();

        Ok(Self {
            inner: Arc::new(RwLock::new(CatalogInner { library, favorites })),
            db,
        })
    }

    /// Opens an ephemeral in-memory catalog (primarily for unit tests).
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let db = LibraryDatabase::open_in_memory()?;
        let library = db.load_library().unwrap_or_default();
        let fav_list = db.get_favorites().unwrap_or_default();
        let favorites = fav_list.into_iter().collect();

        Ok(Self {
            inner: Arc::new(RwLock::new(CatalogInner { library, favorites })),
            db,
        })
    }

    /// Returns a snapshot of the current in-memory library (tracks, playlists, albums, artists).
    pub fn library(&self) -> Library {
        self.inner.read().unwrap().library.clone()
    }

    /// O(1) in-memory favorite check without hitting SQLite.
    pub fn is_favorite(&self, track_id: &TrackId) -> bool {
        self.inner.read().unwrap().favorites.contains(track_id)
    }

    /// Returns all favorited track IDs currently cached in memory.
    pub fn get_favorites(&self) -> Vec<TrackId> {
        self.inner
            .read()
            .unwrap()
            .favorites
            .iter()
            .cloned()
            .collect()
    }

    /// Retrieves a playlist by ID from the in-memory cache.
    pub fn get_playlist(&self, id: &PlaylistId) -> Option<Playlist> {
        self.inner
            .read()
            .unwrap()
            .library
            .playlists
            .iter()
            .find(|p| &p.id == id)
            .cloned()
    }

    /// Atomically toggles the favorite status of a track in SQLite and the in-memory index.
    pub fn toggle_favorite(&self, track_id: &TrackId) -> anyhow::Result<bool> {
        let favorited = self.db.toggle_favorite(track_id)?;
        let mut inner = self.inner.write().unwrap();
        if favorited {
            inner.favorites.insert(track_id.clone());
        } else {
            inner.favorites.remove(track_id);
        }
        Ok(favorited)
    }

    /// Atomically creates a playlist in SQLite and registers it in the in-memory library.
    pub fn create_playlist(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> anyhow::Result<Playlist> {
        let playlist = self.db.create_playlist(title, description)?;
        let mut inner = self.inner.write().unwrap();
        inner.library.playlists.push(playlist.clone());
        Ok(playlist)
    }

    /// Atomically renames a playlist in SQLite and updates the in-memory representation.
    pub fn rename_playlist(
        &self,
        id: &PlaylistId,
        title: &str,
        description: Option<&str>,
    ) -> anyhow::Result<()> {
        self.db.rename_playlist(id, title, description)?;
        let mut inner = self.inner.write().unwrap();
        if let Some(pl) = inner.library.playlists.iter_mut().find(|p| &p.id == id) {
            pl.title = title.to_string();
            if let Some(desc) = description {
                pl.description = Some(desc.to_string());
            }
        }
        Ok(())
    }

    /// Atomically deletes a playlist from SQLite and the in-memory library.
    pub fn delete_playlist(&self, id: &PlaylistId) -> anyhow::Result<()> {
        self.db.delete_playlist(id)?;
        let mut inner = self.inner.write().unwrap();
        inner.library.playlists.retain(|p| &p.id != id);
        Ok(())
    }

    /// Atomically associates a track with a playlist in SQLite and the in-memory representation.
    pub fn add_track_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> anyhow::Result<()> {
        self.db.add_track_to_playlist(playlist_id, track_id)?;
        let mut inner = self.inner.write().unwrap();
        if let Some(pl) = inner
            .library
            .playlists
            .iter_mut()
            .find(|p| &p.id == playlist_id)
        {
            if !pl.track_ids.contains(track_id) {
                pl.track_ids.push(track_id.clone());
            }
        }
        Ok(())
    }

    /// Atomically removes a track from a playlist in SQLite and the in-memory representation.
    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> anyhow::Result<()> {
        self.db.remove_track_from_playlist(playlist_id, track_id)?;
        let mut inner = self.inner.write().unwrap();
        if let Some(pl) = inner
            .library
            .playlists
            .iter_mut()
            .find(|p| &p.id == playlist_id)
        {
            pl.track_ids.retain(|t| t != track_id);
        }
        Ok(())
    }

    /// Upserts tracks into SQLite and updates the in-memory catalog snapshot.
    pub fn upsert_tracks(&self, tracks: &[Track]) -> anyhow::Result<()> {
        self.db.upsert_tracks(tracks)?;
        let mut inner = self.inner.write().unwrap();
        for t in tracks {
            if let Some(existing) = inner.library.tracks.iter_mut().find(|trk| trk.id == t.id) {
                *existing = t.clone();
            } else {
                inner.library.tracks.push(t.clone());
            }
        }
        inner.library.aggregate_artists_and_albums();
        Ok(())
    }

    /// Fetches library from the specified provider, persists tracks to SQLite,
    /// preserves custom local playlists, and atomically updates the in-memory cache.
    pub async fn sync_provider(&self, provider: &dyn MusicProvider) -> anyhow::Result<()> {
        let mut new_lib = provider.fetch_library().await?;
        self.db.upsert_tracks(&new_lib.tracks)?;

        // Preserve custom playlists from SQLite
        if let Ok(db_lib) = self.db.load_library() {
            for pl in db_lib.playlists {
                if !new_lib.playlists.iter().any(|p| p.id == pl.id) {
                    new_lib.playlists.push(pl);
                }
            }
        }

        new_lib.aggregate_artists_and_albums();

        let mut inner = self.inner.write().unwrap();
        inner.library = new_lib;
        Ok(())
    }

    /// Returns a reference to the underlying `LibraryDatabase` if direct DAO access is needed.
    pub fn db(&self) -> &LibraryDatabase {
        &self.db
    }
}
