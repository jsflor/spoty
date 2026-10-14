use serde::{Deserialize, Serialize};
use spoty_storage::{S3Config, SpotifyConfig, SubsonicConfig};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    #[default]
    Demo,
    LocalFs,
    S3,
    Subsonic,
    Spotify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotyConfig {
    pub active_provider: ProviderType,
    pub local_path: String,
    pub s3: S3Config,
    pub subsonic: SubsonicConfig,
    pub spotify: SpotifyConfig,
}

impl Default for SpotyConfig {
    fn default() -> Self {
        Self {
            active_provider: ProviderType::Demo,
            local_path: String::new(),
            s3: S3Config {
                bucket: "music-bucket".into(),
                endpoint: Some("https://<account_id>.r2.cloudflarestorage.com".into()),
                region: Some("auto".into()),
                access_key_id: None,
                secret_access_key: None,
                root: Some("music/".into()),
            },
            subsonic: SubsonicConfig::new("http://raspberrypi:4533", "admin", "secret"),
            spotify: SpotifyConfig::default(),
        }
    }
}

impl SpotyConfig {
    pub fn config_path() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config").join("spoty").join("config.json")
        } else {
            PathBuf::from("spoty_config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let cfg = SpotyConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: SpotyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.active_provider, deserialized.active_provider);
        assert_eq!(cfg.s3.bucket, deserialized.s3.bucket);
        assert_eq!(cfg.spotify.device_name, deserialized.spotify.device_name);
    }
}
