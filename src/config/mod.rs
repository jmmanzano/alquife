//! Configuration loading and management

pub mod paths;
pub mod equalizer;

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

use crate::error::ConfigError;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Subsonic server base URL
    #[serde(rename = "BaseURL", default)]
    pub base_url: String,

    /// Username for authentication
    #[serde(rename = "Username", default)]
    pub username: String,

    /// Password for authentication
    #[serde(rename = "Password", default)]
    pub password: String,

    /// UI Theme name
    #[serde(rename = "Theme", default)]
    pub theme: String,

    /// Enable automatic queue extension with similar songs
    #[serde(rename = "NonStopMode", default)]
    pub non_stop_mode: bool,
    #[serde(rename = "AudioBackend", default)]
    pub audio_backend: String,
    #[serde(rename = "EqualizerEnabled", default)]
    pub equalizer_enabled: bool,
    #[serde(rename = "EqualizerPreset", default = "Config::default_equalizer_preset")]
    pub equalizer_preset: String,
    /// Enable desktop notifications when a song changes
    #[serde(rename = "Notifications", default)]
    pub notifications_enabled: bool,
    /// Audio output device name (e.g., "Echo Dot (Stereo)")
    #[serde(rename = "AudioOutputDevice", default)]
    pub audio_output_device: Option<String>,
    /// Enable automatic update checks
    #[serde(rename = "AutoUpdateEnabled", default = "Config::default_auto_update")]
    pub auto_update_enabled: bool,
    /// Show album artwork on the queue page.
    #[serde(rename = "ShowCoverArt", default = "Config::default_show_cover_art")]
    pub show_cover_art: bool,
    /// Maximum song cache size in GB (0 disables the cache).
    #[serde(rename = "CacheSizeGB", default = "Config::default_cache_size_gb", deserialize_with = "deserialize_cache_size")]
    pub cache_size_gb: u8,
}

fn deserialize_cache_size<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    let size = u8::deserialize(deserializer)?;
    if size > 5 {
        return Err(serde::de::Error::custom("CacheSizeGB must be between 0 and 5"));
    }
    Ok(size)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: String::new(), username: String::new(), password: String::new(),
            theme: String::new(), non_stop_mode: false, audio_backend: String::new(),
            equalizer_enabled: false, equalizer_preset: String::new(),
            notifications_enabled: false, audio_output_device: None,
            auto_update_enabled: false, show_cover_art: true, cache_size_gb: 1,
        }
    }
}

impl Config {
    fn default_equalizer_preset() -> String {
        "Flat".to_string()
    }

    fn default_auto_update() -> bool {
        true
    }

    fn default_show_cover_art() -> bool { true }

    fn default_cache_size_gb() -> u8 { 1 }

    /// Create a new empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Load config from the default location
    pub fn load_default() -> Result<Self, ConfigError> {
        let path = paths::config_file().ok_or_else(|| ConfigError::NotFound {
            path: "default config location".to_string(),
        })?;

        if path.exists() {
            Self::load_from_file(&path)
        } else {
            info!("No config file found at {}, using defaults", path.display());
            Ok(Self::new())
        }
    }

    /// Load config from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        debug!("Loading config from {}", path.display());

        if !path.exists() {
            return Err(ConfigError::NotFound {
                path: path.display().to_string(),
            });
        }

        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        debug!("Config loaded successfully");
        Ok(config)
    }

    /// Save config to the default location
    pub fn save_default(&self) -> Result<(), ConfigError> {
        let path = paths::config_file().ok_or_else(|| ConfigError::NotFound {
            path: "default config location".to_string(),
        })?;

        self.save_to_file(&path)
    }

    /// Save config to a specific file
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        debug!("Saving config to {}", path.display());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;

        info!("Config saved to {}", path.display());
        Ok(())
    }

    /// Check if the config has valid server settings
    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty() && !self.username.is_empty() && !self.password.is_empty()
    }

    /// Validate the config
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.base_url.is_empty() {
            return Err(ConfigError::MissingField {
                field: "BaseURL".to_string(),
            });
        }

        // Validate URL format
        if url::Url::parse(&self.base_url).is_err() {
            return Err(ConfigError::InvalidUrl {
                url: self.base_url.clone(),
            });
        }

        if self.username.is_empty() {
            warn!("Username is empty");
        }

        if self.password.is_empty() {
            warn!("Password is empty");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parse() {
        let toml_content = r#"
BaseURL = "https://example.com"
Username = "testuser"
Password = "testpass"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.username, "testuser");
        assert_eq!(config.password, "testpass");
        assert!(config.show_cover_art);
    }

    #[test]
    fn cover_setting_round_trips() {
        let mut config = Config::new();
        assert!(config.show_cover_art);
        config.show_cover_art = false;
        let serialized = toml::to_string(&config).unwrap();
        let loaded: Config = toml::from_str(&serialized).unwrap();
        assert!(!loaded.show_cover_art);
    }

    #[test]
    fn cache_size_defaults_and_validates_range() {
        let default: Config = toml::from_str("").unwrap();
        assert_eq!(default.cache_size_gb, 1);
        for size in 0..=5 {
            let config: Config = toml::from_str(&format!("CacheSizeGB = {size}")).unwrap();
            assert_eq!(config.cache_size_gb, size);
            let roundtrip: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
            assert_eq!(roundtrip.cache_size_gb, size);
        }
        assert!(toml::from_str::<Config>("CacheSizeGB = 6").is_err());
        assert!(toml::from_str::<Config>("CacheSizeGB = -1").is_err());
    }

    #[test]
    fn test_is_configured() {
        let mut config = Config::new();
        assert!(!config.is_configured());

        config.base_url = "https://example.com".to_string();
        config.username = "user".to_string();
        config.password = "pass".to_string();
        assert!(config.is_configured());
    }
}
