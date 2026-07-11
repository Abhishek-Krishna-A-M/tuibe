use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;

const ACCENT: Color = Color::Rgb(124, 58, 237);
const HIGHLIGHT_BG: Color = Color::Rgb(124, 58, 237);

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max.saturating_sub(3)).collect::<String>())
    }
}

pub fn render_queue(frame: &mut ratatui::Frame, app: &App, area: Rect, full_width: bool) {
    if app.queue.is_empty() {
        let p = Paragraph::new(" Queue is empty")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    }

    let vis_height = area.height as usize;
    let sel = app.queue_selected;
    let scroll = if full_width && sel >= vis_height {
        sel - vis_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();

    let status = match app.playback_state {
        crate::app::PlaybackState::Playing => "▶",
        crate::app::PlaybackState::Paused => "⏸",
        crate::app::PlaybackState::Stopped => "■",
    };

    let visible_range = if full_width {
        let start = scroll;
        let end = (scroll + vis_height).min(app.queue.len());
        start..end
    } else {
        let start = scroll;
        let end = (scroll + vis_height).min(app.queue.len());
        start..end
    };

    for i in visible_range {
        let track = &app.queue[i];
        let selected = full_width && i == app.queue_selected;
        let is_current = i == app.queue_index;

        let style = if selected {
            Style::default().bg(HIGHLIGHT_BG).fg(Color::White)
        } else if is_current {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(Color::Gray)
        };

        let number_style = if selected {
            Style::default().bg(HIGHLIGHT_BG).fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let marker = if is_current { format!(" {} ", status) } else { "   ".to_string() };
        let num = format!("{:>2}. ", i + 1);

        let title = if full_width { track.title.clone() } else { truncate(&track.title, 22) };
        let artist = if full_width { track.artist.clone() } else { truncate(&track.artist, 12) };

        lines.push(Line::from(vec![
            Span::styled(marker, number_style),
            Span::styled(num, number_style),
            Span::styled(title, style),
            Span::styled(format!("  {}", artist), style),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}
