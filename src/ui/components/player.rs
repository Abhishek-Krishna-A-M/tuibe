use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, PlaybackState};

const PROGRESS_CHARS: &[char] = &['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

fn smooth_progress(ratio: f64, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    let filled = (ratio * width as f64) as usize;
    let frac = (ratio * width as f64) - filled as f64;
    let frac_idx = (frac * 8.0) as usize;

    let mut spans = Vec::with_capacity(width);

    for _ in 0..filled.min(width) {
        spans.push(Span::styled(
            "█",
            Style::default().fg(Color::Rgb(124, 58, 237)),
        ));
    }

    if filled < width {
        if frac_idx > 0 {
            spans.push(Span::styled(
                PROGRESS_CHARS[frac_idx.min(7)].to_string(),
                Style::default().fg(Color::Rgb(124, 58, 237)),
            ));
        }
        while spans.len() < width {
            spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
        }
    }

    Line::from(spans)
}

pub fn render_now_playing_bar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    let inner_h = inner.height as usize;

    frame.render_widget(block, area);

    match &app.current_track {
        Some(track) => {
            let icon = match app.playback_state {
                PlaybackState::Playing => "▶",
                PlaybackState::Paused => "⏸",
                PlaybackState::Stopped => "■",
            };

            let liked = if app.playlists.is_liked(&track.id) {
                " ♥"
            } else {
                "  "
            };

            let pos = super::super::fmt_dur(app.position);
            let dur = super::super::fmt_dur(app.duration);

            let progress = if app.duration.as_secs() > 0 {
                app.position.as_secs_f64() / app.duration.as_secs_f64()
            } else {
                0.0
            };

            let mut y = inner.y;

            if inner_h >= 1 {
                let info = Line::from(vec![
                    Span::styled(
                        format!(" {} ", icon),
                        Style::default().fg(Color::Rgb(124, 58, 237)),
                    ),
                    Span::styled(
                        track.title.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" — ", Style::default().fg(Color::DarkGray)),
                    Span::styled(track.artist.clone(), Style::default().fg(Color::Gray)),
                    Span::styled(liked, Style::default().fg(Color::Red)),
                ]);
                let info_area = Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                };
                frame.render_widget(Paragraph::new(info), info_area);
                y += 1;
            }

            if inner_h >= 2 {
                let time_width = pos.len() + dur.len() + 3;
                let bar_width = (inner.width as usize).saturating_sub(time_width);

                let bar_line = smooth_progress(progress, bar_width);

                let time_span = Span::styled(
                    format!(" {} {} ", pos, dur),
                    Style::default().fg(Color::DarkGray),
                );

                let mut spans = bar_line.spans;
                spans.push(time_span);

                let bar_area = Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                };
                frame.render_widget(Paragraph::new(Line::from(spans)), bar_area);
            }
        }
        None => {
            if inner_h >= 1 {
                let p = Paragraph::new(Line::from(Span::styled(
                    "  No track playing",
                    Style::default().fg(Color::DarkGray),
                )));
                frame.render_widget(p, inner);
            }
        }
    }
}
