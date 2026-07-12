use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, Screen};

const ACCENT: Color = Color::Rgb(124, 58, 237);

pub fn render_status_bar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let vol_pct = (app.volume * 100.0) as u32;
    let vol_icon = if app.volume == 0.0 {
        ""
    } else if app.volume < 0.33 {
        ""
    } else {
        ""
    };

    let repeat_icon = app.repeat_mode.icon();
    let shuffle_icon = if app.shuffle { " " } else { "" };
    let autoplay_icon = if app.autoplay { " " } else { "" };

    let hint = match app.screen {
        Screen::Search => " / search  ↑↓ nav  ↵ play  ␣ pause  ♥ like  A add_pl",
        Screen::Queue => " Tab back  ↑↓ nav  d remove",
        Screen::PlaylistBrowser => " ↵ open  n new  I import  d delete  Esc back",
        Screen::PlaylistDetail => " ↵ play  d remove  Esc back",
    };

    let line = Line::from(vec![
        Span::styled(
            format!("{}{:>3}%", vol_icon, vol_pct),
            Style::default().fg(ACCENT),
        ),
        Span::styled(shuffle_icon, Style::default().fg(Color::Yellow)),
        Span::styled(repeat_icon, Style::default().fg(Color::Yellow)),
        Span::styled(autoplay_icon, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("  Q{:>2}", app.queue.len().min(99)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("  {}b {}s", app.cava_bar_count, app.cava_sensitivity),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" │", Style::default().fg(Color::DarkGray)),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
        Span::styled("  []sens  {}bars  t autoplay  v vis", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
