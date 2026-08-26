use crossterm::event::{self, KeyCode};

use crate::error::Error;

use super::*;

impl App {
    /// Handle settings page keys
    pub(super) async fn handle_settings_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        let mut config_changed = false;
        let mut equalizer_changed = false;

        {
            let mut state = self.state.write().await;
            let field = state.settings_state.selected_field;

            match key.code {
                // Navigate between fields
                KeyCode::Up | KeyCode::Char('k') => {
                    if field > 0 {
                        state.settings_state.selected_field = field - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if field < 4 {
                        state.settings_state.selected_field = field + 1;
                    }
                }
                // Left
                KeyCode::Left | KeyCode::Char('h') => match field {
                    0 => {
                        state.settings_state.prev_theme();
                        state.config.theme = state.settings_state.theme_name().to_string();
                        let label = state.settings_state.theme_name().to_string();
                        state.notify(format!("Theme: {}", label));
                        config_changed = true;
                    }
                    1 => {
                        // Audio Backend is fixed to FFmpeg only
                        state.notify("Audio Backend: FFmpeg (only supported backend)");
                    }
                    2 => {
                        state.settings_state.non_stop_mode = !state.settings_state.non_stop_mode;
                        state.config.non_stop_mode = state.settings_state.non_stop_mode;
                        let status = if state.settings_state.non_stop_mode { "On" } else { "Off" };
                        state.notify(format!("Non-stop mode: {}", status));
                        config_changed = true;
                    }
                    3 => {
                        state.settings_state.equalizer_enabled = !state.settings_state.equalizer_enabled;
                        state.config.equalizer_enabled = state.settings_state.equalizer_enabled;
                        let status = if state.settings_state.equalizer_enabled { "On" } else { "Off" };
                        let preset_name = state.settings_state.equalizer_preset_name().to_string();
                        state.notify(format!("Equalizer: {} ({})", status, preset_name));
                        config_changed = true;
                        equalizer_changed = true;
                    }
                    4 => {
                        state.settings_state.notifications_enabled = !state.settings_state.notifications_enabled;
                        state.config.notifications_enabled = state.settings_state.notifications_enabled;
                        let status = if state.settings_state.notifications_enabled { "On" } else { "Off" };
                        state.notify(format!("Notifications: {}", status));
                        config_changed = true;
                    }
                    _ => {}
                },
                // Right / Enter / Space
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                    match field {
                        0 => {
                            state.settings_state.next_theme();
                            state.config.theme = state.settings_state.theme_name().to_string();
                            let label = state.settings_state.theme_name().to_string();
                            state.notify(format!("Theme: {}", label));
                            config_changed = true;
                        }
                        1 => {
                            // Audio Backend is fixed to FFmpeg only
                            state.notify("Audio Backend: FFmpeg (only supported backend)");
                        }
                        2 => {
                            state.settings_state.non_stop_mode = !state.settings_state.non_stop_mode;
                            state.config.non_stop_mode = state.settings_state.non_stop_mode;
                            let status = if state.settings_state.non_stop_mode { "On" } else { "Off" };
                            state.notify(format!("Non-stop mode: {}", status));
                            config_changed = true;
                        }
                        3 => {
                            state.settings_state.equalizer_enabled = !state.settings_state.equalizer_enabled;
                            state.config.equalizer_enabled = state.settings_state.equalizer_enabled;
                            let status = if state.settings_state.equalizer_enabled { "On" } else { "Off" };
                            let preset_name = state.settings_state.equalizer_preset_name().to_string();
                            state.notify(format!("Equalizer: {} ({})", status, preset_name));
                            config_changed = true;
                            equalizer_changed = true;
                        }
                        4 => {
                            state.settings_state.notifications_enabled = !state.settings_state.notifications_enabled;
                            state.config.notifications_enabled = state.settings_state.notifications_enabled;
                            let status = if state.settings_state.notifications_enabled { "On" } else { "Off" };
                            state.notify(format!("Notifications: {}", status));
                            config_changed = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if config_changed {
            // Save config
            let state = self.state.read().await;
            if let Err(e) = state.config.save_default() {
                drop(state);
                let mut state = self.state.write().await;
                state.notify_error(format!("Failed to save: {}", e));
            } else {
                drop(state);
                if equalizer_changed {
                    self.schedule_equalizer_apply_debounced();
                }
            }
        }

        Ok(())
    }
}
