use anyhow::Result;

use super::events::Track;
use super::provider::SearchProvider;
use super::ytmusic_helper::run_python;

pub struct YTMusicProvider;

impl SearchProvider for YTMusicProvider {
    fn search(&self, query: &str) -> Result<Vec<Track>> {
        let stdout = run_python(&["search", query])?;
        let tracks: Vec<Track> = serde_json::from_str(&stdout)?;
        Ok(tracks)
    }
}
