use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::style::{Color, Style};
use ratatui::widgets::Borders;

use crate::app::{App, InputMode, Screen};

pub mod chunks;
pub mod components;
pub mod search;

pub fn render(frame: &mut ratatui::Frame, app: &App) {
    let layout = chunks::compute_layout(frame.area(), app.vis_mode);

    match app.input_mode {
        Some(InputMode::AddToPlaylist) => {
            render_playlist_picker(frame, app, layout.content);
        }
        Some(InputMode::ImportPlaylist) => {
            // Cursor visible here too — same prompt component as search/new.
            components::search::render_input_prompt(
                frame,
                &InputMode::ImportPlaylist,
                &app.input_text,
                app.cursor,
                layout.content,
            );
        }
        Some(ref mode) => {
            components::search::render_input_prompt(frame, mode, &app.input_text, app.cursor, layout.content);
        }
        None => match app.screen {
        Screen::PlaylistBrowser => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Playlists ");
            let inner = block.inner(layout.content);
            frame.render_widget(block, layout.content);
            components::playlist::render_playlist_browser(frame, app, inner);
        }
        Screen::PlaylistDetail => {
            components::playlist::render_playlist_detail(frame, app, layout.results);
            let queue_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" Queue ({}) ", app.queue.len()));
            let queue_inner = queue_block.inner(layout.queue);
            frame.render_widget(queue_block, layout.queue);
            components::render_queue(frame, app, queue_inner, false);
        }
            Screen::Queue => {
                render_queue_layout(frame, app, &layout);
            }
            Screen::Search => {
                render_search_layout(frame, app, &layout);
            }
        }
    }

    components::render_now_playing_bar(frame, app, layout.player);
    components::render_visualizer(frame, app, layout.visualizer);
    components::render_status_bar(frame, app, layout.status);
}

fn render_queue_layout(frame: &mut ratatui::Frame, app: &App, layout: &chunks::LayoutAreas) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" Queue ({}) ", app.queue.len()));
    let inner = block.inner(layout.content);
    frame.render_widget(block, layout.content);
    components::render_queue(frame, app, inner, true);
}

fn render_search_layout(frame: &mut ratatui::Frame, app: &App, layout: &chunks::LayoutAreas) {
    let search_layout = search::compute_search_layout(layout.results);

    // Scope badge so you always know what the list shows.
    let scope_badge = match &app.search_state {
        crate::app::SearchState::Loaded(r) => match r {
            crate::search::ScopedResults::Artists(_) => " [artist] ",
            crate::search::ScopedResults::Albums(_) => " [album] ",
            crate::search::ScopedResults::Tracks(_) => match app.search_scope {
                crate::search::SearchScope::Artists => " [artist→songs] ",
                crate::search::SearchScope::Albums => " [album→songs] ",
                _ => " [songs] ",
            },
        },
        crate::app::SearchState::Searching => match app.search_scope {
            crate::search::SearchScope::Artists => " [artist] ",
            crate::search::SearchScope::Albums => " [album] ",
            _ => " [songs] ",
        },
        crate::app::SearchState::LoadingDetail(_) => " [loading] ",
        _ => "",
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(if app.input_text.is_empty() {
            Color::DarkGray
        } else {
            Color::White
        }))
        .title(format!(" Search{} ", scope_badge));

    frame.render_widget(input_block, search_layout.input);

    let input_inner = Rect {
        x: search_layout.input.x + 1,
        y: search_layout.input.y + 1,
        width: search_layout.input.width.saturating_sub(2),
        height: search_layout.input.height.saturating_sub(2),
    };

    // Last query stays visible; show live prefix hint underneath the scope.
    let query_line = if app.input_text.is_empty() {
        " Type / to search — artist:  album:  movie:  song:"
    } else {
        ""
    };
    if !query_line.is_empty() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(
                ratatui::text::Span::styled(query_line, Style::default().fg(Color::DarkGray)),
            )),
            input_inner,
        );
    } else {
        let shown = if app.search_query.is_empty() {
            app.input_text.clone()
        } else {
            format!(
                "{}:{}",
                app.search_scope.short_label(),
                app.search_query
            )
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(
                ratatui::text::Span::styled(
                    format!(" {}", shown),
                    Style::default().fg(Color::White),
                ),
            )),
            input_inner,
        );
    }

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let results_inner = results_block.inner(search_layout.results);
    frame.render_widget(results_block, search_layout.results);

    match &app.search_state {
        crate::app::SearchState::Idle => {
            let p = ratatui::widgets::Paragraph::new(vec![
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    " No results yet — press / and try:",
                    Style::default().fg(Color::DarkGray),
                )),
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    "   artist:   album:   movie:   song:",
                    Style::default().fg(Color::Gray),
                )),
            ]);
            frame.render_widget(p, results_inner);
        }
        crate::app::SearchState::Searching => {
            let p = ratatui::widgets::Paragraph::new(format!(
                " Searching {} for \"{}\"...",
                app.search_scope.short_label(),
                app.search_query
            ))
            .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(p, results_inner);
        }
        crate::app::SearchState::LoadingDetail(label) => {
            let p = ratatui::widgets::Paragraph::new(format!(" Loading {}...", label))
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(p, results_inner);
        }
        crate::app::SearchState::Error(e) => {
            let p = ratatui::widgets::Paragraph::new(e.as_str())
                .style(Style::default().fg(Color::Red));
            frame.render_widget(p, results_inner);
        }
        crate::app::SearchState::Loaded(results) if results.is_empty() => {
            let p = ratatui::widgets::Paragraph::new(vec![
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    format!(
                        " No {} found for \"{}\".",
                        app.search_scope.label(),
                        app.search_query
                    ),
                    Style::default().fg(Color::Yellow),
                )),
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    " Try another spelling or prefix (artist:/album:/movie:/song:).",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);
            frame.render_widget(p, results_inner);
        }
        crate::app::SearchState::Loaded(results) => {
            components::render_scoped_results(frame, results, app.selected_index, results_inner);
        }
    }

    let queue_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" Queue ({}) ", app.queue.len()));
    let queue_inner = queue_block.inner(layout.queue);
    frame.render_widget(queue_block, layout.queue);
    components::render_queue(frame, app, queue_inner, false);
}

pub(crate) fn fmt_dur(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

fn render_playlist_picker(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(124, 58, 237)))
        .title(" Add to Playlist ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.playlists.playlists.is_empty() {
        let p = ratatui::widgets::Paragraph::new(" No playlists. Press Esc to cancel.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, inner);
        return;
    }

    let mut lines = Vec::new();
    for (i, pl) in app.playlists.playlists.iter().enumerate() {
        let selected = i == app.playlist_selected;
        let style = if selected {
            Style::default().bg(Color::Rgb(124, 58, 237)).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };
        let marker = if selected { " ▸ " } else { "   " };
        lines.push(ratatui::text::Line::from(
            ratatui::text::Span::styled(
                format!("{}{}  ({} tracks)", marker, pl.name, pl.tracks.len()),
                style,
            ),
        ));
    }

    let height = inner.height as usize;
    while lines.len() < height {
        lines.push(ratatui::text::Line::from(""));
    }
    if lines.len() > height {
        lines.truncate(height);
    }

    frame.render_widget(ratatui::widgets::Paragraph::new(lines), inner);
}
