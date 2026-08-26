//! Audio output devices page: select system audio output sinks

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::{AppState, AudioOutputMode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let colors = *state.settings_state.theme_colors();
    let ads = &state.audio_devices_state;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Audio Output Devices ")
        .border_style(Style::default().fg(colors.border_focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 6 || inner.width < 20 {
        return;
    }

    // Split in two columns: left = mode filter, right = sink list
    let cols = Layout::horizontal([Constraint::Length(22), Constraint::Min(10)]).split(inner);

    // ── Left column: filter selector ────────────────────────────────────────

    let mode_block = Block::default()
        .borders(Borders::ALL)
        .title(" Filter ")
        .border_style(if ads.focus == 0 {
            Style::default().fg(colors.border_focused)
        } else {
            Style::default().fg(colors.border_unfocused)
        });

    let mode_inner = mode_block.inner(cols[0]);
    frame.render_widget(mode_block, cols[0]);

    let modes = [AudioOutputMode::All, AudioOutputMode::Standard, AudioOutputMode::Bluetooth];
    let mut mode_lines: Vec<Line> = Vec::new();

    for m in &modes {
        let is_selected = ads.mode == *m;
        let count = ads.all_sinks.iter().filter(|s| match m {
            AudioOutputMode::All => true,
            AudioOutputMode::Standard => !s.is_bluetooth,
            AudioOutputMode::Bluetooth => s.is_bluetooth,
        }).count();

        let prefix = if is_selected && ads.focus == 0 { " ▶ " } else if is_selected { " ► " } else { "   " };
        let label = format!("{}{} ({})", prefix, m.label(), count);

        let style = if is_selected && ads.focus == 0 {
            Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.muted)
        };

        mode_lines.push(Line::from(Span::styled(label, style)));
    }

    mode_lines.push(Line::from(""));
    mode_lines.push(Line::from(Span::styled(
        " ↑↓ change filter",
        Style::default().fg(colors.muted),
    )));
    mode_lines.push(Line::from(Span::styled(
        " r  refresh list",
        Style::default().fg(colors.muted),
    )));

    // Active sink info
    if let Some(ref active) = state.config.audio_output_device {
        if let Some(sink) = ads.all_sinks.iter().find(|s| &s.name == active) {
            mode_lines.push(Line::from(""));
            mode_lines.push(Line::from(Span::styled(
                " Active:",
                Style::default().fg(colors.muted),
            )));
            // Wrap long description
            let desc = &sink.description;
            if desc.len() > 16 {
                for chunk in desc.as_bytes().chunks(16) {
                    let s = String::from_utf8_lossy(chunk);
                    mode_lines.push(Line::from(Span::styled(
                        format!("  {}", s),
                        Style::default().fg(colors.accent),
                    )));
                }
            } else {
                mode_lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(colors.accent),
                )));
            }
        }
    }

    frame.render_widget(Paragraph::new(mode_lines), mode_inner);

    // ── Right column: sink list ──────────────────────────────────────────────

    let list_title = match ads.mode {
        AudioOutputMode::All => " All Devices ",
        AudioOutputMode::Standard => " Standard Devices ",
        AudioOutputMode::Bluetooth => " Bluetooth Devices ",
    };

    let device_block = Block::default()
        .borders(Borders::ALL)
        .title(list_title)
        .border_style(if ads.focus == 1 {
            Style::default().fg(colors.border_focused)
        } else {
            Style::default().fg(colors.border_unfocused)
        });

    let device_inner = device_block.inner(cols[1]);
    frame.render_widget(device_block, cols[1]);

    let filtered = ads.filtered_sinks();
    let configured = state.config.audio_output_device.as_deref();
    // Reserve last line for help bar
    let max_visible = device_inner.height.saturating_sub(1) as usize;

    if filtered.is_empty() {
        let msg = match ads.mode {
            AudioOutputMode::Bluetooth =>
                " No Bluetooth sinks found.\n \n  ➜ Pair your device from the OS:\n    Linux:   bluetoothctl pair <MAC>\n    Windows: Settings → Bluetooth\n \n  Then press 'r' to refresh.",
            AudioOutputMode::Standard =>
                " No standard output sinks found.\n Press 'r' to refresh.",
            AudioOutputMode::All =>
                " No audio sinks found.\n Press 'r' to refresh.",
        };
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(colors.muted)),
            device_inner,
        );
        return;
    }

    let scroll = ads.scroll_offset.min(filtered.len().saturating_sub(1));
    let visible = filtered.iter().enumerate().skip(scroll).take(max_visible);

    let mut lines: Vec<Line> = Vec::new();
    for (abs_idx, sink) in visible {
        let is_highlighted = abs_idx == ads.selected_index && ads.focus == 1;
        let is_active = configured.map_or(false, |c| c == sink.name);

        let bt_icon = if sink.is_bluetooth { "📶" } else { "🔊" };
        let active_mark = if is_active { "●" } else { " " };
        let cursor = if is_highlighted { "▶" } else { " " };
        // Show description (human name) + state if suspended
        let state_hint = if sink.state.to_uppercase() == "SUSPENDED" { " [off]" } else { "" };
        let label = format!(" {} {} {} {}{}", cursor, active_mark, bt_icon, sink.description, state_hint);

        let style = if is_highlighted {
            Style::default()
                .fg(colors.highlight_fg)
                .bg(colors.highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(colors.accent)
        } else if sink.state.to_uppercase() == "SUSPENDED" {
            Style::default().fg(colors.muted)
        } else {
            Style::default().fg(colors.song)
        };

        lines.push(Line::from(Span::styled(label, style)));
    }

    frame.render_widget(Paragraph::new(lines), device_inner);

    // ── Help bar ─────────────────────────────────────────────────────────────
    let help_text = if ads.focus == 1 {
        " ↑↓ Navigate  Enter Select  Del Reset  ←/Tab Filter"
    } else {
        " ↑↓ Filter  →/Tab Devices  r Refresh"
    };

    if device_inner.height > 0 {
        let help_area = Rect::new(
            device_inner.x,
            device_inner.y + device_inner.height.saturating_sub(1),
            device_inner.width,
            1,
        );
        frame.render_widget(
            Paragraph::new(help_text)
                .style(Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
            help_area,
        );
    }
}

