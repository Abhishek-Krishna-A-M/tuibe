use anyhow::Result;

use super::events::Track;

pub trait SearchProvider: Send {
    fn search(&self, query: &str) -> Result<Vec<Track>>;
}
