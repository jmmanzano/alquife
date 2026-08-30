//! Radio station management operations

use tracing::info;
use crate::error::Error;
use super::*;

impl App {
    /// Create a new internet radio station
    pub async fn create_radio_station(
        &mut self,
        name: &str,
        stream_url: &str,
        home_page_url: Option<&str>,
    ) -> Result<(), Error> {
        if name.is_empty() || stream_url.is_empty() {
            return Err(Error::Ui(crate::error::UiError::Input(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Radio name and stream URL cannot be empty",
                ),
            )));
        }

        // Validate URLs
        if !stream_url.starts_with("http://") && !stream_url.starts_with("https://") {
            return Err(Error::Ui(crate::error::UiError::Input(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Stream URL must start with http:// or https://",
                ),
            )));
        }

        // Create station via Subsonic API
        let client = self.subsonic.as_ref().ok_or_else(|| {
            Error::Subsonic(crate::error::SubsonicError::NotConfigured)
        })?;

        client
            .create_internet_radio_station(name, stream_url, home_page_url)
            .await?;

        // Refresh the station list to get the updated data
        let stations = client.get_internet_radio_stations().await?;
        let mut state = self.state.write().await;
        state.radio.stations = stations;
        state.radio.selected = Some(state.radio.stations.len().saturating_sub(1));
        
        info!("Created radio station: {}", name);
        Ok(())
    }

    /// Update an existing internet radio station
    pub async fn update_radio_station(
        &mut self,
        id: &str,
        name: &str,
        stream_url: &str,
        home_page_url: Option<&str>,
    ) -> Result<(), Error> {
        if name.is_empty() || stream_url.is_empty() {
            return Err(Error::Ui(crate::error::UiError::Input(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Radio name and stream URL cannot be empty",
                ),
            )));
        }

        // Validate URLs
        if !stream_url.starts_with("http://") && !stream_url.starts_with("https://") {
            return Err(Error::Ui(crate::error::UiError::Input(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Stream URL must start with http:// or https://",
                ),
            )));
        }

        // Update station via Subsonic API
        let client = self.subsonic.as_ref().ok_or_else(|| {
            Error::Subsonic(crate::error::SubsonicError::NotConfigured)
        })?;

        client
            .update_internet_radio_station(id, name, stream_url, home_page_url)
            .await?;

        // Refresh the station list to get the updated data
        let stations = client.get_internet_radio_stations().await?;
        
        let mut state = self.state.write().await;
        let current_selection = state.radio.stations.iter().position(|s| s.id == id);
        state.radio.stations = stations;
        
        // Preserve selection if the updated station still exists
        if let Some(pos) = state.radio.stations.iter().position(|s| s.id == id) {
            state.radio.selected = Some(pos);
        } else if let Some(sel) = current_selection {
            state.radio.selected = Some(sel.min(state.radio.stations.len().saturating_sub(1)));
        }
        
        info!("Updated radio station: {}", name);
        Ok(())
    }

    /// Delete an internet radio station
    pub async fn delete_radio_station(&mut self, id: &str) -> Result<(), Error> {
        // Delete via Subsonic API
        let client = self.subsonic.as_ref().ok_or_else(|| {
            Error::Subsonic(crate::error::SubsonicError::NotConfigured)
        })?;

        client
            .delete_internet_radio_station(id)
            .await?;

        // Remove from state
        let mut state = self.state.write().await;
        let original_len = state.radio.stations.len();
        state.radio.stations.retain(|s| s.id != id);
        
        // Update selection if needed
        if let Some(sel) = state.radio.selected {
            if sel >= state.radio.stations.len() && !state.radio.stations.is_empty() {
                state.radio.selected = Some(state.radio.stations.len() - 1);
            }
        }
        
        if state.radio.stations.len() < original_len {
            info!("Deleted radio station with id: {}", id);
        }
        
        Ok(())
    }

    /// Refresh the list of radio stations from server
    pub async fn refresh_radio_stations(&mut self) -> Result<(), Error> {
        let client = self.subsonic.as_ref().ok_or_else(|| {
            Error::Subsonic(crate::error::SubsonicError::NotConfigured)
        })?;

        let stations = client.get_internet_radio_stations().await?;
        let mut state = self.state.write().await;
        state.radio.stations = stations;
        state.radio.selected = None;
        state.radio.scroll_offset = 0;
        info!("Refreshed radio stations list");
        Ok(())
    }
}
