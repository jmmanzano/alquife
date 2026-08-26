//! Platform-specific configuration paths

use std::path::PathBuf;

/// Get the default config directory for alquife
/// - Windows: %LOCALAPPDATA%\Alquife
/// - Linux: ~/.config/alquife
/// - macOS: ~/Library/Application Support/alquife
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("Alquife"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::config_dir().map(|p| p.join("alquife"))
    }
}

/// Get the default config file path
pub fn config_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("config.toml"))
}

/// Get the themes directory path
pub fn themes_dir() -> Option<PathBuf> {
    config_dir().map(|p| p.join("themes"))
}

/// Get the equalizer presets directory path
pub fn equalizer_presets_dir() -> Option<PathBuf> {
    config_dir().map(|p| p.join("equalizer"))
}

/// Get the log file path
#[allow(dead_code)]
pub fn log_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("alquife.log"))
}

/// Get persisted queue snapshot file path
pub fn queue_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("queue.json"))
}

/// Get the UI state snapshot file path
pub fn ui_state_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("ui_state.json"))
}

/// Ensure the config directory exists
#[allow(dead_code)]
pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let dir = config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        )
    })?;

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}
