use anyhow::Result;

use super::events::{Album, Artist, ScopedResults, Track};
use super::provider::SearchProvider;
use super::query::SearchScope;
use super::ytmusic_helper::run_python;

pub struct YTMusicProvider;

impl SearchProvider for YTMusicProvider {
    fn search(&self, query: &str, scope: SearchScope) -> Result<ScopedResults> {
        let stdout = run_python(&["search", scope.api_filter(), query])?;
        match scope {
            SearchScope::Songs => {
                let tracks: Vec<Track> = serde_json::from_str(&stdout)?;
                Ok(ScopedResults::Tracks(tracks))
            }
            SearchScope::Artists => {
                let artists: Vec<Artist> = serde_json::from_str(&stdout)?;
                Ok(ScopedResults::Artists(artists))
            }
            SearchScope::Albums => {
                let albums: Vec<Album> = serde_json::from_str(&stdout)?;
                Ok(ScopedResults::Albums(albums))
            }
        }
    }
}
