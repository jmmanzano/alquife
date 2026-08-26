//! Audio sink enumeration and routing via PulseAudio/PipeWire (pactl) on Linux.
//! On Windows, enumerates via PowerShell (future).

use std::process::Command;
use tracing::{debug, warn};

/// Represents a system audio output sink
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSink {
    /// PulseAudio/PipeWire internal sink name (used for routing)
    pub name: String,
    /// Human-readable description shown to the user
    pub description: String,
    /// Whether this sink is a Bluetooth device
    pub is_bluetooth: bool,
    /// Current sink state (Running, Suspended, etc.)
    pub state: String,
}

impl AudioSink {
    fn is_bluetooth_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with("bluez_") || lower.contains("bluetooth")
    }
}

/// List all available audio output sinks from the system.
/// Uses `pactl list sinks` on Linux to get name + description.
pub fn list_sinks() -> Vec<AudioSink> {
    #[cfg(target_os = "linux")]
    {
        list_sinks_linux()
    }
    #[cfg(target_os = "windows")]
    {
        list_sinks_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// Get the PulseAudio sink-input index for the current process (by PID).
/// Used to move the stream to a different sink via `pactl move-sink-input`.
pub fn get_own_sink_input_index() -> Option<u32> {
    let own_pid = std::process::id();
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    let mut current_index: Option<u32> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Sink Input #") {
            current_index = rest.trim().parse().ok();
        }
        // Match both "application.process.id" and localized variants
        if trimmed.contains("application.process.id") {
            if let Some(val) = extract_quoted_value(trimmed) {
                if val.parse::<u32>().ok() == Some(own_pid) {
                    return current_index;
                }
            }
        }
    }
    None
}

/// Route the audio stream identified by `sink_input_index` to `sink_name`.
pub fn move_sink_input(sink_input_index: u32, sink_name: &str) -> Result<(), String> {
    let output = Command::new("pactl")
        .args(["move-sink-input", &sink_input_index.to_string(), sink_name])
        .output()
        .map_err(|e| format!("pactl not available: {}", e))?;

    if output.status.success() {
        debug!("Moved sink-input {} to sink '{}'", sink_input_index, sink_name);
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("pactl move-sink-input failed: {}", err))
    }
}

/// Set the default PulseAudio/PipeWire sink.
/// Alquife's new streams will start on this sink after calling this.
pub fn set_default_sink(sink_name: &str) -> Result<(), String> {
    let output = Command::new("pactl")
        .args(["set-default-sink", sink_name])
        .output()
        .map_err(|e| format!("pactl not available: {}", e))?;

    if output.status.success() {
        debug!("Default sink set to '{}'", sink_name);
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("pactl set-default-sink failed: {}", err))
    }
}

// ─── Linux implementation ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn list_sinks_linux() -> Vec<AudioSink> {
    let output = match Command::new("pactl").args(["list", "sinks"]).output() {
        Ok(o) => o,
        Err(e) => {
            warn!("Failed to run pactl: {}", e);
            return Vec::new();
        }
    };

    let text = String::from_utf8_lossy(&output.stdout);
    parse_pactl_sinks(&text)
}

fn parse_pactl_sinks(text: &str) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut state = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // New sink block
        if trimmed.starts_with("Sink #") || trimmed.starts_with("Destino #") {
            // Flush previous
            if let (Some(n), Some(d)) = (name.take(), description.take()) {
                let is_bt = AudioSink::is_bluetooth_name(&n);
                sinks.push(AudioSink { name: n, description: d, is_bluetooth: is_bt, state: state.clone() });
            }
            state.clear();
            continue;
        }

        // State line (localized: "Estado:" or English "State:")
        if let Some(val) = strip_field(trimmed, &["State:", "Estado:"]) {
            state = val.to_string();
            continue;
        }

        // Name line
        if let Some(val) = strip_field(trimmed, &["Name:", "Nombre:"]) {
            name = Some(val.to_string());
            continue;
        }

        // Description line (first occurrence = the display name)
        if description.is_none() {
            if let Some(val) = strip_field(trimmed, &["Description:", "Descripción:", "Descripcion:"]) {
                description = Some(val.to_string());
            }
        }
    }

    // Flush last
    if let (Some(n), Some(d)) = (name, description) {
        let is_bt = AudioSink::is_bluetooth_name(&n);
        sinks.push(AudioSink { name: n, description: d, is_bluetooth: is_bt, state });
    }

    debug!("Enumerated {} audio sinks", sinks.len());
    sinks
}

fn strip_field<'a>(line: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for prefix in prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

fn extract_quoted_value(line: &str) -> Option<&str> {
    // Handles: key = "value"
    let start = line.find('"')? + 1;
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

// ─── Windows stub ────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn list_sinks_windows() -> Vec<AudioSink> {
    // On Windows, use CPAL to enumerate WDM/WASAPI devices since pactl is not available.
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    host.output_devices()
        .map(|devices| {
            devices
                .filter_map(|d| {
                    let n = d.name().ok()?;
                    let is_bt = n.to_lowercase().contains("bluetooth")
                        || n.to_lowercase().contains("a2dp");
                    Some(AudioSink {
                        description: n.clone(),
                        name: n,
                        is_bluetooth: is_bt,
                        state: "Unknown".to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pactl_output() {
        let sample = r#"
Sink #101
	State: RUNNING
	Name: alsa_output.pci-0000_00_1f.3.hdmi-stereo
	Description: Internal Audio Digital Stereo (HDMI)

Sink #1946
	State: SUSPENDED
	Name: bluez_output.7C_D5_66_51_36_DB.1
	Description: Echo Dot-6Q2
"#;
        let sinks = parse_pactl_sinks(sample);
        assert_eq!(sinks.len(), 2);
        assert_eq!(sinks[0].description, "Internal Audio Digital Stereo (HDMI)");
        assert!(!sinks[0].is_bluetooth);
        assert_eq!(sinks[1].description, "Echo Dot-6Q2");
        assert!(sinks[1].is_bluetooth);
        assert_eq!(sinks[1].state, "SUSPENDED");
    }
}
