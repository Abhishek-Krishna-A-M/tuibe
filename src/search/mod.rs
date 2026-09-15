mod events;
mod provider;
pub mod query;
mod service;
pub(crate) mod ytmusic_helper;
mod ytmusic_provider;

pub use events::{Album, Artist, ScopedResults, SearchResult, Track};
pub use query::{parse_query, SearchScope};
pub use service::{spawn_detail, spawn_search, spawn_related, DetailRequest};
