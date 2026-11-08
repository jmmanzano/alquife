use crossterm::event::{self, KeyCode};
use crate::error::Error;
use crate::app::state::{RadioEditMode, RadioInputField};
use super::*;

impl App {
    pub(super) async fn handle_radio_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        {
            let mut state = self.state.write().await;
            if state.radio.browser_searching {
                match key.code {
                    KeyCode::Esc => state.radio.browser_searching = false,
                    KeyCode::Tab | KeyCode::Down => state.radio.browser_search_field = (state.radio.browser_search_field + 1) % 3,
                    KeyCode::BackTab | KeyCode::Up => state.radio.browser_search_field = (state.radio.browser_search_field + 2) % 3,
                    KeyCode::Backspace => { let field = state.radio.browser_search_field; state.radio.browser_search_fields[field].pop(); },
                    KeyCode::Char(c) => { let field = state.radio.browser_search_field; if state.radio.browser_search_fields[field].len() < 200 { state.radio.browser_search_fields[field].push(c); } },
                    KeyCode::Enter => {
                        [state.radio.browser_query, state.radio.browser_country, state.radio.browser_genre] = state.radio.browser_search_fields.clone();
                        state.radio.browser_searching = false;
                        drop(state);
                        self.search_radio_browser().await;
                    }
                    _ => {}
                }
                return Ok(());
            }
        }
        let mode = {
            let state = self.state.read().await;
            state.radio.edit_mode
        };

        if mode != RadioEditMode::Browse {
            return self.handle_radio_edit_key(key).await;
        }

        // Browse mode inputs
        let mut state = self.state.write().await;
        match key.code {
            KeyCode::Tab if state.config.radio_browser_enabled => {
                state.radio.browser_active = !state.radio.browser_active;
                state.radio.selected = (!state.radio.visible_stations().is_empty()).then_some(0);
                state.radio.scroll_offset = 0;
                let load = state.radio.browser_active && state.radio.browser_stations.is_empty() && !state.radio.browser_loading;
                drop(state);
                if load { self.search_radio_browser().await; }
                return Ok(());
            }
            KeyCode::Char('/') if state.radio.browser_active => {
                state.radio.browser_search_fields = [state.radio.browser_query.clone(), state.radio.browser_country.clone(), state.radio.browser_genre.clone()];
                state.radio.browser_search_field = 0;
                state.radio.browser_searching = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(sel) = state.radio.selected {
                    if sel > 0 {
                        state.radio.selected = Some(sel - 1);
                    }
                } else if !state.radio.visible_stations().is_empty() {
                    state.radio.selected = Some(0);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = state.radio.visible_stations().len().saturating_sub(1);
                if let Some(sel) = state.radio.selected {
                    if sel < max {
                        state.radio.selected = Some(sel + 1);
                    }
                } else if !state.radio.visible_stations().is_empty() {
                    state.radio.selected = Some(0);
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = state.radio.selected {
                    if let Some(station) = state.radio.visible_stations().get(idx).cloned() {
                        drop(state);
                        return self.play_radio_station(&station).await;
                    }
                }
            }
            KeyCode::Char('s') => {
                // Stop radio playback
                let is_radio = state.playing_radio;
                drop(state);
                if is_radio {
                    let _ = self.stop_playback().await;
                }
                return Ok(());
            }
            KeyCode::Char('a') if state.radio.browser_active => {
                let station = state.radio.selected.and_then(|idx| state.radio.browser_stations.get(idx)).cloned();
                drop(state);
                if let Some(station) = station {
                    self.add_browser_station(&station).await;
                }
                return Ok(());
            }
            KeyCode::Char('a') if !state.radio.browser_active => {
                // Add new radio
                tracing::debug!("Add radio pressed - switching to AddNew mode");
                state.radio.edit_mode = RadioEditMode::AddNew;
                state.radio.radio_name.clear();
                state.radio.radio_stream_url.clear();
                state.radio.radio_home_page_url.clear();
                state.radio.focused_field = RadioInputField::Name;
                state.radio.editing_radio = None;
                tracing::debug!("Radio edit mode set to AddNew, fields cleared");
            }
            KeyCode::Char('e') if !state.radio.browser_active => {
                // Edit selected radio
                if let Some(idx) = state.radio.selected {
                    if let Some(station) = state.radio.stations.get(idx).cloned() {
                        state.radio.edit_mode = RadioEditMode::Edit;
                        state.radio.editing_radio = Some(station.clone());
                        state.radio.radio_name = station.name.clone();
                        state.radio.radio_stream_url = station.stream_url.clone();
                        state.radio.radio_home_page_url = station.home_page_url.clone().unwrap_or_default();
                        state.radio.focused_field = RadioInputField::Name;
                    }
                }
            }
            KeyCode::Char('d') if !state.radio.browser_active => {
                // Delete selected radio
                if let Some(idx) = state.radio.selected {
                    if let Some(station) = state.radio.stations.get(idx).cloned() {
                        state.radio.edit_mode = RadioEditMode::Delete;
                        state.radio.editing_radio = Some(station);
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh radios list
                let browser = state.radio.browser_active;
                drop(state);
                if browser { self.search_radio_browser().await; return Ok(()); }
                let _ = self.refresh_radio_stations().await;
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    async fn add_browser_station(&self, station: &crate::subsonic::models::InternetRadioStation) {
        let Some(client) = self.subsonic.as_ref() else {
            self.state.write().await.notify_error("Configure a Subsonic server before adding a station");
            return;
        };
        let result = client.create_internet_radio_station(
            &station.name, &station.stream_url, station.home_page_url.as_deref(),
        ).await;
        if let Err(error) = result {
            self.state.write().await.notify_error(format!("Could not add station to server: {error}"));
            return;
        }

        // A failed refresh must not be reported as a failed creation: retrying could
        // otherwise create duplicates on the server.
        let refreshed = client.get_internet_radio_stations().await;
        let mut state = self.state.write().await;
        match refreshed {
            Ok(stations) => {
                state.radio.stations = stations;
                state.notify(format!("Added '{}' to server radios", station.name));
            }
            Err(error) => {
                tracing::warn!("Station added but server radio refresh failed: {error}");
                state.notify_error("Station added, but server list could not be refreshed; press r on the server source");
            }
        }
    }

    async fn search_radio_browser(&self) {
        let mut state = self.state.write().await;
        if !state.config.radio_browser_enabled { return; }
        state.radio.browser_generation = state.radio.browser_generation.wrapping_add(1);
        let generation = state.radio.browser_generation;
        let query = state.radio.browser_query.trim().to_string();
        let country = state.radio.browser_country.trim().to_string();
        let genre = state.radio.browser_genre.trim().to_string();
        state.radio.browser_loading = true;
        let shared = self.state.clone();
        drop(state);
        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || -> Result<Vec<crate::subsonic::models::InternetRadioStation>, String> {
                let api = radiobrowser::blocking::RadioBrowserAPI::new().map_err(|e| e.to_string())?;
                let mut search = api.get_stations().limit("100").hidebroken(true)
                    .order(radiobrowser::StationOrder::Clickcount).reverse(true);
                if !query.is_empty() { search = search.name(&query); }
                if !country.is_empty() { search = search.country(&country); }
                if !genre.is_empty() { search = search.tag(&genre); }
                let stations = search.send().map_err(|e| e.to_string())?;
                Ok(stations.into_iter().filter_map(|s| {
                    let stream_url = if s.url_resolved.is_empty() { s.url } else { s.url_resolved };
                    if !(stream_url.starts_with("http://") || stream_url.starts_with("https://")) { return None; }
                    Some(crate::subsonic::models::InternetRadioStation { id: s.stationuuid, name: s.name, stream_url, home_page_url: (!s.homepage.is_empty()).then_some(s.homepage) })
                }).collect())
            }).await;
            let mut state = shared.write().await;
            if state.radio.browser_generation != generation || !state.config.radio_browser_enabled { return; }
            state.radio.browser_loading = false;
            match result {
                Ok(Ok(stations)) => {
                    state.radio.browser_stations = stations;
                    if state.radio.browser_active { state.radio.selected = (!state.radio.browser_stations.is_empty()).then_some(0); state.radio.scroll_offset = 0; }
                }
                Ok(Err(error)) => state.notify_error(format!("Radio Browser: {error}")),
                Err(error) => state.notify_error(format!("Radio Browser task: {error}")),
            }
        });
    }

    async fn handle_radio_edit_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        use crate::app::state::RadioEditMode;

        match key.code {
            KeyCode::Esc => {
                let mut state = self.state.write().await;
                state.radio.edit_mode = RadioEditMode::Browse;
                state.radio.radio_name.clear();
                state.radio.radio_stream_url.clear();
                state.radio.radio_home_page_url.clear();
                state.radio.editing_radio = None;
            }
            KeyCode::Tab => {
                let mut state = self.state.write().await;
                // Move to next field (without clearing input)
                state.radio.focused_field = match state.radio.focused_field {
                    RadioInputField::Name => RadioInputField::StreamUrl,
                    RadioInputField::StreamUrl => RadioInputField::HomePageUrl,
                    RadioInputField::HomePageUrl => RadioInputField::Name,
                };
            }
            KeyCode::Enter => {
                let (mode, name, stream_url, home_page_url, editing_radio) = {
                    let state = self.state.read().await;
                    (
                        state.radio.edit_mode,
                        state.radio.radio_name.clone(),
                        state.radio.radio_stream_url.clone(),
                        state.radio.radio_home_page_url.clone(),
                        state.radio.editing_radio.clone(),
                    )
                };
                
                self.save_radio(mode, &name, &stream_url, &home_page_url, editing_radio).await?;
                
                let mut state = self.state.write().await;
                state.radio.edit_mode = RadioEditMode::Browse;
                state.radio.radio_name.clear();
                state.radio.radio_stream_url.clear();
                state.radio.radio_home_page_url.clear();
                state.radio.editing_radio = None;
            }
            KeyCode::Backspace => {
                let mut state = self.state.write().await;
                match state.radio.focused_field {
                    RadioInputField::Name => { state.radio.radio_name.pop(); },
                    RadioInputField::StreamUrl => { state.radio.radio_stream_url.pop(); },
                    RadioInputField::HomePageUrl => { state.radio.radio_home_page_url.pop(); },
                }
            }
            KeyCode::Char(c) => {
                let mut state = self.state.write().await;
                match state.radio.focused_field {
                    RadioInputField::Name => {
                        if state.radio.radio_name.len() < 255 {
                            state.radio.radio_name.push(c);
                        }
                    },
                    RadioInputField::StreamUrl => {
                        if state.radio.radio_stream_url.len() < 500 {
                            state.radio.radio_stream_url.push(c);
                        }
                    },
                    RadioInputField::HomePageUrl => {
                        if state.radio.radio_home_page_url.len() < 500 {
                            state.radio.radio_home_page_url.push(c);
                        }
                    },
                }
            }
            KeyCode::Delete => {
                let mut state = self.state.write().await;
                if state.radio.edit_mode == RadioEditMode::Delete {
                    // Confirm deletion
                    if let Some(station) = state.radio.editing_radio.take() {
                        let id = station.id.clone();
                        drop(state);
                        let _ = self.delete_radio_station(&id).await;
                        let mut state = self.state.write().await;
                        state.radio.edit_mode = RadioEditMode::Browse;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn save_radio(
        &mut self,
        mode: RadioEditMode,
        name: &str,
        stream_url: &str,
        home_page_url: &str,
        editing_radio: Option<crate::subsonic::models::InternetRadioStation>,
    ) -> Result<(), Error> {
        let name = name.trim();
        let stream_url = stream_url.trim();
        let home_page_url = home_page_url.trim();

        // Validate required fields
        if name.is_empty() {
            let mut state = self.state.write().await;
            state.notify_error("Radio name cannot be empty");
            return Ok(());
        }

        if stream_url.is_empty() {
            let mut state = self.state.write().await;
            state.notify_error("Stream URL cannot be empty");
            return Ok(());
        }

        // Auto-prefix stream URL if needed
        let stream_url = if !stream_url.starts_with("http://") && !stream_url.starts_with("https://") {
            format!("https://{}", stream_url)
        } else {
            stream_url.to_string()
        };

        // Auto-prefix home page URL if provided and doesn't have protocol
        let home_page_url = if !home_page_url.is_empty() {
            if !home_page_url.starts_with("http://") && !home_page_url.starts_with("https://") {
                format!("https://{}", home_page_url)
            } else {
                home_page_url.to_string()
            }
        } else {
            String::new()
        };

        let result = match mode {
            RadioEditMode::AddNew => {
                let home_page = if home_page_url.is_empty() { None } else { Some(home_page_url.as_str()) };
                self.create_radio_station(name, &stream_url, home_page).await
            }
            RadioEditMode::Edit => {
                if let Some(station) = editing_radio {
                    let home_page = if home_page_url.is_empty() { None } else { Some(home_page_url.as_str()) };
                    self.update_radio_station(&station.id, name, &stream_url, home_page).await
                } else {
                    Ok(())
                }
            }
            _ => Ok(())
        };

        match result {
            Ok(_) => {
                let mut state = self.state.write().await;
                state.notify(format!("Radio station '{}' saved successfully", name));
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.notify_error(format!("Failed to save radio: {}", e));
            }
        }
        
        Ok(())
    }
}
