//! Audio device selector widget

use ratatui::{
    text::Span,
    widgets::{Block, Borders, List, ListItem},
};

use crate::ui::theme::ThemeColors;

pub struct AudioDeviceSelector<'a> {
    devices: &'a [String],
    selected_index: usize,
    colors: ThemeColors,
}

impl<'a> AudioDeviceSelector<'a> {
    pub fn new(devices: &'a [String], selected_index: usize, colors: ThemeColors) -> Self {
        Self {
            devices,
            selected_index,
            colors,
        }
    }

    pub fn render_list(&self) -> List<'a> {
        let items: Vec<ListItem> = self
            .devices
            .iter()
            .enumerate()
            .map(|(idx, device_name)| {
                let content = if idx == self.selected_index {
                    format!("▶ {}", device_name)
                } else {
                    format!("  {}", device_name)
                };
                let style = if idx == self.selected_index {
                    ratatui::style::Style::default().fg(self.colors.accent)
                } else {
                    ratatui::style::Style::default().fg(self.colors.song)
                };
                ListItem::new(Span::styled(content, style))
            })
            .collect();

        List::new(items)
            .block(
                Block::default()
                    .title("Audio Output Devices")
                    .borders(Borders::ALL)
                    .border_style(
                        ratatui::style::Style::default()
                            .fg(self.colors.accent),
                    ),
            )
            .style(ratatui::style::Style::default().fg(self.colors.song))
    }
}
