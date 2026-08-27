//! Auto-update functionality for Alquife
//!
//! Checks for new releases on GitHub and allows users to update the binary.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use semver::Version;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const GITHUB_API_URL: &str = "https://api.github.com/repos/jmmanzano/alquife/releases/latest";

/// GitHub Release API response
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub Asset (downloadable file)
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Update check state
#[derive(Debug, Serialize, Deserialize)]
struct UpdateState {
    /// Last checked version
    last_checked_version: String,
    /// Timestamp of last check
    last_check_time: i64,
}

impl UpdateState {
    fn file_path() -> Result<PathBuf> {
        let config_dir = crate::config::paths::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join(".update_check"))
    }

    fn load() -> Result<Option<Self>> {
        match Self::file_path() {
            Ok(path) => {
                if path.exists() {
                    let content = fs::read_to_string(&path)?;
                    Ok(Some(serde_json::from_str(&content)?))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    fn save(&self) -> Result<()> {
        if let Ok(path) = Self::file_path() {
            let content = serde_json::to_string(self)?;
            let mut file = fs::File::create(path)?;
            file.write_all(content.as_bytes())?;
        }
        Ok(())
    }
}

/// Check for updates and prompt user
pub async fn check_for_update(current_version: &str, auto_update_enabled: bool) -> Result<()> {
    // Skip if auto-update is disabled
    if !auto_update_enabled {
        return Ok(());
    }

    // Parse current version
    let current = Version::parse(current_version)
        .with_context(|| format!("Invalid current version: {}", current_version))?;

    // Check state to avoid spamming
    let mut should_check = true;
    if let Ok(Some(state)) = UpdateState::load() {
        let now = chrono::Local::now().timestamp();
        // Only check once per 24 hours
        if now - state.last_check_time < 86400 {
            should_check = false;
        }
    }

    if !should_check {
        return Ok(());
    }

    match fetch_latest_release().await {
        Ok(release) => {
            match parse_version(&release.tag_name) {
                Ok(latest) => {
                    let state = UpdateState {
                        last_checked_version: release.tag_name.clone(),
                        last_check_time: chrono::Local::now().timestamp(),
                    };
                    let _ = state.save();

                    if latest > current {
                        info!(
                            "New version available: {} -> {}",
                            current_version, release.tag_name
                        );
                        if should_update(&release.tag_name)? {
                            match download_and_update(&release).await {
                                Ok(_) => {
                                    info!("Update completed successfully");
                                }
                                Err(e) => {
                                    warn!("Update failed: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Could not parse latest version: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("Could not check for updates: {}", e);
            // Silently fail - don't block app startup
        }
    }

    Ok(())
}

/// Fetch latest release info from GitHub
async fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::new();
    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", "alquife")
        .send()
        .await?;

    let release: GitHubRelease = response.json().await?;
    Ok(release)
}

/// Parse version from GitHub tag (e.g., "v0.0.2" -> "0.0.2")
fn parse_version(tag: &str) -> Result<Version> {
    let version_str = tag.trim_start_matches('v');
    Version::parse(version_str).context("Invalid version format")
}

/// Prompt user for update
fn should_update(version: &str) -> Result<bool> {
    println!("\n╔════════════════════════════════════╗");
    println!("║  New version available: {}          ║", version);
    println!("║  Would you like to update?         ║");
    println!("╚════════════════════════════════════╝");
    print!("Update now? (y/n): ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

/// Download and install the update
async fn download_and_update(release: &GitHubRelease) -> Result<()> {
    // Determine the correct asset for this platform
    let asset = find_asset_for_platform(&release.assets)?;

    let exe_path = std::env::current_exe()?;
    let temp_dir = tempfile::tempdir()?;
    let zip_path = temp_dir.path().join("alquife.zip");

    println!("Downloading update...");
    download_file(&asset.browser_download_url, &zip_path).await?;

    println!("Extracting update...");
    extract_zip(&zip_path, temp_dir.path())?;

    println!("Installing update...");
    install_update(&exe_path, temp_dir.path())?;

    println!("Update completed! Please restart the application.");
    Ok(())
}

/// Find the correct asset for the current platform
fn find_asset_for_platform(assets: &[GitHubAsset]) -> Result<&GitHubAsset> {
    #[cfg(target_os = "windows")]
    {
        assets
            .iter()
            .find(|a| a.name.ends_with("windows.zip") || a.name == "alquife.exe")
            .ok_or_else(|| anyhow::anyhow!("No Windows binary found in release"))
    }

    #[cfg(target_os = "linux")]
    {
        assets
            .iter()
            .find(|a| a.name.ends_with("linux") || a.name.ends_with("linux.tar.gz"))
            .ok_or_else(|| anyhow::anyhow!("No Linux binary found in release"))
    }

    #[cfg(target_os = "macos")]
    {
        assets
            .iter()
            .find(|a| a.name.ends_with("macos") || a.name.ends_with("macos.zip"))
            .ok_or_else(|| anyhow::anyhow!("No macOS binary found in release"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

/// Download file from URL
async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    fs::write(dest, bytes)?;
    Ok(())
}

/// Extract ZIP file
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Install the new binary
#[cfg(target_os = "windows")]
fn install_update(current_exe: &Path, temp_dir: &Path) -> Result<()> {
    // Find the new executable in temp directory
    let new_exe = find_exe_in_dir(temp_dir)?;

    // Create a backup
    let backup_path = current_exe.with_extension("exe.bak");
    fs::copy(current_exe, &backup_path)?;

    // Replace the executable
    fs::copy(&new_exe, current_exe)?;

    info!("Update installed. Backup saved to: {}", backup_path.display());
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_update(current_exe: &Path, temp_dir: &Path) -> Result<()> {
    let new_exe = find_exe_in_dir(temp_dir)?;

    // Create a backup
    let backup_path = current_exe.with_extension("bak");
    fs::copy(current_exe, &backup_path)?;

    // Make executable
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&new_exe, fs::Permissions::from_mode(0o755))?;

    // Replace the executable
    fs::copy(&new_exe, current_exe)?;
    fs::set_permissions(current_exe, fs::Permissions::from_mode(0o755))?;

    info!("Update installed. Backup saved to: {}", backup_path.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_update(current_exe: &Path, temp_dir: &Path) -> Result<()> {
    let new_exe = find_exe_in_dir(temp_dir)?;

    // Create a backup
    let backup_path = current_exe.with_extension("bak");
    fs::copy(current_exe, &backup_path)?;

    // Make executable
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&new_exe, fs::Permissions::from_mode(0o755))?;

    // Replace the executable
    fs::copy(&new_exe, current_exe)?;
    fs::set_permissions(current_exe, fs::Permissions::from_mode(0o755))?;

    info!("Update installed. Backup saved to: {}", backup_path.display());
    Ok(())
}

/// Find executable in directory
fn find_exe_in_dir(dir: &Path) -> Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            #[cfg(target_os = "windows")]
            {
                if path.extension().and_then(|s| s.to_str()) == Some("exe") {
                    return Ok(path);
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy() == "alquife" {
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Executable not found in extracted files"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(
            parse_version("v0.0.2").unwrap(),
            Version::parse("0.0.2").unwrap()
        );
        assert_eq!(
            parse_version("0.0.2").unwrap(),
            Version::parse("0.0.2").unwrap()
        );
    }
}
