use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Track {
    pub id: String,
    #[serde(alias = "track")]
    pub title: String,
    #[serde(alias = "uploader", alias = "channel", default = "default_artist")]
    pub artist: String,
    #[serde(default)]
    pub duration: f64,
    #[serde(alias = "webpage_url")]
    pub url: String,
    pub thumbnail: Option<String>,
}

fn default_artist() -> String {
    "Unknown".to_string()
}

impl Track {
    pub fn duration_string(&self) -> String {
        let secs = self.duration as u64;
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}:{:02}", mins, secs)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub subscribers: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Album {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopedResults {
    Tracks(Vec<Track>),
    Artists(Vec<Artist>),
    Albums(Vec<Album>),
}

impl ScopedResults {
    pub fn len(&self) -> usize {
        match self {
            ScopedResults::Tracks(t) => t.len(),
            ScopedResults::Artists(a) => a.len(),
            ScopedResults::Albums(a) => a.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub enum SearchResult {
    Ready {
        results: ScopedResults,
        scope: super::query::SearchScope,
        query: String,
    },
    Error(String),
}
