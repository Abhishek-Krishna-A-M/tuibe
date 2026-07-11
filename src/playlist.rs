use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::search::Track;

pub type PlaylistId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub tracks: Vec<Track>,
}

impl Playlist {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tracks: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistManager {
    pub playlists: Vec<Playlist>,
    dirty: bool,
}

impl PlaylistManager {
    pub fn load() -> Self {
        let path = Self::playlists_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut pm) = serde_json::from_str::<PlaylistManager>(&content) {
                    pm.dirty = false;
                    return pm;
                }
            }
        }
        let mut pm = Self {
            playlists: vec![Playlist::new("liked", "Liked")],
            dirty: false,
        };
        pm.migrate_favorites();
        let _ = pm.save();
        pm
    }

    fn migrate_favorites(&mut self) {
        let old_path = favorites_path();
        if old_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&old_path) {
                #[derive(Deserialize)]
                struct OldFavs { tracks: Vec<Track> }
                if let Ok(old) = serde_json::from_str::<OldFavs>(&content) {
                    if let Some(liked) = self.playlists.iter_mut().find(|p| p.id == "liked") {
                        liked.tracks = old.tracks;
                    }
                    let _ = std::fs::remove_file(&old_path);
                }
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = Self::playlists_dir();
        std::fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::playlists_path(), content)?;
        Ok(())
    }

    pub fn create(&mut self, name: &str) -> PlaylistId {
        let id = format!("pl_{}", name.to_lowercase().replace(' ', "_"));
        let pl = Playlist::new(id.clone(), name);
        self.playlists.push(pl);
        let _ = self.save();
        id
    }

    pub fn delete(&mut self, id: &str) {
        if id == "liked" { return; }
        self.playlists.retain(|p| p.id != id);
        let _ = self.save();
    }

    pub fn add_track(&mut self, playlist_id: &str, track: Track) {
        if let Some(pl) = self.playlists.iter_mut().find(|p| p.id == playlist_id) {
            if !pl.tracks.iter().any(|t| t.id == track.id) {
                pl.tracks.push(track);
                let _ = self.save();
            }
        }
    }

    pub fn remove_track(&mut self, playlist_id: &str, track_id: &str) {
        if let Some(pl) = self.playlists.iter_mut().find(|p| p.id == playlist_id) {
            pl.tracks.retain(|t| t.id != track_id);
            let _ = self.save();
        }
    }

    pub fn is_liked(&self, track_id: &str) -> bool {
        self.playlists
            .iter()
            .find(|p| p.id == "liked")
            .map(|p| p.tracks.iter().any(|t| t.id == track_id))
            .unwrap_or(false)
    }

    pub fn get(&self, id: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.id == id)
    }

    fn playlists_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tuibe")
    }

    fn playlists_path() -> PathBuf {
        Self::playlists_dir().join("playlists.json")
    }
}

fn favorites_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tuibe")
        .join("favorites.json")
}
