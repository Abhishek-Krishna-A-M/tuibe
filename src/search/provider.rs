use anyhow::Result;

use super::events::ScopedResults;
use super::query::SearchScope;

pub trait SearchProvider: Send {
    fn search(&self, query: &str, scope: SearchScope) -> Result<ScopedResults>;
}
