use super::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

pub fn filter_chip(label: &str, active: bool, theme: Theme) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(if active { theme.neutral_white } else { theme.neutral_gray })
            .bg(if active { theme.panel_selected } else { Color::Reset }),
    )
}

pub fn tab_span(label: &str, active: bool, theme: Theme) -> Span<'static> {
    Span::styled(
        label.to_owned(),
        Style::default()
            .fg(if active { theme.neutral_black } else { theme.neutral_white })
            .bg(if active { theme.brand } else { theme.panel_alternate })
            .add_modifier(if active { Modifier::BOLD } else { Modifier::empty() }),
    )
}

pub fn styled_cell(text: &str, bg: Option<Color>, fg: Option<Color>) -> Span<'static> {
    Span::styled(
        text.to_owned(),
        Style::default()
            .bg(bg.unwrap_or(Color::Reset))
            .fg(fg.unwrap_or(Color::Reset)),
    )
}

pub fn login_field_line(
    label: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    mask: bool,
    theme: Theme,
) -> Line<'static> {
    let rendered = if value.is_empty() {
        placeholder.to_owned()
    } else if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_owned()
    };
    Line::from(vec![
        Span::styled(
            format!("{}{}: ", if focused { "> " } else { "  " }, label),
            Style::default()
                .fg(if focused { theme.brand } else { theme.neutral_gray })
                .add_modifier(if focused { Modifier::BOLD } else { Modifier::empty() }),
        ),
        Span::raw(rendered),
    ])
}

pub fn centered_message_lines(
    message: &str,
    height: u16,
    width: u16,
    style: Style,
) -> Vec<Line<'static>> {
    if height == 0 || width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let top_padding = usize::from(height.saturating_sub(1) / 2);
    for _ in 0..top_padding {
        lines.push(Line::from(""));
    }
    let message_width = UnicodeWidthStr::width(message);
    let left_pad = usize::from(width).saturating_sub(message_width) / 2;
    lines.push(Line::from(Span::styled(
        format!("{}{}", " ".repeat(left_pad), crate::ui::text::truncate_text(message, usize::from(width))),
        style,
    )));
    lines
}
