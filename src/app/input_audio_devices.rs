use crossterm::event::{self, KeyCode};

use crate::audio::{list_sinks, move_sink_input, get_own_sink_input_index};
use crate::app::state::AudioOutputMode;
use crate::error::Error;

use super::*;

impl App {
    /// Handle audio devices page keys
    pub(super) async fn handle_audio_devices_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        let mut sink_to_apply: Option<Option<String>> = None; // None = system default

        {
            let mut state = self.state.write().await;
            let ads = &mut state.audio_devices_state;

            match key.code {
                // Switch focus between mode pane and device list
                KeyCode::Tab | KeyCode::Right => {
                    if ads.focus == 0 {
                        ads.focus = 1;
                        ads.selected_index = 0;
                        ads.scroll_offset = 0;
                    } else {
                        ads.focus = 0;
                    }
                }
                KeyCode::Left => {
                    if ads.focus == 1 {
                        ads.focus = 0;
                    }
                }

                KeyCode::Up | KeyCode::Char('k') => {
                    if ads.focus == 0 {
                        ads.mode = match ads.mode {
                            AudioOutputMode::All => AudioOutputMode::Bluetooth,
                            AudioOutputMode::Bluetooth => AudioOutputMode::Standard,
                            AudioOutputMode::Standard => AudioOutputMode::All,
                        };
                        ads.selected_index = 0;
                        ads.scroll_offset = 0;
                    } else if ads.selected_index > 0 {
                        ads.selected_index -= 1;
                        if ads.selected_index < ads.scroll_offset {
                            ads.scroll_offset = ads.selected_index;
                        }
                    }
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    if ads.focus == 0 {
                        ads.mode = match ads.mode {
                            AudioOutputMode::All => AudioOutputMode::Standard,
                            AudioOutputMode::Standard => AudioOutputMode::Bluetooth,
                            AudioOutputMode::Bluetooth => AudioOutputMode::All,
                        };
                        ads.selected_index = 0;
                        ads.scroll_offset = 0;
                    } else {
                        let count = ads.filtered_sinks().len();
                        if count > 0 && ads.selected_index < count - 1 {
                            ads.selected_index += 1;
                            if ads.selected_index >= ads.scroll_offset + 10 {
                                ads.scroll_offset = ads.selected_index.saturating_sub(9);
                            }
                        }
                    }
                }

                // Confirm device selection
                KeyCode::Enter => {
                    if ads.focus == 1 {
                        let filtered: Vec<String> = ads
                            .filtered_sinks()
                            .into_iter()
                            .map(|s| s.name.clone())
                            .collect();
                        if let Some(name) = filtered.get(ads.selected_index) {
                            sink_to_apply = Some(Some(name.clone()));
                        }
                    } else {
                        ads.focus = 1;
                        ads.selected_index = 0;
                        ads.scroll_offset = 0;
                    }
                }

                // Reset to system default
                KeyCode::Backspace | KeyCode::Delete => {
                    if ads.focus == 1 {
                        sink_to_apply = Some(None);
                    }
                }

                // Refresh sink list
                KeyCode::Char('r') => {
                    let new_sinks = list_sinks();
                    let count = new_sinks.len();
                    ads.all_sinks = new_sinks;
                    ads.selected_index = 0;
                    ads.scroll_offset = 0;
                    state.notify(format!("Found {} audio sinks", count));
                }

                _ => {}
            }
        }

        if let Some(new_sink) = sink_to_apply {
            self.apply_audio_sink(new_sink).await;
        }

        Ok(())
    }

    /// Apply an audio sink change: update config and route the active stream.
    async fn apply_audio_sink(&mut self, sink_name: Option<String>) {
        let label = sink_name.as_deref().unwrap_or("System default").to_string();

        // Save to config
        {
            let mut state = self.state.write().await;
            state.config.audio_output_device = sink_name.clone();
            if let Err(e) = state.config.save_default() {
                state.notify_error(format!("Failed to save config: {}", e));
                return;
            }
        }

        // Get current playback state (if playing)
        let current_url = self.ffmpeg.get_current_url();
        let was_playing = self.ffmpeg.is_playing();
        let position = if current_url.is_some() {
            self.ffmpeg.get_time_pos().ok()
        } else {
            None
        };

        match &sink_name {
            Some(name) => {
                // Set PULSE_SINK so all future CPAL streams from THIS process go to this sink only.
                // This does NOT affect other applications.
                std::env::set_var("PULSE_SINK", name);

                // Move the already-playing stream to the new sink immediately
                let sink = name.clone();
                let result = tokio::task::spawn_blocking(move || {
                    if let Some(input_idx) = get_own_sink_input_index() {
                        move_sink_input(input_idx, &sink)
                    } else {
                        // Nothing playing yet; PULSE_SINK env var is enough for when it starts
                        Ok(())
                    }
                })
                .await;

                let mut state = self.state.write().await;
                match result {
                    Ok(Ok(())) => state.notify(format!("Audio output: {}", label)),
                    Ok(Err(e)) => state.notify_error(format!("pactl error: {}", e)),
                    Err(e) => state.notify_error(format!("Internal error: {}", e)),
                }

                // Restart FFmpeg with the new device if something was playing
                if let Some(url) = current_url {
                    drop(state);
                    self.restart_audio_backend(sink_name.clone()).await;
                    
                    // Reload the track if it was playing
                    if was_playing {
                        let _ = self.ffmpeg.loadfile(&url);
                        if let Some(pos) = position {
                            let _ = self.ffmpeg.seek(pos);
                        }
                        let _ = self.ffmpeg.resume();
                    } else {
                        // Just load it without playing
                        let _ = self.ffmpeg.loadfile(&url);
                    }
                }
            }
            None => {
                // Remove the forced routing — new streams go to system default again
                std::env::remove_var("PULSE_SINK");
                let mut state = self.state.write().await;
                state.notify("Audio output: System default");

                // Restart FFmpeg with default device if something was playing
                if let Some(url) = current_url {
                    drop(state);
                    self.restart_audio_backend(None).await;
                    
                    if was_playing {
                        let _ = self.ffmpeg.loadfile(&url);
                        if let Some(pos) = position {
                            let _ = self.ffmpeg.seek(pos);
                        }
                        let _ = self.ffmpeg.resume();
                    } else {
                        let _ = self.ffmpeg.loadfile(&url);
                    }
                }
            }
        }
    }

    /// Restart the FFmpeg audio backend with a new device
    async fn restart_audio_backend(&mut self, device_name: Option<String>) {
        use crate::audio::ffmpeg::FfmpegController;

        // Stop current playback
        let _ = self.ffmpeg.stop();

        // Create new controller with the new device
        self.ffmpeg = FfmpegController::with_device(device_name);

        // Restart FFmpeg
        if let Err(e) = self.ffmpeg.start() {
            let mut state = self.state.write().await;
            state.notify_error(format!("Failed to restart audio backend: {}", e));
        }
    }
}

