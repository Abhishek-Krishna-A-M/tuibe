use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::search::Track;

const ACCENT: Color = Color::Rgb(124, 58, 237);

pub fn render_track_table(
    frame: &mut ratatui::Frame,
    tracks: &[Track],
    selected: usize,
    area: Rect,
) {
    let vis_height = area.height.saturating_sub(2) as usize;
    let scroll = if selected >= vis_height {
        selected - vis_height + 1
    } else {
        0
    };

    let visible: Vec<&Track> = tracks.iter().skip(scroll).take(vis_height).collect();
    let display_selected = selected - scroll;

    let header = Row::new(vec![
        Cell::from(" # ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Title ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Artist ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Time ").style(Style::default().fg(Color::DarkGray)),
    ]);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let is_sel = i == display_selected;
            let style = if is_sel {
                Style::default()
                    .fg(Color::White)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Row::new(vec![
                Cell::from(format!(" {} ", i + 1))
                    .style(if is_sel {
                        style.fg(Color::White).bg(ACCENT)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
                Cell::from(track.title.clone()).style(style),
                Cell::from(track.artist.clone())
                    .style(if is_sel { style } else { Style::default().fg(Color::DarkGray) }),
                Cell::from(track.duration_string())
                    .style(if is_sel { style } else { Style::default().fg(Color::DarkGray) }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Results ({}) ", tracks.len())),
    );

    frame.render_widget(table, area);
}
