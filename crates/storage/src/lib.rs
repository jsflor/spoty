pub mod demo;
pub mod opendal_provider;
pub mod provider;
pub mod subsonic;

pub use demo::DemoProvider;
pub use opendal_provider::{OpenDalProvider, S3Config};
pub use provider::{AudioStream, Library, MusicProvider};
pub use subsonic::{SubsonicConfig, SubsonicProvider};
