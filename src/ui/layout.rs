//! Main layout and rendering

use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use crate::app::state::{AppState, LayoutAreas, Page};

use super::footer::Footer;
use super::header::Header;
use super::pages;
use super::widgets::NowPlayingWidget;

/// Draw the entire UI
pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    // Main layout:
    // [Header]          - 1 line
    // [Page Content]    - flexible
    // [Now Playing]     - 7 lines
    // [Footer]          - 1 line

    let chunks = Layout::vertical([
        Constraint::Length(1),  // Header
        Constraint::Min(10),   // Page content
        Constraint::Length(7), // Now playing
        Constraint::Length(1), // Footer
    ])
    .split(area);

    let header_area = chunks[0];
    let content_area = chunks[1];
    let now_playing_area = chunks[2];
    let footer_area = chunks[3];

    // Compute dual-pane splits for pages that use them
    let (content_left, content_right) = match state.page {
        Page::Artists | Page::Playlists => {
            let panes = Layout::horizontal([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(content_area);
            (Some(panes[0]), Some(panes[1]))
        }
        _ => (None, None),
    };

    // Store layout areas for mouse hit-testing
    state.layout = LayoutAreas {
        header: header_area,
        content: content_area,
        now_playing: now_playing_area,
        content_left,
        content_right,
    };

    // Render header
    let colors = *state.settings_state.theme_colors();
    let header = Header::new(state.page, state.now_playing.state, colors);
    frame.render_widget(header, header_area);

    // Render current page
    match state.page {
        Page::Artists => {
            pages::artists::render(frame, content_area, state);
        }
        Page::Queue => {
            pages::queue::render(frame, content_area, state);
        }
        Page::Playlists => {
            pages::playlists::render(frame, content_area, state);
        }
        Page::Radio => {
            pages::radio::render(frame, content_area, state);
        }
        Page::Server => {
            pages::server::render(frame, content_area, state);
        }
        Page::Settings => {
            pages::settings::render(frame, content_area, state);
        }
        Page::Equalizer => {
            pages::equalizer::render(frame, content_area, state);
        }
        Page::AudioDevices => {
            pages::audio_devices::render(frame, content_area, state);
        }
    }

    // Render now playing
    let now_playing = NowPlayingWidget::new(&state.now_playing, colors);
    frame.render_widget(now_playing, now_playing_area);

    // Render footer
    let footer = Footer::new(state.page, colors)
        .sample_rate(state.now_playing.sample_rate)
        .notification(state.notification.as_ref());
    frame.render_widget(footer, footer_area);
}
