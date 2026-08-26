//! Settings page with app preferences and theming

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::AppState;
use crate::ui::theme::ThemeColors;

/// Render the settings page
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let colors = *state.settings_state.theme_colors();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings ")
        .border_style(Style::default().fg(colors.border_focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 15 {
        return;
    }

    let settings = &state.settings_state;

    // Layout fields vertically with spacing
    let chunks = Layout::vertical([
        Constraint::Length(1), // Spacing
        Constraint::Length(2), // Theme selector
        Constraint::Length(1), // Spacing
        Constraint::Length(2), // Audio backend
        Constraint::Length(1), // Spacing
        Constraint::Length(2), // Non-stop mode
        Constraint::Length(1), // Spacing
        Constraint::Length(2), // Equalizer
        Constraint::Length(1), // Spacing
        Constraint::Length(2), // Notifications
        Constraint::Min(1),    // Remaining space
    ])
    .split(inner);

    // Theme selector (field 0)
    render_option(
        frame,
        chunks[1],
        "Theme",
        settings.theme_name(),
        settings.selected_field == 0,
        &colors,
    );

    // Audio backend (field 1)
    render_option(
        frame,
        chunks[3],
        "Audio Backend",
        settings.audio_backend.label(),
        settings.selected_field == 1,
        &colors,
    );

    // Non-stop mode (field 2)
    let non_stop_value = if settings.non_stop_mode { "On" } else { "Off" };
    render_option(
        frame,
        chunks[5],
        "Non-stop mode",
        non_stop_value,
        settings.selected_field == 2,
        &colors,
    );

    // Equalizer (field 3)
    let eq_value = if settings.equalizer_enabled {
        format!("On ({})", settings.equalizer_preset_name())
    } else {
        format!("Off ({})", settings.equalizer_preset_name())
    };
    render_option(
        frame,
        chunks[7],
        "Equalizer",
        &eq_value,
        settings.selected_field == 3,
        &colors,
    );

    // Notifications (field 4)
    let notif_value = if settings.notifications_enabled { "On" } else { "Off" };
    render_option(
        frame,
        chunks[9],
        "Notifications",
        notif_value,
        settings.selected_field == 4,
        &colors,
    );

    // Help text at bottom
    let help_text = match settings.selected_field {
        0 => "← → or Enter to change theme (auto-saves)",
        1 => "Audio backend: FFmpeg (only supported backend)",
        2 => "← → or Enter to toggle Non-stop mode (auto-saves)",
        3 => "← → or Enter to toggle equalizer (auto-saves), F7 to choose preset",
        4 => "← → or Enter to toggle desktop notifications when a song changes (auto-saves)",
        _ => "",
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(colors.muted));

    let help_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(2),
        inner.width,
        1,
    );
    frame.render_widget(help, help_area);
}

/// Render an option selector
fn render_option(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    colors: &ThemeColors,
) {
    let label_style = if selected {
        Style::default()
            .fg(colors.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(colors.highlight_fg)
    };

    let value_style = if selected {
        Style::default().fg(colors.accent)
    } else {
        Style::default().fg(colors.muted)
    };

    // Label
    let label_text = Paragraph::new(label).style(label_style);
    frame.render_widget(label_text, Rect::new(area.x, area.y, area.width, 1));

    // Value with arrows
    let value_text = if selected {
        format!("  ◀ {} ▶", value)
    } else {
        format!("    {}", value)
    };

    let value_para = Paragraph::new(value_text).style(value_style);
    frame.render_widget(value_para, Rect::new(area.x, area.y + 1, area.width, 1));
}
