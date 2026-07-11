use std::time::Duration;

#[derive(Debug)]
pub enum PlayerCommand {
    Play { url: String, track_id: String, generation: u64 },
    Stop,
    Pause,
    Resume,
    Seek(Duration),
    SetVolume(f32),
    Quit,
}
