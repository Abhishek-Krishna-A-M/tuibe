use std::io::{Read, Seek};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rodio::{Decoder, DeviceSinkBuilder, Player, Source};

use crate::cache::{AudioCache, CachedFile};
use crate::stream::{self, StreamHandle};

use super::commands::PlayerCommand;
use super::events::PlayerEvent;

trait ReadSeek: Read + Seek + Send + Sync + 'static {}
impl ReadSeek for CachedFile {}
impl ReadSeek for StreamHandle {}

pub fn spawn_player(
    cache: AudioCache,
) -> (
    Sender<PlayerCommand>,
    Receiver<PlayerEvent>,
    JoinHandle<()>,
) {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (evt_tx, evt_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        run_player(cmd_rx, evt_tx, cache);
    });

    (cmd_tx, evt_rx, handle)
}

fn get_stream(
    url: &str,
    track_id: &str,
    cache: &mut AudioCache,
) -> Result<Box<dyn ReadSeek>, String> {
    if cache.is_cached(track_id) {
        cache.touch(track_id);
        if let Some(f) = cache.open_cached(track_id) {
            return Ok(Box::new(f));
        }
    }
    let cache_path = cache.cache_path_for(track_id);
    let (handle, _join) = stream::spawn_cached_stream(url, &cache_path);
    cache.touch(track_id);
    Ok(Box::new(handle))
}

fn drain_player(player: &Player, timeout: Duration) {
    let start = Instant::now();
    while !player.empty() && start.elapsed() < timeout {
        thread::sleep(Duration::from_millis(5));
    }
}

fn run_player(
    cmd_rx: Receiver<PlayerCommand>,
    evt_tx: Sender<PlayerEvent>,
    mut cache: AudioCache,
) {
    let sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(s) => s,
        Err(e) => {
            let _ = evt_tx.send(PlayerEvent::Error(format!("Audio device: {}", e)));
            return;
        }
    };
    let player = Player::connect_new(sink.mixer());

    let mut current_gen: u64 = 0;
    let mut playing = false;
    let mut track_dur = Duration::ZERO;
    let mut last_progress = Instant::now();
    let mut last_pos = Duration::ZERO;
    let mut stalled_cycles: u32 = 0;
    let mut empty_cycles: u32 = 0;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                PlayerCommand::Play {
                    url,
                    track_id,
                    generation,
                } => {
                    if generation < current_gen {
                        continue;
                    }
                    current_gen = generation;
                    playing = false;
                    track_dur = Duration::ZERO;
                    last_pos = Duration::ZERO;
                    stalled_cycles = 0;
                    empty_cycles = 0;

                    player.stop();
                    drain_player(&player, Duration::from_millis(200));

                    let decoder_result = match get_stream(&url, &track_id, &mut cache) {
                        Ok(stream) => Decoder::new(stream).map_err(|e| e.to_string()),
                        Err(e) => Err(e),
                    };

                    match decoder_result {
                        Ok(decoder) => {
                            track_dur = decoder.total_duration().unwrap_or(Duration::ZERO);
                            player.append(decoder);
                            playing = true;
                            last_progress = Instant::now();
                            let _ = evt_tx.send(PlayerEvent::Started {
                                duration: track_dur,
                            });
                        }
                        Err(e) => {
                            let _ = evt_tx
                                .send(PlayerEvent::Error(format!("Decoder error: {}", e)));
                        }
                    }
                }
                PlayerCommand::Stop => {
                    playing = false;
                    track_dur = Duration::ZERO;
                    player.stop();
                }
                PlayerCommand::Pause => {
                    playing = false;
                    player.pause();
                }
                PlayerCommand::Resume => {
                    playing = true;
                    player.play();
                    last_progress = Instant::now();
                    stalled_cycles = 0;
                }
                PlayerCommand::Seek(pos) => {
                    let _ = player.try_seek(pos);
                    last_progress = Instant::now();
                    last_pos = pos;
                    stalled_cycles = 0;
                    empty_cycles = 0;
                    let _ = evt_tx.send(PlayerEvent::Progress {
                        position: pos,
                        duration: track_dur,
                    });
                }
                PlayerCommand::SetVolume(vol) => {
                    player.set_volume(vol);
                }
                PlayerCommand::Quit => {
                    player.stop();
                    return;
                }
            }
        }

        if playing {
            let now = Instant::now();
            if now.duration_since(last_progress) >= Duration::from_millis(250) {
                last_progress = now;
                let pos = player.get_pos();

                let _ = evt_tx.send(PlayerEvent::Progress {
                    position: pos,
                    duration: track_dur,
                });

                let is_empty = player.empty();

                // Debounce empty: need 2 consecutive ticks (500ms) to confirm EOF
                if is_empty {
                    empty_cycles += 1;
                } else {
                    empty_cycles = 0;
                }

                // Detect finished: source exhausted and position at/near duration
                let near_end = track_dur > Duration::ZERO
                    && pos >= track_dur.saturating_sub(Duration::from_millis(500));

                // Detect stalled: position hasn't advanced for 1+ second
                if pos == last_pos {
                    stalled_cycles += 1;
                } else {
                    stalled_cycles = 0;
                }
                let stalled = stalled_cycles >= 4; // 1 second with no progress

                last_pos = pos;

                if empty_cycles >= 2 || near_end || stalled {
                    playing = false;
                    let _ = evt_tx.send(PlayerEvent::Finished);
                }
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}
