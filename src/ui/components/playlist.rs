use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use crate::app::App;

pub fn render_playlist_browser(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Playlists ");

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for (i, pl) in app.playlists.playlists.iter().enumerate() {
        let style = if i == app.playlist_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let icon = if i == app.playlist_selected { "▶" } else { " " };
        let count = pl.tracks.len();
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", icon), style.fg(Color::DarkGray)),
            Span::styled(pl.name.clone(), style),
            Span::styled(
                format!(" ({} tracks)", count),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    if inner.height < lines.len() as u16 {
        lines.truncate(inner.height as usize);
    }
    while lines.len() < inner.height as usize {
        lines.push(Line::from(""));
    }

    let help = format!(" [n] new  [d] delete  [Enter] open  [P] back  [q] quit");
    let footer = Line::from(Span::styled(help, Style::default().fg(Color::DarkGray)));
    frame.render_widget(
        Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }),
        inner,
    );

    let footer_y = area.y + area.height - 1;
    let footer_area = Rect {
        x: area.x,
        y: footer_y,
        width: area.width,
        height: 1,
    };
    frame.render_widget(footer, footer_area);
}

pub fn render_playlist_detail(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let playlist = app
        .playlist_detail_id
        .as_ref()
        .and_then(|id| app.playlists.playlists.iter().find(|p| p.id == *id));

    let (name, tracks) = match playlist {
        Some(pl) => (pl.name.as_str(), pl.tracks.as_slice()),
        None => {
            let p = Paragraph::new("Playlist not found").style(Style::default().fg(Color::Red));
            frame.render_widget(p, area);
            return;
        }
    };

    if tracks.is_empty() {
        let p = Paragraph::new("Empty playlist — add tracks from search with f").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" {} ", name)),
        );
        frame.render_widget(p, area);
        return;
    }

    render_track_table_custom(frame, tracks, app.selected_index, area, name);
}

fn render_track_table_custom(
    frame: &mut ratatui::Frame,
    tracks: &[crate::search::Track],
    selected: usize,
    area: Rect,
    title: &str,
) {
    let header = Row::new(vec![
        Cell::from(" # ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Title ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Artist ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Time ").style(Style::default().fg(Color::DarkGray)),
    ]);

    let rows: Vec<Row> = tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Row::new(vec![
                Cell::from(format!(" {} ", i + 1)).style(style.fg(Color::DarkGray)),
                Cell::from(track.title.clone()).style(style),
                Cell::from(track.artist.clone()).style(style.fg(Color::DarkGray)),
                Cell::from(track.duration_string()).style(style.fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(4),
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Percentage(35),
            ratatui::layout::Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" {} ", title)),
    );

    frame.render_widget(table, area);
}
