use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

pub fn to_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn truncate_text(value: &str, width: usize) -> String {
    fit_text(value, width).trim_end().to_owned()
}

pub fn fit_text(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(value) <= width {
        return format!("{value:<width$}");
    }
    let mut result = String::new();
    for character in value.chars() {
        if UnicodeWidthStr::width(result.as_str())
            + UnicodeWidthStr::width(character.encode_utf8(&mut [0; 4]))
            > width.saturating_sub(1)
        {
            break;
        }
        result.push(character);
    }
    result.push('…');
    while UnicodeWidthStr::width(result.as_str()) < width {
        result.push(' ');
    }
    result
}

pub fn line_with_right(
    left: &str,
    right: &str,
    width: usize,
    left_style: Style,
    right_style: Style,
) -> Line<'static> {
    if right.is_empty() {
        return Line::from(Span::styled(truncate_text(left, width), left_style));
    }
    let right_width = UnicodeWidthStr::width(right);
    if width <= right_width + 1 {
        return Line::from(Span::styled(truncate_text(right, width), right_style));
    }
    let max_left_width = width.saturating_sub(right_width + 1);
    let left_text = truncate_text(left, max_left_width);
    let gap = width.saturating_sub(UnicodeWidthStr::width(left_text.as_str()) + right_width);
    Line::from(vec![
        Span::styled(left_text, left_style),
        Span::raw(" ".repeat(gap)),
        Span::styled(right.to_owned(), right_style),
    ])
}

pub fn render_input_text(value: &str, cursor: usize, mask: bool) -> String {
    let value = if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_owned()
    };
    if value.is_empty() {
        return "_".to_owned();
    }
    if cursor >= value.chars().count() {
        return format!("{value}_");
    }
    value
}
