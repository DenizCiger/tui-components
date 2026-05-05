use crate::ui::layout::centered_rect_fixed;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct SettingsItemView {
    pub keys: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct SettingsSectionView {
    pub title: String,
    pub items: Vec<SettingsItemView>,
}

#[derive(Debug, Clone)]
pub struct SettingsModal<'a> {
    pub title: &'a str,
    pub sections: Vec<SettingsSectionView>,
    pub scroll: u16,
    pub width_percent: u16,
    pub height_percent: u16,
    pub key_width: usize,
}

impl From<crate::shortcuts::ShortcutDisplay> for SettingsItemView {
    fn from(value: crate::shortcuts::ShortcutDisplay) -> Self {
        Self {
            keys: value.keys.to_owned(),
            action: value.action.to_owned(),
        }
    }
}

impl From<crate::shortcuts::ShortcutSection> for SettingsSectionView {
    fn from(value: crate::shortcuts::ShortcutSection) -> Self {
        Self {
            title: value.title.to_owned(),
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> SettingsModal<'a> {
    pub fn new(title: &'a str, sections: Vec<SettingsSectionView>) -> Self {
        Self {
            title,
            sections,
            scroll: 0,
            width_percent: 80,
            height_percent: 80,
            key_width: 16,
        }
    }

    pub fn from_shortcuts(
        title: &'a str,
        sections: Vec<crate::shortcuts::ShortcutSection>,
    ) -> Self {
        Self::new(title, sections.into_iter().map(Into::into).collect())
    }

    pub fn render(&self, frame: &mut Frame, outer: Rect, theme: Theme) {
        let width = ((outer.width as u32 * self.width_percent.min(100) as u32) / 100) as u16;
        let height = ((outer.height as u32 * self.height_percent.min(100) as u32) / 100) as u16;
        let area = centered_rect_fixed(width.max(1), height.max(1), outer);
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(theme.brand));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        let lines = self.lines(theme);
        let total = lines.len() as u16;
        let viewport = layout[0].height;
        let max_scroll = total.saturating_sub(viewport);
        let clamped = self.scroll.min(max_scroll);
        frame.render_widget(Paragraph::new(lines).scroll((clamped, 0)), layout[0]);

        let footer = if max_scroll > 0 {
            format!("↑/↓ scroll · {clamped}/{max_scroll} · Esc or ? to close")
        } else {
            "Esc or ? to close".to_owned()
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                footer,
                Style::default().fg(theme.neutral_gray),
            ))),
            layout[1],
        );
    }

    fn lines(&self, theme: Theme) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for section in &self.sections {
            lines.push(Line::from(Span::styled(
                section.title.clone(),
                Style::default().fg(theme.brand).add_modifier(Modifier::BOLD),
            )));
            for item in &section.items {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<width$}", item.keys, width = self.key_width),
                        Style::default().fg(theme.warning),
                    ),
                    Span::raw(item.action.clone()),
                ]));
            }
            lines.push(Line::from(""));
        }
        lines
    }
}
