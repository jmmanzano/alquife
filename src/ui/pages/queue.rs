//! Queue page showing current play queue

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::state::AppState;

pub fn queue_areas(area: Rect) -> (Rect, Rect) {
    let [queue, cover] = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(area);
    (queue, cover)
}

fn render_cover(frame: &mut Frame, area: Rect, state: &AppState, cover: Option<&mut ratatui_image::protocol::StatefulProtocol>) {
    let colors = state.settings_state.theme_colors();
    let block = Block::default().borders(Borders::ALL).title(" Album cover ")
        .border_style(Style::default().fg(colors.border_unfocused));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 { return; }
    let Some(image) = cover else {
        let text = if state.queue_state.cover_loading { "Loading cover..." } else { "No cover available" };
        frame.render_widget(Paragraph::new(text).style(Style::default().fg(colors.muted)), inner);
        return;
    };
    let resize = ratatui_image::Resize::Fit(None);
    let size = image.size_for(resize.clone(), inner.into());
    let area = Rect::new(
        inner.x + inner.width.saturating_sub(size.width) / 2,
        inner.y + inner.height.saturating_sub(size.height) / 2,
        size.width.min(inner.width),
        size.height.min(inner.height),
    );
    frame.render_stateful_widget(ratatui_image::StatefulImage::new().resize(resize), area, image);
}

fn song_rating_stars(song: &crate::subsonic::models::Child) -> String {
    let rating = song
        .user_rating
        .map(|r| r.min(5))
        .or_else(|| {
            song.average_rating
                .map(|r| r.round().clamp(0.0, 5.0) as u8)
        })
        .unwrap_or(0);

    format!("{}{}", "★".repeat(rating as usize), "☆".repeat((5 - rating) as usize))
}


/// Render the queue page
pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState, cover: Option<&mut ratatui_image::protocol::StatefulProtocol>) {
    let colors = *state.settings_state.theme_colors();
    let area = if state.config.show_cover_art {
        let (queue, cover_area) = queue_areas(area);
        render_cover(frame, cover_area, state, cover);
        queue
    } else { area };

    let filter_suffix = if state.queue_state.min_playback_rating > 0 {
        format!(" | min {}+", state.queue_state.min_playback_rating)
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Queue ({}){} ", state.queue.len(), filter_suffix))
        .border_style(Style::default().fg(colors.border_focused));

    if state.queue.is_empty() {
        let hint = Paragraph::new("Queue is empty. Add songs from Artists or Playlists.")
            .style(Style::default().fg(colors.muted))
            .block(block);
        frame.render_widget(hint, area);
        return;
    }

    let items: Vec<ListItem> = state
        .queue
        .iter()
        .enumerate()
        .map(|(i, song)| {
            let is_current = state.queue_position == Some(i);
            let is_selected = state.queue_state.selected == Some(i);
            let is_played = state.queue_position.map(|pos| i < pos).unwrap_or(false);

            let indicator = if is_current { "▶ " } else { "  " };

            let artist = song.artist.clone().unwrap_or_default();
            let duration = song.format_duration();
            let rating = song_rating_stars(song);
            // Show disc.track for songs with disc info
            let track_info = match (song.disc_number, song.track) {
                (Some(d), Some(t)) if d > 1 => format!(" [{}.{}]", d, t),
                (_, Some(t)) => format!(" [#{}]", t),
                _ => String::new(),
            };

            // Color scheme: played = muted, current = playing color, upcoming = song color
            let (title_style, artist_style, number_style) = if is_current {
                (
                    Style::default()
                        .fg(colors.playing)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(colors.playing),
                    Style::default().fg(colors.playing),
                )
            } else if is_played {
                (
                    if is_selected {
                        Style::default()
                            .fg(colors.played)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.played)
                    },
                    Style::default().fg(colors.muted),
                    Style::default().fg(colors.muted),
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(colors.primary)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(colors.muted),
                    Style::default().fg(colors.muted),
                )
            } else {
                (
                    Style::default().fg(colors.song),
                    Style::default().fg(colors.muted),
                    Style::default().fg(colors.muted),
                )
            };

            let line = Line::from(vec![
                Span::styled(format!("{:3}. ", i + 1), number_style),
                Span::styled(indicator, Style::default().fg(colors.playing)),
                Span::styled(song.title.clone(), title_style),
                Span::styled(track_info, Style::default().fg(colors.muted)),
                Span::styled(format!(" {}", rating), Style::default().fg(colors.muted)),
                if !artist.is_empty() {
                    Span::styled(format!(" - {}", artist), artist_style)
                } else {
                    Span::raw("")
                },
                Span::styled(
                    format!(" [{}]", duration),
                    Style::default().fg(colors.muted),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(colors.highlight_bg))
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(state.queue_state.selected);

    frame.render_stateful_widget(list, area, &mut list_state);
    state.queue_state.scroll_offset = list_state.offset();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn queue_and_cover_fit_the_available_area() {
        let area = Rect::new(4, 2, 100, 20);
        let (queue, cover) = queue_areas(area);
        assert_eq!(queue.width, 60);
        assert_eq!(cover.width, 40);
        assert_eq!(queue.right(), cover.x);
        assert_eq!(cover.right(), area.right());
    }

    #[test]
    fn cover_renders_pixels_and_empty_queue_hint() {
        let mut state = AppState::default();
        let mut pixels = image::RgbImage::new(128, 128);
        for pixel in pixels.pixels_mut() { *pixel = image::Rgb([255, 0, 0]); }
        let mut cover = ratatui_image::picker::Picker::halfblocks()
            .new_resize_protocol(image::DynamicImage::ImageRgb8(pixels));
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|frame| render(frame, frame.area(), &mut state, Some(&mut cover))).unwrap();
        let buffer = terminal.backend().buffer();
        assert!(buffer.content().iter().any(|cell| cell.fg == ratatui::style::Color::Rgb(255, 0, 0) || cell.bg == ratatui::style::Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn cover_is_centered_when_panel_is_larger_than_image() {
        let state = AppState::default();
        let pixels = image::RgbImage::from_pixel(64, 64, image::Rgb([255, 0, 0]));
        let mut cover = ratatui_image::picker::Picker::halfblocks()
            .new_resize_protocol(image::DynamicImage::ImageRgb8(pixels));
        let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
        terminal.draw(|frame| render_cover(frame, frame.area(), &state, Some(&mut cover))).unwrap();
        let buffer = terminal.backend().buffer();
        let positions: Vec<_> = (1..29).flat_map(|y| (1..79).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let cell = &buffer[(x, y)];
                cell.fg == ratatui::style::Color::Rgb(255, 0, 0) || cell.bg == ratatui::style::Color::Rgb(255, 0, 0)
            }).collect();
        assert!(!positions.is_empty());
        let min_x = positions.iter().map(|p| p.0).min().unwrap();
        let max_x = positions.iter().map(|p| p.0).max().unwrap();
        let min_y = positions.iter().map(|p| p.1).min().unwrap();
        let max_y = positions.iter().map(|p| p.1).max().unwrap();
        assert!((i32::from(min_x + max_x) - 79).abs() <= 1);
        assert!((i32::from(min_y + max_y) - 29).abs() <= 1);
    }
}
