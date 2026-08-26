//! Theme color definitions — file-based themes loaded from ~/.config/alquife/themes/

use std::path::Path;

use ratatui::style::Color;
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::config::paths;

/// Color palette for a theme
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    /// Primary highlight color (focused elements, selected tabs)
    pub primary: Color,
    /// Secondary color (borders, less important elements)
    pub secondary: Color,
    /// Accent color (currently playing, important highlights)
    pub accent: Color,
    /// Artist names
    pub artist: Color,
    /// Album names
    pub album: Color,
    /// Song titles (default)
    pub song: Color,
    /// Muted text (track numbers, durations, hints)
    pub muted: Color,
    /// Selection/highlight background
    pub highlight_bg: Color,
    /// Text on highlighted background
    pub highlight_fg: Color,
    /// Success messages
    pub success: Color,
    /// Error messages
    pub error: Color,
    /// Playing indicator
    pub playing: Color,
    /// Played songs in queue
    pub played: Color,
    /// Border color (focused)
    pub border_focused: Color,
    /// Border color (unfocused)
    pub border_unfocused: Color,
}

/// A loaded theme: display name + colors
#[derive(Debug, Clone)]
pub struct ThemeData {
    /// Display name (e.g. "Catppuccin", "Default")
    pub name: String,
    /// UI colors
    pub colors: ThemeColors,
}

// ── TOML deserialization structs ──────────────────────────────────────────────

#[derive(Deserialize)]
struct ThemeFile {
    colors: ThemeFileColors,
}

#[derive(Deserialize)]
struct ThemeFileColors {
    primary: String,
    secondary: String,
    accent: String,
    artist: String,
    album: String,
    song: String,
    muted: String,
    highlight_bg: String,
    highlight_fg: String,
    success: String,
    error: String,
    playing: String,
    played: String,
    border_focused: String,
    border_unfocused: String,
}

// ── Hex color parsing ─────────────────────────────────────────────────────────

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    warn!("Invalid hex color '{}', falling back to white", hex);
    Color::White
}

// ── ThemeData construction ────────────────────────────────────────────────────

impl ThemeData {
    fn from_file_content(name: &str, content: &str) -> Result<Self, String> {
        let file: ThemeFile =
            toml::from_str(content).map_err(|e| format!("Failed to parse theme '{}': {}", name, e))?;

        let c = &file.colors;
        let colors = ThemeColors {
            primary: hex_to_color(&c.primary),
            secondary: hex_to_color(&c.secondary),
            accent: hex_to_color(&c.accent),
            artist: hex_to_color(&c.artist),
            album: hex_to_color(&c.album),
            song: hex_to_color(&c.song),
            muted: hex_to_color(&c.muted),
            highlight_bg: hex_to_color(&c.highlight_bg),
            highlight_fg: hex_to_color(&c.highlight_fg),
            success: hex_to_color(&c.success),
            error: hex_to_color(&c.error),
            playing: hex_to_color(&c.playing),
            played: hex_to_color(&c.played),
            border_focused: hex_to_color(&c.border_focused),
            border_unfocused: hex_to_color(&c.border_unfocused),
        };

        Ok(ThemeData {
            name: name.to_string(),
            colors,
        })
    }

    /// The hardcoded Default theme
    pub fn default_theme() -> Self {
        ThemeData {
            name: "Default".to_string(),
            colors: ThemeColors {
                primary: Color::Cyan,
                secondary: Color::DarkGray,
                accent: Color::Yellow,
                artist: Color::LightGreen,
                album: Color::Magenta,
                song: Color::Magenta,
                muted: Color::Gray,
                highlight_bg: Color::Rgb(102, 51, 153),
                highlight_fg: Color::White,
                success: Color::Green,
                error: Color::Red,
                playing: Color::LightGreen,
                played: Color::Red,
                border_focused: Color::Cyan,
                border_unfocused: Color::DarkGray,
            },
        }
    }
}

// ── Loading ───────────────────────────────────────────────────────────────────

/// Load all themes: Default (hardcoded) + TOML files from themes dir (sorted alphabetically)
pub fn load_themes() -> Vec<ThemeData> {
    let mut themes = vec![ThemeData::default_theme()];

    if let Some(dir) = paths::themes_dir() {
        if dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "toml")
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                // Capitalize first letter for display name
                let name = titlecase_filename(stem);

                match std::fs::read_to_string(&path) {
                    Ok(content) => match ThemeData::from_file_content(&name, &content) {
                        Ok(theme) => {
                            info!("Loaded theme '{}' from {}", name, path.display());
                            themes.push(theme);
                        }
                        Err(e) => error!("{}", e),
                    },
                    Err(e) => error!("Failed to read {}: {}", path.display(), e),
                }
            }
        }
    }

    themes
}

/// Convert a filename stem like "tokyo-night" or "rose_pine" to "Tokyo Night" or "Rose Pine"
fn titlecase_filename(s: &str) -> String {
    s.split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Seeding built-in themes ───────────────────────────────────────────────────

/// Write the built-in themes as TOML files into the given directory.
/// Only writes files that don't already exist.
pub fn seed_default_themes(dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        error!("Failed to create themes directory: {}", e);
        return;
    }

    for (filename, content) in BUILTIN_THEMES {
        let path = dir.join(filename);
        if !path.exists() {
            if let Err(e) = std::fs::write(&path, content) {
                error!("Failed to write theme {}: {}", filename, e);
            } else {
                info!("Seeded theme file: {}", filename);
            }
        }
    }
}

use super::theme_builtins::BUILTIN_THEMES;
