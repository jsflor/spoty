use crate::provider::Library;
use rusqlite::{Connection, params};
use spoty_core::{AudioFormat, Playlist, PlaylistId, Track, TrackId};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LibraryDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl LibraryDatabase {
    /// Opens or creates the SQLite database at the specified filesystem path
    pub fn open_or_create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// Opens an ephemeral in-memory SQLite database (primarily for unit tests)
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                duration_secs INTEGER NOT NULL,
                cover_uri TEXT,
                source_uri TEXT NOT NULL,
                format TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                cover_uri TEXT
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id)
            );

            CREATE TABLE IF NOT EXISTS favorites (
                track_id TEXT PRIMARY KEY,
                added_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )?;

        Ok(())
    }

    /// Inserts or updates tracks into the local database
    pub fn upsert_tracks(&self, tracks: &[Track]) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO tracks (id, title, artist, album, duration_secs, cover_uri, source_uri, format)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    artist=excluded.artist,
                    album=excluded.album,
                    duration_secs=excluded.duration_secs,
                    cover_uri=excluded.cover_uri,
                    source_uri=excluded.source_uri,
                    format=excluded.format
                "#,
            )?;

            for t in tracks {
                let fmt_str = serde_json::to_string(&t.format)
                    .unwrap_or_default()
                    .replace('"', "");
                stmt.execute(params![
                    t.id.as_str(),
                    t.title,
                    t.artist,
                    t.album,
                    t.duration_secs,
                    t.cover_uri,
                    t.source_uri,
                    fmt_str,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Loads the complete cached library from SQLite
    pub fn load_library(&self) -> anyhow::Result<Library> {
        let conn = self.conn.lock().unwrap();

        // 1. Load Tracks
        let mut track_stmt = conn.prepare(
            "SELECT id, title, artist, album, duration_secs, cover_uri, source_uri, format FROM tracks",
        )?;

        let track_iter = track_stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let artist: String = row.get(2)?;
            let album: Option<String> = row.get(3)?;
            let duration_secs: u32 = row.get(4)?;
            let cover_uri: Option<String> = row.get(5)?;
            let source_uri: String = row.get(6)?;
            let fmt_str: String = row.get(7)?;

            let format = AudioFormat::from_extension(&fmt_str);

            Ok(Track {
                id: TrackId::new(id_str),
                title,
                artist,
                album,
                duration_secs,
                cover_uri,
                source_uri,
                format,
            })
        })?;

        let mut tracks = Vec::new();
        for t in track_iter {
            tracks.push(t?);
        }

        // 2. Load Playlists
        let mut pl_stmt =
            conn.prepare("SELECT id, title, description, cover_uri FROM playlists")?;
        let pl_iter = pl_stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let cover_uri: Option<String> = row.get(3)?;

            Ok(Playlist {
                id: PlaylistId::new(id_str),
                title,
                description,
                cover_uri,
                track_ids: Vec::new(),
            })
        })?;

        let mut playlists = Vec::new();
        for p in pl_iter {
            let mut pl = p?;
            // Load track ids for this playlist
            let mut pt_stmt = conn.prepare(
                "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position ASC",
            )?;
            let ids = pt_stmt.query_map(params![pl.id.as_str()], |r| {
                let tid: String = r.get(0)?;
                Ok(TrackId::new(tid))
            })?;

            for id in ids {
                pl.track_ids.push(id?);
            }
            playlists.push(pl);
        }

        let mut lib = Library {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists,
        };
        lib.aggregate_artists_and_albums();
        Ok(lib)
    }

    /// Toggles the favorite status for a track, returning true if now favorited
    pub fn toggle_favorite(&self, track_id: &TrackId) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE track_id = ?1)",
            params![track_id.as_str()],
            |row| row.get(0),
        )?;

        if exists {
            conn.execute(
                "DELETE FROM favorites WHERE track_id = ?1",
                params![track_id.as_str()],
            )?;
            Ok(false)
        } else {
            conn.execute(
                "INSERT INTO favorites (track_id) VALUES (?1)",
                params![track_id.as_str()],
            )?;
            Ok(true)
        }
    }

    pub fn is_favorite(&self, track_id: &TrackId) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE track_id = ?1)",
            params![track_id.as_str()],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn get_favorites(&self) -> anyhow::Result<Vec<TrackId>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT track_id FROM favorites ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(TrackId::new(id))
        })?;

        let mut favs = Vec::new();
        for r in rows {
            favs.push(r?);
        }
        Ok(favs)
    }

    pub fn create_playlist(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> anyhow::Result<Playlist> {
        let conn = self.conn.lock().unwrap();
        let pl_id = format!(
            "custom_pl_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );

        conn.execute(
            "INSERT INTO playlists (id, title, description, cover_uri) VALUES (?1, ?2, ?3, NULL)",
            params![&pl_id, title, description],
        )?;

        Ok(Playlist {
            id: PlaylistId::new(pl_id),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            cover_uri: None,
            track_ids: Vec::new(),
        })
    }

    pub fn add_track_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let next_pos: u32 = conn.query_row(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id.as_str()],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id.as_str(), track_id.as_str(), next_pos],
        )?;

        Ok(())
    }

    /// Deletes a playlist and all its associated track entries
    pub fn delete_playlist(&self, playlist_id: &PlaylistId) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id.as_str()],
        )?;
        tx.execute(
            "DELETE FROM playlists WHERE id = ?1",
            params![playlist_id.as_str()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Removes a single track from a playlist
    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id.as_str(), track_id.as_str()],
        )?;
        Ok(())
    }

    /// Renames a playlist and optionally updates its description
    pub fn rename_playlist(
        &self,
        playlist_id: &PlaylistId,
        new_title: &str,
        new_description: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE playlists SET title = ?1, description = COALESCE(?2, description) WHERE id = ?3",
            params![new_title, new_description, playlist_id.as_str()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_crud_and_favorites() {
        let db = LibraryDatabase::open_in_memory().unwrap();

        let track1 = Track {
            id: TrackId::new("trk_db_1"),
            title: "Database Anthem".into(),
            artist: "Rustacean".into(),
            album: Some("Zero Cost".into()),
            duration_secs: 195,
            cover_uri: None,
            source_uri: "local://song1.mp3".into(),
            format: AudioFormat::Mp3,
        };

        db.upsert_tracks(&[track1.clone()]).unwrap();

        let lib = db.load_library().unwrap();
        assert_eq!(lib.tracks.len(), 1);
        assert_eq!(lib.tracks[0].title, "Database Anthem");

        // Favorites test
        assert!(!db.is_favorite(&track1.id).unwrap());
        let favorited = db.toggle_favorite(&track1.id).unwrap();
        assert!(favorited);
        assert!(db.is_favorite(&track1.id).unwrap());

        let unfavorited = db.toggle_favorite(&track1.id).unwrap();
        assert!(!unfavorited);
        assert!(!db.is_favorite(&track1.id).unwrap());

        // Custom playlist test
        let playlist = db
            .create_playlist("My Chill Mix", Some("Relaxing tunes"))
            .unwrap();
        db.add_track_to_playlist(&playlist.id, &track1.id).unwrap();

        let updated_lib = db.load_library().unwrap();
        assert_eq!(updated_lib.playlists.len(), 1);
        assert_eq!(updated_lib.playlists[0].title, "My Chill Mix");
        assert_eq!(updated_lib.playlists[0].track_ids.len(), 1);
        assert_eq!(updated_lib.playlists[0].track_ids[0], track1.id);

        // Rename playlist test
        db.rename_playlist(&playlist.id, "Super Chill Mix", Some("Updated description"))
            .unwrap();
        let renamed_lib = db.load_library().unwrap();
        assert_eq!(renamed_lib.playlists[0].title, "Super Chill Mix");
        assert_eq!(
            renamed_lib.playlists[0].description.as_deref(),
            Some("Updated description")
        );

        // Remove track from playlist test
        db.remove_track_from_playlist(&playlist.id, &track1.id)
            .unwrap();
        let track_removed_lib = db.load_library().unwrap();
        assert_eq!(track_removed_lib.playlists[0].track_ids.len(), 0);

        // Delete playlist test
        db.add_track_to_playlist(&playlist.id, &track1.id).unwrap();
        db.delete_playlist(&playlist.id).unwrap();
        let deleted_lib = db.load_library().unwrap();
        assert_eq!(deleted_lib.playlists.len(), 0);
    }
}
