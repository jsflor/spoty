pub mod demo;
pub mod metadata;
pub mod opendal_provider;
pub mod provider;
pub mod spotify;
pub mod subsonic;

pub use demo::DemoProvider;
pub use metadata::{extract_audio_metadata, ExtractedMetadata};
pub use opendal_provider::{OpenDalProvider, S3Config};
pub use provider::{AudioStream, Library, MusicProvider};
pub use spotify::{SpotifyConfig, SpotifyProvider};
pub use subsonic::{SubsonicConfig, SubsonicProvider};
