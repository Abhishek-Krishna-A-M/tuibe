use std::sync::mpsc;
use std::thread;

use anyhow::Result;

use super::events::Track;
use super::provider::SearchProvider;
use super::ytmusic_helper::run_python;
use super::ytmusic_provider::YTMusicProvider;

use super::events::SearchResult;
use super::query::SearchScope;

pub fn spawn_search(query: String, scope: SearchScope, tx: mpsc::Sender<SearchResult>) {
    thread::spawn(move || {
        let result = match YTMusicProvider.search(&query, scope) {
            Ok(results) => SearchResult::Ready {
                results,
                scope,
                query,
            },
            Err(e) => SearchResult::Error(e.to_string()),
        };
        let _ = tx.send(result);
    });
}

/// Detail fetch: expanding one artist/album row into playable tracks.
#[derive(Debug, Clone)]
pub enum DetailRequest {
    Artist { id: String, name: String },
    Album { title: String, id: String },
}

pub fn fetch_artist_tracks(browse_id: &str) -> Result<Vec<Track>> {
    let stdout = run_python(&["artist", browse_id])?;
    let tracks: Vec<Track> = serde_json::from_str(&stdout)?;
    Ok(tracks)
}

pub fn fetch_album_tracks(browse_id: &str) -> Result<Vec<Track>> {
    let stdout = run_python(&["album", browse_id])?;
    let tracks: Vec<Track> = serde_json::from_str(&stdout)?;
    Ok(tracks)
}

pub fn spawn_detail(req: DetailRequest, tx: mpsc::Sender<Result<Vec<Track>>>) {
    thread::spawn(move || {
        let result = match &req {
            DetailRequest::Artist { id, .. } => fetch_artist_tracks(id),
            DetailRequest::Album { id, .. } => fetch_album_tracks(id),
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
