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

pub enum SearchResult {
    Ready(Vec<Track>),
    Error(String),
}
