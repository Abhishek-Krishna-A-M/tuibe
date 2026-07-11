use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, VisualizerMode};

const BLOCKS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn bar_color(val: f32) -> Color {
    let r = (val * 200.0) as u8;
    let g = (200.0 * (1.0 - (val - 0.5).abs() * 2.0)) as u8;
    let b = (255.0 * (1.0 - val * 0.7)) as u8;
    Color::Rgb(r, g, b)
}

pub fn render_visualizer(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    if app.vis_mode == VisualizerMode::None || app.vis_bars.is_empty() {
        return;
    }

    let w = area.width as usize;
    let h = area.height as usize;
    if w < 4 || h < 1 {
        return;
    }

    let total_levels = h * 8;
    let bar_count = app.vis_bars.len().min(w);
    let ratio = w as f32 / bar_count as f32;
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(h);

    for row in 0..h {
        let row_top = (h - 1 - row) * 8;
        let row_bot = row_top + 8;
        let mut spans = Vec::with_capacity(w);

        for col in 0..w {
            let bi = (col as f32 / ratio).min(bar_count as f32 - 1.0) as usize;
            let val = app.vis_bars[bi];
            let peak = app.vis_peaks[bi];
            let levels = (val * total_levels as f32) as usize;
            let color = bar_color(val);

            if levels >= row_bot {
                spans.push(Span::styled("█", Style::default().fg(color)));
            } else if levels > row_top {
                let idx = ((levels - row_top) as f32 / 8.0 * 8.0) as usize;
                spans.push(Span::styled(
                    BLOCKS[idx.min(8)].to_string(),
                    Style::default().fg(color),
                ));
            } else {
                let pl = (peak * total_levels as f32) as usize;
                if row == 0 && pl > row_top && pl <= row_bot {
                    spans.push(Span::styled("▔", Style::default().fg(Color::White)));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
        }

        for _ in spans.len()..w {
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}