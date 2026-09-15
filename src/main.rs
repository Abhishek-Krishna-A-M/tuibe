#![allow(dead_code)]

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use crossterm::{
    execute,
    event::{EnableMouseCapture, DisableMouseCapture},
};
use ratatui::prelude::{CrosstermBackend, Terminal};

mod app;
mod cache;
mod cava;
mod config;
mod input;
mod player;
mod playlist;
mod search;
mod stream;
mod theme;
mod ui;
mod volume;

use app::App;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetTitle("tuibe"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = config::Config::load();
    let playlists = playlist::PlaylistManager::load();
    let cache = cache::AudioCache::new(500);
    let (player_cmd, player_evt, player_handle) = player::spawn_player(cache);

    let size = terminal.size()?;
    let bar_count = (size.width as u32).clamp(20, 200);
    let (cava, cava_rx) = cava::CavaProcess::start(bar_count, 50);

    let mut app = App::new(config, playlists, cava_rx, player_cmd, player_evt);
    app.set_terminal_size(size.width, size.height);
    app.cava_process = Some(cava);
    app.cava_bar_count = bar_count;

    // Read PipeWire volume at startup
    app.volume = volume::PipeWireVolume::read();
    let _ = app.player_cmd.send(player::PlayerCommand::SetVolume(app.volume));
    volume::PipeWireVolume::set(app.volume);

    // Auto-sync imported playlists in background
    app.sync_all();

    let mut last_vol_poll = std::time::Instant::now();

    let result = run_app(&mut terminal, &mut app, &mut last_vol_poll);

    let _ = app.player_quit();
    if let Some(cava) = app.cava_process.take() {
        cava.join();
    }
    let _ = player_handle.join();

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    last_vol_poll: &mut std::time::Instant,
) -> Result<()> {
    loop {
        let size = terminal.size()?;
        app.set_terminal_size(size.width, size.height);

        // Poll PipeWire volume every ~1s for external changes
        if last_vol_poll.elapsed() >= Duration::from_secs(1) {
            let pw_vol = volume::PipeWireVolume::read();
            if (pw_vol - app.volume).abs() > 0.01 {
                app.volume = pw_vol;
                let _ = app.player_cmd.send(player::PlayerCommand::SetVolume(pw_vol));
            }
            *last_vol_poll = std::time::Instant::now();
        }

        // Drain all pending events (non-blocking after first poll)
        if event::poll(Duration::from_millis(50))? {
            loop {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(action) = input::map_key(key, app) {
                            if action == input::Action::Quit {
                                return Ok(());
                            }
                            app.dispatch(action);
                        }
                    }
                    Event::Mouse(mouse) => {
                        if let Some(action) = input::map_mouse(mouse, app) {
                            app.dispatch(action);
                        }
                    }
                    _ => {}
                }
                if !event::poll(Duration::from_secs(0))? {
                    break;
                }
            }
        }

        app.drain_player_events();
        app.drain_search_results();
        app.drain_detail();
        app.drain_related();
        app.drain_import();
        app.drain_sync();
        app.update_visualizer();
        terminal.draw(|frame| ui::render(frame, app))?;
    }
}
