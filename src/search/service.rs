use std::sync::mpsc;
use std::thread;

use anyhow::Result;

use super::events::Track;
use super::provider::SearchProvider;
use super::ytmusic_helper::run_python;
use super::ytmusic_provider::YTMusicProvider;

use super::events::SearchResult;

pub fn spawn_search(query: String, tx: mpsc::Sender<SearchResult>) {
    thread::spawn(move || {
        let result = match YTMusicProvider.search(&query) {
            Ok(tracks) => SearchResult::Ready(tracks),
            Err(e) => SearchResult::Error(e.to_string()),
        };
        let _ = tx.send(result);
    });
}

pub fn fetch_up_next(video_id: &str) -> Result<Vec<Track>> {
    let stdout = run_python(&["related", video_id])?;
    let tracks: Vec<Track> = serde_json::from_str(&stdout)?;
    Ok(tracks)
}

pub fn spawn_related(track: &Track, tx: mpsc::Sender<Vec<Track>>) {
    let video_id = track.id.clone();
    thread::spawn(move || {
        if let Ok(tracks) = fetch_up_next(&video_id) {
            if !tracks.is_empty() {
                let _ = tx.send(tracks);
            }
        }
    });
}
