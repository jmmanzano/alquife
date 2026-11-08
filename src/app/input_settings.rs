use crossterm::event::{self, KeyCode};

use crate::error::Error;

use super::*;

impl App {
    /// Handle settings page keys
    pub(super) async fn handle_settings_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        let mut config_changed = false;
        let mut equalizer_changed = false;
        let mut clear_cache = false;

        {
            let mut state = self.state.write().await;
            let field = state.settings_state.selected_field;

            match key.code {
                // Navigate between fields
                KeyCode::Up | KeyCode::Char('k') => {
                    state.settings_state.confirm_clear_cache = false;
                    if field > 0 {
                        state.settings_state.selected_field = field - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.settings_state.confirm_clear_cache = false;
                    if field < 9 {
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
                    5 => {
                        state.settings_state.auto_update_enabled = !state.settings_state.auto_update_enabled;
                        state.config.auto_update_enabled = state.settings_state.auto_update_enabled;
                        let status = if state.settings_state.auto_update_enabled { "On" } else { "Off" };
                        state.notify(format!("Auto-update: {}", status));
                        config_changed = true;
                    }
                    6 => {
                        state.settings_state.show_cover_art = !state.settings_state.show_cover_art;
                        state.config.show_cover_art = state.settings_state.show_cover_art;
                        let status = if state.settings_state.show_cover_art { "On" } else { "Off" };
                        state.notify(format!("Album covers: {}", status));
                        config_changed = true;
                    }
                    7 => {
                        state.config.cache_size_gb = state.config.cache_size_gb.saturating_sub(1);
                        let size = state.config.cache_size_gb;
                        state.notify(format!("Cache size: {} GB", size));
                        config_changed = true;
                    }
                    9 => {
                        state.config.radio_browser_enabled = !state.config.radio_browser_enabled;
                        if !state.config.radio_browser_enabled { state.radio.browser_active = false; state.radio.browser_searching = false; state.radio.browser_generation += 1; state.radio.browser_loading = false; state.radio.selected = (!state.radio.stations.is_empty()).then_some(0); }
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
                        5 => {
                            state.settings_state.auto_update_enabled = !state.settings_state.auto_update_enabled;
                            state.config.auto_update_enabled = state.settings_state.auto_update_enabled;
                            let status = if state.settings_state.auto_update_enabled { "On" } else { "Off" };
                            state.notify(format!("Auto-update: {}", status));
                            config_changed = true;
                        }
                        6 => {
                            state.settings_state.show_cover_art = !state.settings_state.show_cover_art;
                            state.config.show_cover_art = state.settings_state.show_cover_art;
                            let status = if state.settings_state.show_cover_art { "On" } else { "Off" };
                            state.notify(format!("Album covers: {}", status));
                            config_changed = true;
                        }
                        7 => {
                            state.config.cache_size_gb = (state.config.cache_size_gb + 1).min(5);
                            let size = state.config.cache_size_gb;
                            state.notify(format!("Cache size: {} GB", size));
                            config_changed = true;
                        }
                        8 if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) => {
                            if state.settings_state.confirm_clear_cache {
                                state.settings_state.confirm_clear_cache = false;
                                clear_cache = true;
                            } else {
                                state.settings_state.confirm_clear_cache = true;
                                state.notify("Press Enter again to clear all cached songs");
                            }
                        }
                        9 => {
                            state.config.radio_browser_enabled = !state.config.radio_browser_enabled;
                            if !state.config.radio_browser_enabled { state.radio.browser_active = false; state.radio.browser_searching = false; state.radio.browser_generation += 1; state.radio.browser_loading = false; state.radio.selected = (!state.radio.stations.is_empty()).then_some(0); }
                            config_changed = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if clear_cache {
            if let Some(task) = self.cache_task.take() { task.abort(); let _ = task.await; }
            let cache_root = crate::config::paths::cache_dir().map(|p| p.join("songs"));
            let playing_cached = self.ffmpeg.get_current_url().as_ref().is_some_and(|url| {
                cache_root.as_ref().is_some_and(|root| std::path::Path::new(url).starts_with(root))
            });
            if playing_cached {
                let _ = self.ffmpeg.stop();
                let mut state = self.state.write().await;
                state.now_playing.state = PlaybackState::Stopped;
                state.now_playing.song = None;
                state.queue_position = None;
            }
            let result = self.cache.clear().await;
            let mut state = self.state.write().await;
            state.offline_songs = crate::cache::SongCache::songs(&state.config);
            state.offline_selected = (!state.offline_songs.is_empty()).then_some(0);
            match result {
                Ok(()) => state.notify("Cache cleared"),
                Err(e) => { warn!("Failed to clear cache: {}", e); state.notify_error("Could not remove all cached files; see log"); }
            }
        }

        if config_changed {
            if field_is_cache(&self.state).await {
                if let Some(task) = self.cache_task.take() { task.abort(); let _ = task.await; }
                let size = self.state.read().await.config.cache_size_gb;
                self.cache.trim(size).await;
                let mut state = self.state.write().await;
                state.offline_songs = crate::cache::SongCache::songs(&state.config);
                state.offline_selected = (!state.offline_songs.is_empty()).then_some(0);
            }
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

async fn field_is_cache(state: &SharedState) -> bool {
    state.read().await.settings_state.selected_field == 7
}
