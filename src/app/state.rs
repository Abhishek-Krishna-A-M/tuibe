use std::sync::mpsc;
use std::time::Duration;

use crate::cava::CavaProcess;
use crate::config::Config;
use crate::player::{PlayerCommand, PlayerEvent};
use crate::playlist::PlaylistManager;
use crate::search::{ScopedResults, SearchResult, SearchScope, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Idle,
    Searching,
    /// Loading tracks for one artist/album row.
    LoadingDetail(String),
    Loaded(ScopedResults),
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
    ImportPlaylist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    Queue,
    One,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::Queue,
            RepeatMode::Queue => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            RepeatMode::Off => "",
            RepeatMode::Queue => " \u{f0b6}",
            RepeatMode::One => " \u{f0b6} 1",
        }
    }
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
    pub search_scope: SearchScope,
    pub search_query: String,
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
    pub repeat_mode: RepeatMode,
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
    pub(crate) detail_rx: Option<mpsc::Receiver<anyhow::Result<Vec<Track>>>>,
    pub(crate) pending_detail_play: bool,
    pub(crate) related_rx: Option<mpsc::Receiver<Vec<Track>>>,
    pub(crate) pending_add_track: Option<Track>,
    pub(crate) import_rx: Option<mpsc::Receiver<(String, String, Vec<Track>)>>,
    pub(crate) sync_rx: Option<mpsc::Receiver<(String, String, Vec<Track>)>>,
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
            search_scope: SearchScope::Songs,
            search_query: String::new(),
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
            repeat_mode: RepeatMode::Off,
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
            detail_rx: None,
            pending_detail_play: false,
            related_rx: None,
            pending_add_track: None,
            import_rx: None,
            sync_rx: None,
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
            SearchState::Loaded(r) => r.len(),
            _ => 0,
        }
    }

    /// Clamp `cursor` to a valid char boundary (unicode-safe).
    pub fn clamp_cursor(&mut self) {
        let len = self.input_text.len();
        if self.cursor > len {
            self.cursor = len;
        }
        while !self.input_text.is_char_boundary(self.cursor) && self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor one char left (unicode-safe).
    pub fn cursor_left(&mut self) {
        self.clamp_cursor();
        if self.cursor == 0 {
            return;
        }
        let mut next = self.cursor - 1;
        while next > 0 && !self.input_text.is_char_boundary(next) {
            next -= 1;
        }
        self.cursor = next;
    }

    /// Move cursor one char right (unicode-safe).
    pub fn cursor_right(&mut self) {
        self.clamp_cursor();
        if self.cursor >= self.input_text.len() {
            return;
        }
        let mut next = self.cursor + 1;
        while next < self.input_text.len() && !self.input_text.is_char_boundary(next) {
            next += 1;
        }
        self.cursor = next.min(self.input_text.len());
    }

    /// Insert a char at the cursor (unicode-safe).
    pub fn insert_char(&mut self, c: char) {
        self.clamp_cursor();
        self.input_text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the char before the cursor (unicode-safe backspace).
    pub fn backspace(&mut self) {
        self.clamp_cursor();
        if self.cursor == 0 {
            return;
        }
        let mut start = self.cursor - 1;
        while start > 0 && !self.input_text.is_char_boundary(start) {
            start -= 1;
        }
        self.input_text.drain(start..self.cursor);
        self.cursor = start;
    }
}
