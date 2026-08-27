//! Radio page showing internet radio stations

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::state::{AppState, RadioEditMode, RadioInputField};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let colors = *state.settings_state.theme_colors();

    // If in edit mode, show modal instead
    if state.radio.edit_mode != RadioEditMode::Browse {
        render_radio_modal(frame, area, state);
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Radio Stations ({}) ", state.radio.stations.len()))
        .border_style(Style::default().fg(colors.border_focused));

    if state.radio.stations.is_empty() {
        let hint = Paragraph::new(
            "No internet radio stations found. Configure them in your Subsonic server.\n\n\
            [a] Add new  [r] Refresh\n\
            [j/k] or [↑↓] Navigate  [Enter] Play"
        )
            .style(Style::default().fg(colors.muted))
            .block(block);
        frame.render_widget(hint, area);
        return;
    }

    let is_playing_radio = state.playing_radio;
    let current_song_title = state.now_playing.song.as_ref().map(|s| s.title.clone());

    let items: Vec<ListItem> = state
        .radio
        .stations
        .iter()
        .enumerate()
        .map(|(i, station)| {
            let is_selected = state.radio.selected == Some(i);
            let is_playing = is_playing_radio
                && current_song_title
                    .as_ref()
                    .map(|t| t == &station.name)
                    .unwrap_or(false);

            let indicator = if is_playing { "▶ " } else { "  " };

            let (name_style, url_style) = if is_playing {
                (
                    Style::default()
                        .fg(colors.playing)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(colors.playing),
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(colors.primary)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(colors.muted),
                )
            } else {
                (
                    Style::default().fg(colors.song),
                    Style::default().fg(colors.muted),
                )
            };

            let home_page = station
                .home_page_url
                .as_deref()
                .unwrap_or("");
            let url_info = if !home_page.is_empty() {
                format!(" ({})", home_page)
            } else {
                String::new()
            };

            let line = Line::from(vec![
                Span::styled(format!("{:3}. ", i + 1), Style::default().fg(colors.muted)),
                Span::styled(indicator, Style::default().fg(colors.playing)),
                Span::styled(station.name.clone(), name_style),
                Span::styled(url_info, url_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(colors.highlight_bg))
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(state.radio.selected);
    frame.render_stateful_widget(list, area, &mut list_state);
    state.radio.scroll_offset = list_state.offset();
}

fn render_radio_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let colors = *state.settings_state.theme_colors();

    let mode = state.radio.edit_mode;
    let title = match mode {
        RadioEditMode::AddNew => "Add New Radio Station",
        RadioEditMode::Edit => "Edit Radio Station",
        RadioEditMode::Delete => "Delete Radio Station",
        RadioEditMode::Browse => return,
    };

    // Create modal area (centered)
    let modal_height = if mode == RadioEditMode::Delete { 7 } else { 12 };
    let modal_width = 70;
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect {
        x: area.x + modal_x,
        y: area.y + modal_y,
        width: modal_width,
        height: modal_height,
    };

    // Modal background
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(colors.primary));

    let inner = modal_block.inner(modal_area);
    frame.render_widget(modal_block, modal_area);

    if mode == RadioEditMode::Delete {
        if let Some(station) = &state.radio.editing_radio {
            let text = format!(
                "Delete: {}\n\nPress Delete to confirm, Esc to cancel",
                station.name
            );
            let para = Paragraph::new(text)
                .style(Style::default().fg(colors.accent));
            frame.render_widget(para, inner);
        }
    } else {
        let name_label = if state.radio.focused_field == RadioInputField::Name {
            "Name (TAB to next): "
        } else {
            "Name: "
        };

        let url_label = if state.radio.focused_field == RadioInputField::StreamUrl {
            "Stream URL (TAB to next): "
        } else {
            "Stream URL: "
        };

        let home_label = if state.radio.focused_field == RadioInputField::HomePageUrl {
            "Home Page URL (TAB to next): "
        } else {
            "Home Page URL: "
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(inner);

        // Name field
        let name_text = Line::from(vec![
            Span::styled(name_label, Style::default().fg(colors.muted)),
            Span::raw(if state.radio.focused_field == RadioInputField::Name {
                format!("{}|", state.radio.radio_name)
            } else {
                state.radio.radio_name.clone()
            }),
        ]);
        let name_para = Paragraph::new(name_text);
        frame.render_widget(name_para, layout[0]);

        // Stream URL field
        let url_text = Line::from(vec![
            Span::styled(url_label, Style::default().fg(colors.muted)),
            Span::raw(if state.radio.focused_field == RadioInputField::StreamUrl {
                format!("{}|", state.radio.radio_stream_url)
            } else {
                state.radio.radio_stream_url.clone()
            }),
        ]);
        let url_para = Paragraph::new(url_text);
        frame.render_widget(url_para, layout[1]);

        // Home page URL field
        let home_text = Line::from(vec![
            Span::styled(home_label, Style::default().fg(colors.muted)),
            Span::raw(if state.radio.focused_field == RadioInputField::HomePageUrl {
                format!("{}|", state.radio.radio_home_page_url)
            } else {
                state.radio.radio_home_page_url.clone()
            }),
        ]);
        let home_para = Paragraph::new(home_text);
        frame.render_widget(home_para, layout[2]);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled(
                "TAB: next field  |  Enter: save  |  Esc: cancel  |  Backspace: delete",
                Style::default()
                    .fg(colors.muted)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);
        let instr_para = Paragraph::new(instructions);
        frame.render_widget(instr_para, layout[3]);
    }
}
