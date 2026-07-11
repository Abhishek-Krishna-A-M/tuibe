use std::sync::mpsc;
use std::time::Duration;

use crate::cava::CavaProcess;
use crate::config::Config;
use crate::player::{PlayerCommand, PlayerEvent};
use crate::playlist::PlaylistManager;
use crate::search::{SearchResult, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Idle,
    Searching,
    Loaded(Vec<Track>),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Search,
    Queue,
    PlaylistBrowser,
    PlaylistDetail,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Search,
    NewPlaylist,
    AddToPlaylist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualizerMode {
    Spectrum,
    None,
}

impl VisualizerMode {
    pub fn next(self) -> Self {
        match self {
            VisualizerMode::Spectrum => VisualizerMode::None,
            VisualizerMode::None => VisualizerMode::Spectrum,
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub input_mode: Option<InputMode>,
    pub input_text: String,
    pub cursor: usize,
    pub search_state: SearchState,
    pub selected_index: usize,
    pub playlist_selected: usize,
    pub playlist_detail_id: Option<String>,

    pub current_track: Option<Track>,
    pub playback_state: PlaybackState,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub queue: Vec<Track>,
    pub queue_index: usize,
    pub queue_selected: usize,
    pub shuffle: bool,
    pub repeat: bool,
    pub autoplay: bool,

    pub terminal_height: u16,
    pub terminal_width: u16,
    pub config: Config,
    pub playlists: PlaylistManager,
    pub cava_bars_rx: mpsc::Receiver<Vec<f32>>,
    pub cava_process: Option<CavaProcess>,
    pub cava_sensitivity: u32,
    pub cava_bar_count: u32,
    pub vis_mode: VisualizerMode,
    pub vis_bars: Vec<f32>,
    pub vis_peaks: Vec<f32>,

    pub player_cmd: mpsc::Sender<PlayerCommand>,
    pub player_evt: mpsc::Receiver<PlayerEvent>,
    pub(crate) generation: u64,
    pub(crate) search_rx: Option<mpsc::Receiver<SearchResult>>,
    pub(crate) related_rx: Option<mpsc::Receiver<Vec<Track>>>,
    pub(crate) pending_add_track: Option<Track>,
}

impl App {
    pub fn new(
        config: Config,
        playlists: PlaylistManager,
        cava_bars_rx: mpsc::Receiver<Vec<f32>>,
        player_cmd: mpsc::Sender<PlayerCommand>,
        player_evt: mpsc::Receiver<PlayerEvent>,
    ) -> Self {
        let app = Self {
            screen: Screen::Search,
            input_mode: None,
            input_text: String::new(),
            cursor: 0,
            search_state: SearchState::Idle,
            selected_index: 0,
            playlist_selected: 0,
            playlist_detail_id: None,
            current_track: None,
            playback_state: PlaybackState::Stopped,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 0.7,
            queue: Vec::new(),
            queue_index: 0,
            queue_selected: 0,
            shuffle: false,
            repeat: false,
            autoplay: true,
            terminal_height: 24,
            terminal_width: 80,
            config,
            playlists,
            cava_bars_rx,
            cava_process: None,
            cava_sensitivity: 50,
            cava_bar_count: 30,
            vis_mode: VisualizerMode::Spectrum,
            vis_bars: vec![0.0; 32],
            vis_peaks: vec![0.0; 32],
            player_cmd,
            player_evt,
            generation: 0,
            search_rx: None,
            related_rx: None,
            pending_add_track: None,
        };
        let _ = app.player_cmd.send(PlayerCommand::SetVolume(app.volume));
        app
    }

    pub fn input_mode(&self) -> Option<InputMode> {
        self.input_mode.clone()
    }

    pub fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.terminal_width = w;
        self.terminal_height = h;
    }

    pub fn player_quit(&self) -> Result<(), ()> {
        self.player_cmd.send(PlayerCommand::Quit).map_err(|_| ())
    }

    pub(crate) fn current_results_len(&self) -> usize {
        match &self.search_state {
            SearchState::Loaded(t) => t.len(),
            _ => 0,
        }
    }
}
