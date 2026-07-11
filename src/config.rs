use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub volume: f32,
    pub accent_color: String,
    pub keybindings: Keybindings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keybindings {
    pub search: String,
    pub play_pause: String,
    pub next: String,
    pub previous: String,
    pub volume_up: String,
    pub volume_down: String,
    pub favorite: String,
    pub quit: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 0.7,
            accent_color: "#7c3aed".to_string(),
            keybindings: Keybindings::default(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            search: "/".to_string(),
            play_pause: "Space".to_string(),
            next: "n".to_string(),
            previous: "p".to_string(),
            volume_up: "+".to_string(),
            volume_down: "-".to_string(),
            favorite: "f".to_string(),
            quit: "q".to_string(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tuibe")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}
