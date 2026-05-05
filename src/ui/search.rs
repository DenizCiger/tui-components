use crate::input::SearchModalState;
use crate::ui::layout::centered_rect_percent;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Build spans for `text` where `highlight` (sorted unique char indices) is
/// rendered with `highlight_style` and the rest carries `base_style`. Use to
/// surface which characters of a candidate were matched by the search
/// algorithm.
pub fn highlight_spans(
    text: &str,
    highlight: &[usize],
    base_style: Style,
    highlight_style: Style,
) -> Vec<Span<'static>> {
    if highlight.is_empty() {
        return vec![Span::styled(text.to_owned(), base_style)];
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let mut buf_is_highlight = false;
    let mut hi_iter = highlight.iter().copied().peekable();
    for (idx, ch) in text.chars().enumerate() {
        let is_hi = matches!(hi_iter.peek(), Some(&h) if h == idx);
        if is_hi {
            hi_iter.next();
        }
        if !buf.is_empty() && is_hi != buf_is_highlight {
            let style = if buf_is_highlight { highlight_style } else { base_style };
            spans.push(Span::styled(std::mem::take(&mut buf), style));
        }
        buf.push(ch);
        buf_is_highlight = is_hi;
    }
    if !buf.is_empty() {
        let style = if buf_is_highlight { highlight_style } else { base_style };
        spans.push(Span::styled(buf, style));
    }
    spans
}

pub struct SearchModalRow<'a> {
    pub spans: Vec<Span<'a>>,
}

impl<'a> SearchModalRow<'a> {
    pub fn new(spans: Vec<Span<'a>>) -> Self {
        Self { spans }
    }
}

pub struct SearchModalCategory<'a> {
    pub label: &'a str,
    pub active: bool,
}

pub struct SearchModal<'a> {
    pub title: &'a str,
    pub hint: &'a str,
    pub state: &'a SearchModalState,
    pub rows: Vec<SearchModalRow<'a>>,
    pub categories: Option<Vec<SearchModalCategory<'a>>>,
    pub empty_text: &'a str,
    pub theme: Theme,
}

impl<'a> SearchModal<'a> {
    /// Render centered at 70% width × 60% height of the frame.
    pub fn render(self, frame: &mut Frame) {
        let area = centered_rect_percent(70, 60, frame.area());
        self.render_in(frame, area);
    }

    pub fn render_in(self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.to_owned())
            .border_style(Style::default().fg(self.theme.brand));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::with_capacity(self.rows.len() + 4);
        lines.push(Line::from(format!("> {}", self.state.input.value)));

        if let Some(cats) = &self.categories {
            for line in build_category_ribbon(cats, inner.width as usize, self.theme) {
                lines.push(line);
            }
        }

        lines.push(Line::from(Span::styled(
            self.hint.to_owned(),
            Style::default().fg(self.theme.neutral_gray),
        )));
        lines.push(Line::from(""));

        let header_height = lines.len();
        let row_capacity = (inner.height as usize).saturating_sub(header_height);

        if self.rows.is_empty() {
            lines.push(Line::from(Span::styled(
                self.empty_text.to_owned(),
                Style::default().fg(self.theme.neutral_gray),
            )));
        } else {
            let total = self.rows.len();
            let selected = self.state.selected.min(total - 1);
            let visible_start = if row_capacity == 0 {
                selected
            } else {
                selected.saturating_sub(row_capacity.saturating_sub(1))
            };
            for (local, row) in self.rows.into_iter().skip(visible_start).take(row_capacity).enumerate() {
                let idx = visible_start + local;
                let is_selected = idx == selected;
                let mut spans: Vec<Span> = Vec::with_capacity(row.spans.len() + 1);
                spans.push(Span::styled(
                    if is_selected { "> " } else { "  " }.to_owned(),
                    Style::default().fg(if is_selected {
                        self.theme.brand
                    } else {
                        self.theme.neutral_gray
                    }),
                ));
                spans.extend(row.spans);
                lines.push(Line::from(spans));
            }
        }

        frame.render_widget(Paragraph::new(lines), inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn hi_style() -> Style {
        Style::default().fg(Color::Indexed(220)).add_modifier(Modifier::BOLD)
    }

    #[test]
    fn highlight_empty_indices_returns_single_base_span() {
        let spans = highlight_spans("hello", &[], Style::default(), hi_style());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "hello");
    }

    #[test]
    fn highlight_groups_adjacent_indices() {
        let spans = highlight_spans("Mathematics", &[0, 1, 4], Style::default(), hi_style());
        assert_eq!(
            spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>(),
            vec!["Ma", "th", "e", "matics"]
        );
        assert_eq!(spans[0].style.fg, Some(Color::Indexed(220)));
        assert_eq!(spans[1].style.fg, None);
        assert_eq!(spans[2].style.fg, Some(Color::Indexed(220)));
    }

    #[test]
    fn highlight_handles_unicode_char_indices() {
        let spans = highlight_spans("aäb", &[1], Style::default(), hi_style());
        assert_eq!(
            spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>(),
            vec!["a", "ä", "b"]
        );
    }
}

fn build_category_ribbon<'a>(
    categories: &[SearchModalCategory<'a>],
    max_width: usize,
    theme: Theme,
) -> Vec<Line<'a>> {
    let mut out: Vec<Line<'a>> = Vec::new();
    let mut row_spans: Vec<Span<'a>> = Vec::new();
    let mut row_w = 0usize;
    for cat in categories {
        let style = if cat.active {
            Style::default()
                .fg(theme.neutral_black)
                .bg(theme.brand)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.neutral_gray)
        };
        let chunk = format!(" {} ", cat.label);
        let w = chunk.chars().count() + 1;
        if row_w + w > max_width && !row_spans.is_empty() {
            out.push(Line::from(std::mem::take(&mut row_spans)));
            row_w = 0;
        }
        row_spans.push(Span::styled(chunk, style));
        row_spans.push(Span::raw(" "));
        row_w += w;
    }
    if !row_spans.is_empty() {
        out.push(Line::from(row_spans));
    }
    out
}
