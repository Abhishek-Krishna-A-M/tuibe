use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, InputMode, SearchState};

pub fn render_input_prompt(frame: &mut ratatui::Frame, mode: &InputMode, text: &str, area: Rect) {
    let title = match mode {
        InputMode::Search => " Search ",
        InputMode::NewPlaylist => " New Playlist Name ",
        _ => "",
    };
    let placeholder = match mode {
        InputMode::Search => "Type to search...",
        InputMode::NewPlaylist => "Enter playlist name...",
        _ => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .title(title);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let cursor = Span::styled("▎", Style::default().fg(Color::Rgb(124, 58, 237)));

    let display = if text.is_empty() {
        Line::from(vec![
            Span::styled(placeholder, Style::default().fg(Color::DarkGray)),
            cursor,
        ])
    } else {
        Line::from(vec![
            Span::styled(text, Style::default().fg(Color::White)),
            cursor,
        ])
    };

    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(display), inner);
}

pub fn render_search_area(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let search_layout = crate::ui::search::compute_search_layout(area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let input_text = if app.input_text.is_empty() {
        Span::styled(" Type / to search", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!(" {}", app.input_text),
            Style::default().fg(Color::White),
        )
    };

    frame.render_widget(
        Paragraph::new(Line::from(input_text)).block(input_block),
        search_layout.input,
    );

    match &app.search_state {
        SearchState::Idle => {
            let empty = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Results ");
            frame.render_widget(empty, search_layout.results);
        }
        SearchState::Searching => {
            let loading = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Searching... ");
            frame.render_widget(loading, search_layout.results);
        }
        SearchState::Error(e) => {
            let p = Paragraph::new(e.as_str()).style(Style::default().fg(Color::Red));
            let b = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Error ");
            frame.render_widget(p.block(b), search_layout.results);
        }
        SearchState::Loaded(tracks) => {
            crate::ui::components::render_track_table(
                frame,
                tracks,
                app.selected_index,
                search_layout.results,
            );
        }
    }
}
