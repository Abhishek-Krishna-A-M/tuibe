use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct SearchLayout {
    pub input: Rect,
    pub results: Rect,
}

pub fn compute_search_layout(area: Rect) -> SearchLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    SearchLayout {
        input: chunks[0],
        results: chunks[1],
    }
}
