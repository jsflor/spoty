use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

macro_rules! define_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

define_id!(TrackId, "track");
define_id!(AlbumId, "album");
define_id!(ArtistId, "artist");
define_id!(PlaylistId, "playlist");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_id_display_and_equality() {
        let id1 = TrackId::new("trk_123");
        let id2 = TrackId::from("trk_123");
        assert_eq!(id1, id2);
        assert_eq!(id1.to_string(), "trk_123");
        assert_eq!(id1.as_str(), "trk_123");
    }

    #[test]
    fn test_track_id_serialization() {
        let id = TrackId::new("song_42");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"song_42\"");

        let deserialized: TrackId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }
}
