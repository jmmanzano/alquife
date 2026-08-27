use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::error::Error;

use super::*;

impl App {
    /// Handle terminal events
    pub(super) async fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(key) => {
                // Only handle key press events, ignore release and repeat
                if key.kind == event::KeyEventKind::Press {
                    self.handle_key(key).await
                } else {
                    Ok(())
                }
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse).await,
            Event::Resize(_, _) => Ok(()),
            _ => Ok(()),
        }
    }

    /// Handle keyboard input
    pub(super) async fn handle_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        let mut state = self.state.write().await;

        // Clear notification on any keypress
        state.clear_notification();

        // Keep page switching available even when editing text fields
        match key.code {
            KeyCode::F(1) => {
                state.page = Page::Artists;
                return Ok(());
            }
            KeyCode::F(2) => {
                state.page = Page::Queue;
                return Ok(());
            }
            KeyCode::F(3) => {
                state.page = Page::Playlists;
                return Ok(());
            }
            KeyCode::F(4) => {
                state.page = Page::Radio;
                return Ok(());
            }
            KeyCode::F(5) => {
                state.page = Page::Server;
                return Ok(());
            }
            KeyCode::F(6) => {
                state.page = Page::Settings;
                return Ok(());
            }
            KeyCode::F(7) => {
                state.page = Page::Equalizer;
                return Ok(());
            }
            KeyCode::F(8) => {
                state.page = Page::AudioDevices;
                return Ok(());
            }
            _ => {}
        }

        // Bypass global keybindings when typing in text input fields
        let is_server_text_field =
            state.page == Page::Server && state.server_state.selected_field <= 2;
        let is_filtering = state.page == Page::Artists && state.artists.filter_active;
        let is_radio_edit_mode = state.page == Page::Radio && state.radio.edit_mode != crate::app::state::RadioEditMode::Browse;

        if is_server_text_field || is_filtering || is_radio_edit_mode {
            let page = state.page;
            drop(state);
            return match page {
                Page::Server => self.handle_server_key(key).await,
                Page::Artists => self.handle_artists_key(key).await,
                Page::Radio => self.handle_radio_key(key).await,
                _ => Ok(()),
            };
        }

        // Global keybindings
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                state.should_quit = true;
                return Ok(());
            }
            // Playback controls (global)
            (KeyCode::Char('p'), KeyModifiers::NONE) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
                // Toggle pause
                drop(state);
                return self.toggle_pause().await;
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                // Next track
                drop(state);
                return self.next_track().await;
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                // Previous track
                drop(state);
                return self.prev_track().await;
            }
                // Seek backward/forward 10 seconds (Shift+Left / Shift+Right)
                (KeyCode::Left, KeyModifiers::SHIFT) => {
                    drop(state);
                    let _ = self.audio_seek_relative(-10.0);
                    return Ok(());
                }
                (KeyCode::Right, KeyModifiers::SHIFT) => {
                    drop(state);
                    let _ = self.audio_seek_relative(10.0);
                    return Ok(());
                }
            // Cycle theme (global)
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                state.settings_state.next_theme();
                state.config.theme = state.settings_state.theme_name().to_string();
                let label = state.settings_state.theme_name().to_string();
                state.notify(format!("Theme: {}", label));
                let _ = state.config.save_default();
                return Ok(());
            }
            // Ctrl+R to refresh data from server
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                state.notify("Refreshing...");
                drop(state);
                self.load_initial_data().await;
                let mut state = self.state.write().await;
                state.notify("Data refreshed");
                return Ok(());
            }
            _ => {}
        }

        // Page-specific keybindings
        let page = state.page;
        drop(state);
        match page {
            Page::Artists => self.handle_artists_key(key).await,
            Page::Queue => self.handle_queue_key(key).await,
            Page::Playlists => self.handle_playlists_key(key).await,
            Page::Radio => {
                tracing::debug!("Routing to handle_radio_key: {:?}", key);
                self.handle_radio_key(key).await
            },
            Page::Server => self.handle_server_key(key).await,
            Page::Settings => self.handle_settings_key(key).await,
            Page::Equalizer => self.handle_equalizer_key(key).await,
            Page::AudioDevices => self.handle_audio_devices_key(key).await,
        }
    }
}
