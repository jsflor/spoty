use serde::{Deserialize, Serialize};
use crate::id::{AlbumId, ArtistId, PlaylistId, TrackId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    #[default]
    Unknown,
    Mp3,
    Flac,
    Wav,
    Ogg,
    Aac,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "flac" => Self::Flac,
            "wav" => Self::Wav,
            "ogg" | "oga" => Self::Ogg,
            "aac" | "m4a" => Self::Aac,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_secs: u32,
    pub cover_uri: Option<String>,
    pub source_uri: String,
    pub format: AudioFormat,
}

impl Track {
    pub fn formatted_duration(&self) -> String {
        let minutes = self.duration_secs / 60;
        let seconds = self.duration_secs % 60;
        format!("{}:{:02}", minutes, seconds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub cover_uri: Option<String>,
    pub track_ids: Vec<TrackId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub avatar_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub title: String,
    pub description: Option<String>,
    pub cover_uri: Option<String>,
    pub track_ids: Vec<TrackId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackStatus {
    #[default]
    Stopped,
    Playing,
    Paused,
    Buffering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_track: Option<Track>,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f32,
    pub is_muted: bool,
    pub is_shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            current_track: None,
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: 0.8,
            is_muted: false,
            is_shuffle: false,
            repeat_mode: RepeatMode::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_duration_format() {
        let track = Track {
            id: TrackId::new("trk_1"),
            title: "Midnight City".into(),
            artist: "M83".into(),
            album: Some("Hurry Up, We're Dreaming".into()),
            duration_secs: 244,
            cover_uri: None,
            source_uri: "local://test.mp3".into(),
            format: AudioFormat::Mp3,
        };
        assert_eq!(track.formatted_duration(), "4:04");
    }

    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert_eq!(state.status, PlaybackStatus::Stopped);
        assert_eq!(state.volume, 0.8);
        assert!(!state.is_muted);
        assert!(!state.is_shuffle);
        assert_eq!(state.repeat_mode, RepeatMode::Off);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let track = Track {
            id: TrackId::new("trk_roundtrip"),
            title: "Song Title".into(),
            artist: "Artist Name".into(),
            album: None,
            duration_secs: 180,
            cover_uri: Some("https://example.com/cover.jpg".into()),
            source_uri: "demo://track1".into(),
            format: AudioFormat::Flac,
        };

        let json = serde_json::to_string(&track).unwrap();
        let decoded: Track = serde_json::from_str(&json).unwrap();
        assert_eq!(track, decoded);
    }
}
