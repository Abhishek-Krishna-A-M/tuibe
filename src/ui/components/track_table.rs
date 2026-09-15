use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::search::{Album, Artist, ScopedResults, Track};

const ACCENT: Color = Color::Rgb(124, 58, 237);

fn scroll_offset(selected: usize, vis_height: usize) -> usize {
    if vis_height == 0 {
        return 0;
    }
    if selected >= vis_height {
        selected - vis_height + 1
    } else {
        0
    }
}

fn row_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(Color::White)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn dim_style(selected: bool, base: Style) -> Style {
    if selected {
        base
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn block(title: String) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
}

pub fn render_scoped_results(
    frame: &mut ratatui::Frame,
    results: &ScopedResults,
    selected: usize,
    area: Rect,
) {
    match results {
        ScopedResults::Tracks(tracks) => render_track_table(frame, tracks, selected, area),
        ScopedResults::Artists(artists) => render_artist_table(frame, artists, selected, area),
        ScopedResults::Albums(albums) => render_album_table(frame, albums, selected, area),
    }
}

pub fn render_track_table(
    frame: &mut ratatui::Frame,
    tracks: &[Track],
    selected: usize,
    area: Rect,
) {
    let vis_height = area.height.saturating_sub(2) as usize;
    let scroll = scroll_offset(selected, vis_height);

    let visible: Vec<&Track> = tracks.iter().skip(scroll).take(vis_height).collect();
    let display_selected = selected.saturating_sub(scroll);

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
            let style = row_style(is_sel);
            Row::new(vec![
                Cell::from(format!(" {} ", scroll + i + 1)).style(dim_style(is_sel, style)),
                Cell::from(track.title.clone()).style(style),
                Cell::from(track.artist.clone()).style(dim_style(is_sel, style)),
                Cell::from(track.duration_string()).style(dim_style(is_sel, style)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(block(format!(" Songs ({}) ", tracks.len())));

    frame.render_widget(table, area);
}

pub fn render_artist_table(
    frame: &mut ratatui::Frame,
    artists: &[Artist],
    selected: usize,
    area: Rect,
) {
    let vis_height = area.height.saturating_sub(2) as usize;
    let scroll = scroll_offset(selected, vis_height);

    let visible: Vec<&Artist> = artists.iter().skip(scroll).take(vis_height).collect();
    let display_selected = selected.saturating_sub(scroll);

    let header = Row::new(vec![
        Cell::from(" # ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Artist ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Subscribers ").style(Style::default().fg(Color::DarkGray)),
    ]);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, artist)| {
            let is_sel = i == display_selected;
            let style = row_style(is_sel);
            Row::new(vec![
                Cell::from(format!(" {} ", scroll + i + 1)).style(dim_style(is_sel, style)),
                Cell::from(format!(" {}", artist.name)).style(style),
                Cell::from(artist.subscribers.clone().unwrap_or_default())
                    .style(dim_style(is_sel, style)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(55),
            Constraint::Percentage(35),
        ],
    )
    .header(header)
    .block(block(format!(
        " Artists ({}) — ↵ top songs • a enqueue ",
        artists.len()
    )));

    frame.render_widget(table, area);
}

pub fn render_album_table(
    frame: &mut ratatui::Frame,
    albums: &[Album],
    selected: usize,
    area: Rect,
) {
    let vis_height = area.height.saturating_sub(2) as usize;
    let scroll = scroll_offset(selected, vis_height);

    let visible: Vec<&Album> = albums.iter().skip(scroll).take(vis_height).collect();
    let display_selected = selected.saturating_sub(scroll);

    let header = Row::new(vec![
        Cell::from(" # ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Album / Movie ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Artist ").style(Style::default().fg(Color::DarkGray)),
        Cell::from(" Year ").style(Style::default().fg(Color::DarkGray)),
    ]);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, album)| {
            let is_sel = i == display_selected;
            let style = row_style(is_sel);
            Row::new(vec![
                Cell::from(format!(" {} ", scroll + i + 1)).style(dim_style(is_sel, style)),
                Cell::from(album.title.clone()).style(style),
                Cell::from(album.artist.clone()).style(dim_style(is_sel, style)),
                Cell::from(album.year.clone().unwrap_or_default())
                    .style(dim_style(is_sel, style)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(block(format!(
        " Albums / Movies ({}) — ↵ tracks • a enqueue ",
        albums.len()
    )));

    frame.render_widget(table, area);
}
