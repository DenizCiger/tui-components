use crate::ui::layout::centered_rect_fixed;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct LoginFieldView<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub placeholder: &'a str,
    pub focused: bool,
    pub masked: bool,
}

#[derive(Debug, Clone)]
pub struct LoginModal<'a> {
    pub title: &'a str,
    pub help_lines: Vec<&'a str>,
    pub fields: Vec<LoginFieldView<'a>>,
    pub submit_focused: bool,
    pub saved_account: Option<String>,
    pub error: Option<&'a str>,
    pub warning: Option<&'a str>,
    pub busy: bool,
    pub busy_label: &'a str,
    pub submit_label: &'a str,
    pub footer: &'a str,
    pub width: u16,
    pub min_height: u16,
}

impl<'a> LoginModal<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            help_lines: Vec::new(),
            fields: Vec::new(),
            submit_focused: false,
            saved_account: None,
            error: None,
            warning: None,
            busy: false,
            busy_label: "Working…",
            submit_label: "Submit",
            footer: "Tab/Shift+Tab or ↑/↓ fields · Enter submit · Alt+V show password · Esc quit",
            width: 72,
            min_height: 16,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        let dynamic_height = 4
            + self.help_lines.len() as u16
            + self.fields.len().saturating_mul(2) as u16
            + u16::from(self.saved_account.is_some())
            + u16::from(self.error.is_some())
            + u16::from(self.warning.is_some())
            + 3;
        let height = self.min_height.max(dynamic_height).min(area.height);
        let width = self.width.min(area.width);
        let area = centered_rect_fixed(width, height, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(theme.brand));
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);

        let mut constraints = Vec::new();
        constraints.push(Constraint::Length(self.help_lines.len() as u16 + 1));
        for _ in &self.fields {
            constraints.push(Constraint::Length(2));
        }
        constraints.push(Constraint::Length(2));
        constraints.push(Constraint::Min(1));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(inner);

        let help = self
            .help_lines
            .iter()
            .map(|line| Line::from(Span::styled((*line).to_owned(), Style::default().fg(theme.neutral_gray))))
            .chain(std::iter::once(Line::from("")))
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(help).wrap(Wrap { trim: false }), chunks[0]);

        for (idx, field) in self.fields.iter().enumerate() {
            frame.render_widget(Paragraph::new(render_field(field, theme)), chunks[idx + 1]);
        }

        let submit_style = if self.submit_focused {
            Style::default().fg(theme.neutral_white).bg(theme.brand).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.neutral_gray)
        };
        let submit_text = if self.busy { self.busy_label } else { self.submit_label };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!("  [ {submit_text} ]  "), submit_style))),
            chunks[self.fields.len() + 1],
        );

        let mut footer_lines = Vec::new();
        if let Some(error) = self.error {
            footer_lines.push(Line::from(Span::styled(format!("Error: {error}"), Style::default().fg(theme.error))));
        }
        if let Some(warning) = self.warning {
            footer_lines.push(Line::from(Span::styled(warning.to_owned(), Style::default().fg(theme.warning))));
        }
        if let Some(saved) = &self.saved_account {
            footer_lines.push(Line::from(vec![
                Span::styled("Saved account: ", Style::default().fg(theme.brand)),
                Span::raw(saved.clone()),
                Span::raw(" | Ctrl+L login"),
            ]));
        }
        footer_lines.push(Line::from(Span::styled(self.footer.to_owned(), Style::default().fg(theme.neutral_gray))));
        frame.render_widget(Paragraph::new(footer_lines).wrap(Wrap { trim: false }), chunks[self.fields.len() + 2]);
    }
}

fn render_field(field: &LoginFieldView<'_>, theme: Theme) -> Line<'static> {
    let rendered = if field.value.is_empty() {
        field.placeholder.to_owned()
    } else if field.masked {
        "*".repeat(field.value.chars().count())
    } else {
        field.value.to_owned()
    };
    let label_style = Style::default().fg(theme.neutral_gray);
    let value_style = if field.focused {
        Style::default().fg(theme.neutral_white).add_modifier(Modifier::BOLD)
    } else if field.value.is_empty() {
        Style::default().fg(theme.neutral_gray)
    } else {
        Style::default().fg(theme.neutral_white)
    };
    let prefix = if field.focused { "▌ " } else { "  " };
    Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme.brand)),
        Span::styled(format!("{:<12}", field.label), label_style),
        Span::styled(rendered, value_style),
    ])
}
