use std::time::Duration;

use crate::input::Action;
use crate::player::{PlayerCommand, PlayerEvent};
use crate::search::{self, SearchResult};
use crate::search::Track;

use super::state::{App, InputMode, PlaybackState, RepeatMode, Screen, SearchState, VisualizerMode};

impl App {
    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::EnqueueSelected => match self.screen {
                Screen::PlaylistDetail => {
                    if let Some(pl_id) = &self.playlist_detail_id
                        && let Some(pl) = self.playlists.playlists.iter().find(|p| p.id == *pl_id)
                        && let Some(track) = pl.tracks.get(self.selected_index)
                        && !self.queue.iter().any(|t| t.id == track.id)
                    {
                        self.queue.insert(self.queue_index + 1, track.clone());
                    }
                }
                _ => {
                    if let SearchState::Loaded(tracks) = &self.search_state
                        && let Some(track) = tracks.get(self.selected_index)
                        && !self.queue.iter().any(|t| t.id == track.id)
                    {
                        self.queue.insert(self.queue_index + 1, track.clone());
                    }
                }
            },
            Action::Search => {
                if self.screen == Screen::Search {
                    self.input_mode = Some(InputMode::Search);
                    self.cursor = self.input_text.len();
                }
            }
            Action::SearchSubmit => {
                let query = self.input_text.trim().to_string();
                if query.is_empty() {
                    return;
                }
                self.input_mode = None;
                self.selected_index = 0;
                self.search_state = SearchState::Searching;
                let (tx, rx) = std::sync::mpsc::channel();
                search::spawn_search(query, tx);
                self.search_rx = Some(rx);
            }
            Action::ShowPlaylists => {
                self.screen = Screen::PlaylistBrowser;
                self.playlist_selected = 0;
            }
            Action::NewPlaylist => {
                self.input_mode = Some(InputMode::NewPlaylist);
                self.input_text.clear();
            }
            Action::DeletePlaylist => {
                if self.playlist_selected < self.playlists.playlists.len() {
                    let id = self.playlists.playlists[self.playlist_selected].id.clone();
                    self.playlists.delete(&id);
                    self.playlist_selected = self.playlist_selected.min(
                        self.playlists.playlists.len().saturating_sub(1),
                    );
                }
            }
            Action::RemoveFromPlaylist => {
                if let Some(pl_id) = &self.playlist_detail_id {
                    if let Some(pl) = self.playlists.playlists.iter().find(|p| p.id == *pl_id) {
                        if self.selected_index < pl.tracks.len() {
                            let track_id = pl.tracks[self.selected_index].id.clone();
                            self.playlists.remove_track(pl_id, &track_id);
                        }
                    }
                }
            }
            Action::EnterPlaylist => {
                if self.playlist_selected < self.playlists.playlists.len() {
                    let pl = &self.playlists.playlists[self.playlist_selected];
                    if !pl.tracks.is_empty() {
                        self.playlist_detail_id = Some(pl.id.clone());
                        self.screen = Screen::PlaylistDetail;
                        self.selected_index = 0;
                    }
                }
            }
            Action::Back => {
                self.screen = Screen::Search;
                self.playlist_detail_id = None;
            }

            Action::PlaySelected => match self.screen {
                Screen::Queue => {
                    if let Some(track) = self.queue.get(self.queue_selected).cloned() {
                        self.queue_index = self.queue_selected;
                        self.play_track(track);
                    }
                }
                Screen::PlaylistDetail => {
                    if let Some(pl_id) = &self.playlist_detail_id {
                        if let Some(pl) = self.playlists.playlists.iter().find(|p| p.id == *pl_id) {
                            if let Some(track) = pl.tracks.get(self.selected_index) {
                                self.playlist_detail_id = Some(pl.id.clone());
                                let idx = self.selected_index;
                                let rotated: Vec<Track> = pl.tracks[idx..]
                                    .iter()
                                    .chain(pl.tracks[..idx].iter())
                                    .cloned()
                                    .collect();
                                self.queue = rotated;
                                self.queue_index = 0;
                                self.queue_selected = 0;
                                self.play_track(track.clone());
                            }
                        }
                    }
                }
                _ => {
                    if let SearchState::Loaded(tracks) = &self.search_state {
                        if let Some(track) = tracks.get(self.selected_index) {
                            self.queue.clear();
                            self.queue_index = 0;
                            self.queue_selected = 0;
                            self.play_track(track.clone());
                        }
                    }
                }
            },

            Action::PauseResume => match self.playback_state {
                PlaybackState::Playing => {
                    self.playback_state = PlaybackState::Paused;
                    let _ = self.player_cmd.send(PlayerCommand::Pause);
                }
                PlaybackState::Paused => {
                    self.playback_state = PlaybackState::Playing;
                    let _ = self.player_cmd.send(PlayerCommand::Resume);
                }
                PlaybackState::Stopped => {
                    if let Some(track) = &self.current_track {
                        self.play_track(track.clone());
                    }
                }
            },

            Action::Stop => {
                self.playback_state = PlaybackState::Stopped;
                self.position = Duration::ZERO;
                let _ = self.player_cmd.send(PlayerCommand::Stop);
            }

            Action::Next => self.next_track(),
            Action::Previous => self.prev_track(),

            Action::SeekForward => {
                let pos = (self.position + Duration::from_secs(10)).min(self.duration);
                self.seek_to(pos);
            }
            Action::SeekBackward => {
                let pos = self.position.saturating_sub(Duration::from_secs(10));
                self.seek_to(pos);
            }

            Action::VolumeUp => {
                self.volume = (self.volume + 0.05).min(1.0);
                let _ = self.player_cmd.send(PlayerCommand::SetVolume(self.volume));
                crate::volume::PipeWireVolume::set(self.volume);
            }
            Action::VolumeDown => {
                self.volume = (self.volume - 0.05).max(0.0);
                let _ = self.player_cmd.send(PlayerCommand::SetVolume(self.volume));
                crate::volume::PipeWireVolume::set(self.volume);
            }

            Action::ToggleLike => {
                if let Some(track) = &self.current_track {
                    if self.playlists.is_liked(&track.id) {
                        self.playlists.remove_track("liked", &track.id);
                    } else {
                        self.playlists.add_track("liked", track.clone());
                    }
                } else if let SearchState::Loaded(tracks) = &self.search_state {
                    if let Some(track) = tracks.get(self.selected_index) {
                        if self.playlists.is_liked(&track.id) {
                            self.playlists.remove_track("liked", &track.id);
                        } else {
                            self.playlists.add_track("liked", track.clone());
                        }
                    }
                }
            }

            Action::MoveUp => match self.input_mode {
                Some(InputMode::AddToPlaylist) => {
                    self.playlist_selected = self.playlist_selected.saturating_sub(1);
                }
                _ => match self.screen {
                    Screen::Queue => {
                        self.queue_selected = self.queue_selected.saturating_sub(1);
                    }
                    Screen::PlaylistBrowser => {
                        self.playlist_selected = self.playlist_selected.saturating_sub(1);
                    }
                    _ => {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }
                },
            },
            Action::MoveDown => match self.input_mode {
                Some(InputMode::AddToPlaylist) => {
                    let max = self.playlists.playlists.len().saturating_sub(1);
                    self.playlist_selected = self.playlist_selected.min(max).saturating_add(1).min(max);
                }
                _ => match self.screen {
                Screen::Queue => {
                    let max = self.queue.len().saturating_sub(1);
                    self.queue_selected = self.queue_selected.min(max).saturating_add(1).min(max);
                }
                Screen::PlaylistBrowser => {
                    let max = self.playlists.playlists.len().saturating_sub(1);
                    self.playlist_selected = self.playlist_selected.min(max).saturating_add(1).min(max);
                }
                Screen::PlaylistDetail => {
                    if let Some(pl_id) = &self.playlist_detail_id {
                        if let Some(pl) = self.playlists.playlists.iter().find(|p| p.id == *pl_id) {
                            let max = pl.tracks.len().saturating_sub(1);
                            self.selected_index = self.selected_index.min(max).saturating_add(1).min(max);
                        }
                    }
                }
                _ => {
                    let max = self.current_results_len().saturating_sub(1);
                    self.selected_index = self.selected_index.min(max).saturating_add(1).min(max);
                }
            },
            },

            Action::TypeChar(c) => {
                self.input_text.insert(self.cursor, c);
                self.cursor += 1;
            }
            Action::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input_text.remove(self.cursor);
                }
            }
            Action::CursorLeft => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            Action::CursorRight => {
                self.cursor = (self.cursor + 1).min(self.input_text.len());
            }
            Action::ConfirmInput => match self.input_mode {
                Some(InputMode::AddToPlaylist) => {
                    if let Some(track) = self.pending_add_track.take() {
                        if self.playlist_selected < self.playlists.playlists.len() {
                            let pl_id = self.playlists.playlists[self.playlist_selected].id.clone();
                            self.playlists.add_track(&pl_id, track);
                        }
                    }
                    self.input_mode = None;
                }
                Some(InputMode::NewPlaylist) => {
                    let name = self.input_text.trim().to_string();
                    if !name.is_empty() {
                        self.playlists.create(&name);
                    }
                    self.input_mode = None;
                    self.input_text.clear();
                }
                Some(InputMode::ImportPlaylist) => {
                    let input = self.input_text.trim().to_string();
                    if !input.is_empty() {
                        self.input_mode = None;
                        self.input_text.clear();
                        self.cursor = 0;
                        let (tx, rx) = std::sync::mpsc::channel();
                        std::thread::spawn(move || {
                            let result = crate::search::ytmusic_helper::fetch_playlist(&input);
                            let _ = tx.send(result);
                        });
                        self.import_rx = Some(rx);
                    }
                }
                Some(InputMode::Search) => {
                    let query = self.input_text.trim().to_string();
                    if !query.is_empty() {
                        self.input_mode = None;
                        self.cursor = 0;
                        self.selected_index = 0;
                        self.search_state = SearchState::Searching;
                        let (tx, rx) = std::sync::mpsc::channel();
                        search::spawn_search(query, tx);
                        self.search_rx = Some(rx);
                    }
                }
                None => {}
            },
            Action::Cancel => {
                self.input_mode = None;
                self.input_text.clear();
                self.cursor = 0;
                self.pending_add_track = None;
            }

            Action::ToggleRepeat => {
                self.repeat_mode = self.repeat_mode.next();
            }
            Action::ToggleShuffle => {
                self.shuffle = !self.shuffle;
            }
            Action::ToggleQueueView => {
                self.screen = match self.screen {
                    Screen::Queue => {
                        self.queue_selected = 0;
                        Screen::Search
                    }
                    _ => {
                        self.queue_selected = self.queue_index;
                        Screen::Queue
                    }
                };
            }
            Action::RemoveFromQueue => {
                if self.queue.is_empty() {
                    return;
                }
                let idx = self.queue_selected.min(self.queue.len().saturating_sub(1));
                self.queue.remove(idx);
                if self.queue.is_empty() {
                    self.queue_selected = 0;
                    self.queue_index = 0;
                    self.playback_state = PlaybackState::Stopped;
                    let _ = self.player_cmd.send(PlayerCommand::Stop);
                    return;
                }
                if idx < self.queue_index {
                    self.queue_index = self.queue_index.saturating_sub(1);
                } else if idx == self.queue_index {
                    self.queue_index = self.queue_index.min(self.queue.len().saturating_sub(1));
                    if let Some(track) = self.queue.get(self.queue_index).cloned() {
                        self.play_track(track);
                    }
                }
                self.queue_selected = self.queue_selected.min(self.queue.len().saturating_sub(1));
            }
            Action::AddToPlaylist => {
                let track = match self.screen {
                    Screen::PlaylistDetail => {
                        if let Some(pl_id) = &self.playlist_detail_id
                            && let Some(pl) = self.playlists.playlists.iter().find(|p| p.id == *pl_id)
                        {
                            pl.tracks.get(self.selected_index).cloned()
                        } else {
                            None
                        }
                    }
                    _ => {
                        if let SearchState::Loaded(tracks) = &self.search_state {
                            tracks.get(self.selected_index).cloned()
                        } else {
                            None
                        }
                    }
                };
                if let Some(track) = track {
                    self.pending_add_track = Some(track);
                    self.input_mode = Some(InputMode::AddToPlaylist);
                    self.playlist_selected = 0;
                }
            }
            Action::ImportPlaylist => {
                self.input_mode = Some(InputMode::ImportPlaylist);
                self.input_text.clear();
                self.cursor = 0;
            }
            Action::ToggleAutoplay => {
                self.autoplay = !self.autoplay;
                if !self.autoplay {
                    self.related_rx = None;
                }
            }
            Action::ToggleVisualizer => {
                let was_none = self.vis_mode == VisualizerMode::None;
                let now_none = !was_none;
                self.vis_mode = self.vis_mode.next();
                if now_none {
                    if let Some(proc) = &mut self.cava_process {
                        proc.stop();
                    }
                    self.vis_bars = vec![0.0; self.vis_bars.len().max(1)];
                } else {
                    self.restart_cava();
                }
            }
            Action::CavaSensitivityUp => {
                self.cava_sensitivity = (self.cava_sensitivity + 10).min(500);
                self.restart_cava();
            }
            Action::CavaSensitivityDown => {
                self.cava_sensitivity = self.cava_sensitivity.saturating_sub(10).max(10);
                self.restart_cava();
            }
            Action::CavaBarsUp => {
                self.cava_bar_count = (self.cava_bar_count + 5).min(200);
                self.restart_cava();
            }
            Action::CavaBarsDown => {
                self.cava_bar_count = self.cava_bar_count.saturating_sub(5).max(10);
                self.restart_cava();
            }
            Action::Tick => {}
            Action::Quit => {}
        }
    }

    pub fn play_track(&mut self, track: Track) {
        self.generation += 1;
        self.current_track = Some(track.clone());
        self.playback_state = PlaybackState::Playing;
        self.position = Duration::ZERO;

        if !self.queue.iter().any(|t| t.id == track.id) {
            self.queue.push(track.clone());
            self.queue_index = self.queue.len() - 1;
        } else {
            self.queue_index = self.queue.iter().position(|t| t.id == track.id).unwrap();
        }

        // Auto-fill related tracks (search only — not from playlist)
        let from_playlist = self.screen == Screen::PlaylistDetail;
        if self.autoplay && !from_playlist && self.related_rx.is_none() {
            let remaining = self.queue.len().saturating_sub(self.queue_index + 1);
            if remaining <= 2 {
                let (tx, rx) = std::sync::mpsc::channel();
                crate::search::spawn_related(&track, tx);
                self.related_rx = Some(rx);
            }
        }

        if self.screen == Screen::Queue {
            self.queue_selected = self.queue_index;
        }

        let _ = self.player_cmd.send(PlayerCommand::Play {
            url: track.url.clone(),
            track_id: track.id.clone(),
            generation: self.generation,
        });
    }

    pub fn next_track(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        if self.shuffle {
            self.queue_index = rand::random_range(0..self.queue.len());
        } else {
            self.queue_index = (self.queue_index + 1) % self.queue.len();
        }
        if let Some(track) = self.queue.get(self.queue_index).cloned() {
            self.play_track(track);
        }
    }

    pub fn prev_track(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        self.queue_index = if self.queue_index == 0 {
            self.queue.len() - 1
        } else {
            self.queue_index - 1
        };
        if let Some(track) = self.queue.get(self.queue_index).cloned() {
            self.play_track(track);
        }
    }

    pub fn seek_to(&mut self, pos: Duration) {
        self.position = pos;
        let _ = self.player_cmd.send(PlayerCommand::Seek(pos));
    }

    pub fn drain_player_events(&mut self) {
        while let Ok(evt) = self.player_evt.try_recv() {
            match evt {
                PlayerEvent::Started { duration } => {
                    self.duration = duration;
                }
                PlayerEvent::Progress { position, duration } => {
                    self.position = position;
                    if duration > Duration::ZERO {
                        self.duration = duration;
                    }
                }
                PlayerEvent::Finished => {
                    match self.repeat_mode {
                        RepeatMode::One => {
                            if let Some(track) = &self.current_track {
                                self.generation += 1;
                                self.position = Duration::ZERO;
                                self.playback_state = PlaybackState::Playing;
                                let _ = self.player_cmd.send(PlayerCommand::Play {
                                    url: track.url.clone(),
                                    track_id: track.id.clone(),
                                    generation: self.generation,
                                });
                            }
                        }
                        RepeatMode::Queue => {
                            self.next_track();
                        }
                        RepeatMode::Off => {
                            let next = self.queue_index + 1;
                            if next < self.queue.len() {
                                self.queue_index = next;
                                if let Some(track) = self.queue.get(next).cloned() {
                                    self.play_track(track);
                                }
                            } else if let Some(ref track) = self.current_track.clone() {
                                // End of queue — fetch related tracks
                                self.playback_state = PlaybackState::Stopped;
                                if self.related_rx.is_none() {
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    crate::search::spawn_related(track, tx);
                                    self.related_rx = Some(rx);
                                }
                            } else {
                                self.playback_state = PlaybackState::Stopped;
                            }
                        }
                    }
                }
                PlayerEvent::Error(_) => {
                    self.playback_state = PlaybackState::Stopped;
                }
            }
        }
    }

    pub fn drain_search_results(&mut self) {
        if let Some(ref rx) = self.search_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    SearchResult::Ready(tracks) => {
                        self.search_state = SearchState::Loaded(tracks);
                    }
                    SearchResult::Error(e) => {
                        self.search_state = SearchState::Error(e);
                    }
                }
                self.search_rx = None;
            }
        }
    }

    pub fn drain_related(&mut self) {
        if let Some(ref rx) = self.related_rx {
            if let Ok(tracks) = rx.try_recv() {
                let was_stopped = self.playback_state == PlaybackState::Stopped;
                for t in &tracks {
                    if !self.queue.iter().any(|qt| qt.id == t.id) {
                        self.queue.push(t.clone());
                    }
                }
                self.related_rx = None;
                // Auto-play first new track if queue had ended
                if was_stopped {
                    if let Some(t) = tracks.into_iter().find(|t| {
                        self.current_track.as_ref().map_or(true, |c| t.id != c.id)
                    }) {
                        self.queue_index = self.queue.iter()
                            .position(|qt| qt.id == t.id)
                            .unwrap_or(self.queue.len().saturating_sub(1));
                        self.play_track(t);
                    }
                }
            }
        }
    }

    pub fn drain_import(&mut self) {
        if let Some(ref rx) = self.import_rx {
            if let Ok((name, tracks)) = rx.try_recv() {
                if !tracks.is_empty() {
                    let id = self.playlists.create(&name);
                    for t in tracks {
                        self.playlists.add_track(&id, t);
                    }
                }
                self.import_rx = None;
            }
        }
    }

    pub fn update_visualizer(&mut self) {
        if self.vis_mode != VisualizerMode::Spectrum {
            return;
        }
        while let Ok(bars) = self.cava_bars_rx.try_recv() {
            self.vis_bars.clear();
            self.vis_bars.extend(bars.iter().map(|v| v.clamp(0.0, 1.0)));
            self.vis_peaks.resize(self.vis_bars.len(), 0.0);
        }
        for p in &mut self.vis_peaks {
            *p *= 0.97;
        }
        for (p, b) in self.vis_peaks.iter_mut().zip(&self.vis_bars) {
            if *b > *p {
                *p = *b;
            }
        }
    }

    fn restart_cava(&mut self) {
        if let Some(proc) = &mut self.cava_process {
            proc.restart(self.cava_bar_count, self.cava_sensitivity);
        }
    }
}
