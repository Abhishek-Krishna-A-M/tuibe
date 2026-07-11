use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::VisualizerMode;

pub struct LayoutAreas {
    pub content: Rect,
    pub results: Rect,
    pub queue: Rect,
    pub player: Rect,
    pub visualizer: Rect,
    pub status: Rect,
}

pub fn compute_layout(area: Rect, vis_mode: VisualizerMode) -> LayoutAreas {
    let vis_height: u16 = match vis_mode {
        VisualizerMode::Spectrum => 8,
        VisualizerMode::None => 0,
    };

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(4),
            Constraint::Length(vis_height),
            Constraint::Length(1),
        ])
        .split(area);

    let content_area = main[0];
    let player = main[1];
    let visualizer_area = main[2];
    let status = main[3];

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content_area);

    LayoutAreas {
        content: content_area,
        results: columns[0],
        queue: columns[1],
        player,
        visualizer: visualizer_area,
        status,
    }
}
