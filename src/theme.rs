use ratatui::style::Color;

pub struct Theme {
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Rgb(124, 58, 237), // #7c3aed
            text: Color::White,
            muted: Color::DarkGray,
            border: Color::DarkGray,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
        }
    }
}
