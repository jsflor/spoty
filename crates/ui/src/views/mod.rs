pub mod home;
pub mod playlist;
pub mod search;
pub mod settings;
pub mod songs;

pub use home::HomeView;
pub use playlist::PlaylistView;
pub use search::SearchView;
pub use settings::SettingsView;
pub use songs::{SongsMode, SongsView};
