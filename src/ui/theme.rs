use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub brand: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub neutral_white: Color,
    pub neutral_black: Color,
    pub neutral_gray: Color,
    pub neutral_bright_black: Color,
    pub panel_header: Color,
    pub panel_selected: Color,
    pub panel_alternate: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            brand: Color::Indexed(39),
            warning: Color::Indexed(220),
            error: Color::Indexed(196),
            success: Color::Indexed(84),
            neutral_white: Color::Indexed(15),
            neutral_black: Color::Indexed(16),
            neutral_gray: Color::Indexed(244),
            neutral_bright_black: Color::Indexed(240),
            panel_header: Color::Indexed(238),
            panel_selected: Color::Indexed(24),
            panel_alternate: Color::Indexed(236),
        }
    }
}
