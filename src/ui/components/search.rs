use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, InputMode, SearchState};

pub const CURSOR_GLYPH: &str = "▎";
pub const ACCENT: Color = Color::Rgb(124, 58, 237);

/// Clamp a byte index to a char boundary.
fn clamp_cursor(text: &str, cursor: usize) -> usize {
    let mut c = cursor.min(text.len());
    while c > 0 && !text.is_char_boundary(c) {
        c -= 1;
    }
    c
}

/// Build a single-line input with a visible block cursor (`▎`), with
/// horizontal scrolling so long queries stay visible.
pub fn input_line(text: &str, cursor: usize, max_width: usize) -> Line<'static> {
    let cursor = clamp_cursor(text, cursor);
    let cursor_span = Span::styled(CURSOR_GLYPH, Style::default().fg(ACCENT));

    if text.is_empty() {
        return Line::from(vec![cursor_span]);
    }

    // Char-based scroll: keep the cursor in view.
    let chars: Vec<char> = text.chars().collect();
    let cursor_char = text[..cursor].chars().count();
    let width = max_width.max(1);
    let start = if cursor_char >= width {
        cursor_char - width + 1
    } else {
        0
    };
    let end = (start + width).min(chars.len());
    let visible: String = chars[start..end].iter().collect();
    let cursor_in_view = cursor_char - start;

    let before: String = visible.chars().take(cursor_in_view).collect();
    let after: String = visible.chars().skip(cursor_in_view).collect();

    Line::from(vec![
        Span::styled(before, Style::default().fg(Color::White)),
        cursor_span,
        Span::styled(after, Style::default().fg(Color::White)),
    ])
}

pub fn placeholder_line(placeholder: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(placeholder.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(CURSOR_GLYPH, Style::default().fg(ACCENT)),
    ])
}

pub fn render_input_prompt(
    frame: &mut ratatui::Frame,
    mode: &InputMode,
    text: &str,
    cursor: usize,
    area: Rect,
) {
    let (title, placeholder, hint) = match mode {
        InputMode::Search => (
            " Search (artist: / album: / movie: / song:) ",
            "artist:  •  album:  •  movie:  •  song:",
            "Enter search • Esc cancel • ←/→ move cursor",
        ),
        InputMode::NewPlaylist => (
            " New Playlist Name ",
            "Enter playlist name...",
            "Enter save • Esc cancel • ←/→ move cursor",
        ),
        InputMode::ImportPlaylist => (
            " Import Playlist ",
            "Paste a YouTube Music playlist URL or ID...",
            "Enter import • Esc cancel • ←/→ move cursor",
        ),
        InputMode::AddToPlaylist => (" Add to Playlist ", "", "Enter add • Esc cancel"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::White))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let max_width = inner.width as usize;
    let display = if text.is_empty() && !placeholder.is_empty() {
        placeholder_line(placeholder)
    } else {
        input_line(text, cursor, max_width)
    };
    let input_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1.min(inner.height),
    };
    frame.render_widget(Paragraph::new(display), input_area);

    if inner.height >= 2 && !hint.is_empty() {
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                hint,
                Style::default().fg(Color::DarkGray),
            ))),
            hint_area,
        );
    }

    if inner.height >= 3 && matches!(mode, InputMode::Search) {
        let help_area = Rect {
            x: inner.x,
            y: inner.y + 2,
            width: inner.width,
            height: inner.height - 2,
        };
        let help = vec![
            Line::from(Span::styled(
                "Prefixes: artist:  album:  movie:  song:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "No prefix = song search. Enter on an artist/album loads its tracks.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(help), help_area);
    }
}

pub fn render_search_area(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let search_layout = crate::ui::search::compute_search_layout(area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let input_text = if app.input_text.is_empty() {
        Span::styled(
            " Type / to search — artist:  album:  movie:  song:",
            Style::default().fg(Color::DarkGray),
        )
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
            let empty = Paragraph::new(vec![
                Line::from(Span::styled(
                    " No results yet — press / and try:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "   artist:   album:   movie:   song:",
                    Style::default().fg(Color::Gray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Results "),
            );
            frame.render_widget(empty, search_layout.results);
        }
        SearchState::Searching => {
            let loading = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Searching... ");
            frame.render_widget(loading, search_layout.results);
            let p = Paragraph::new(format!(
                " Searching {} for \"{}\"...",
                app.search_scope.short_label(),
                app.search_query
            ))
            .style(Style::default().fg(Color::DarkGray));
            let inner = Rect {
                x: search_layout.results.x + 1,
                y: search_layout.results.y + 1,
                width: search_layout.results.width.saturating_sub(2),
                height: search_layout.results.height.saturating_sub(2),
            };
            frame.render_widget(p, inner);
        }
        SearchState::LoadingDetail(label) => {
            let loading = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" Loading {}... ", label));
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
        SearchState::Loaded(results) => {
            crate::ui::components::render_scoped_results(
                frame,
                results,
                app.selected_index,
                search_layout.results,
            );
        }
    }
}
