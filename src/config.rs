use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub sounds_dir: String,
    pub volume: f32,
    pub events: HashMap<String, Vec<String>>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path()?;
        let raw = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn sounds_dir_abs(&self) -> PathBuf {
        // HOOKPLAYER_SOUNDS_DIR env var overrides config
        let raw = std::env::var("HOOKPLAYER_SOUNDS_DIR")
            .unwrap_or_else(|_| self.sounds_dir.clone());
        expand_tilde(&raw)
    }

    pub fn sounds_for_event(&self, event: &str) -> Vec<PathBuf> {
        let base = self.sounds_dir_abs();
        let key = if self.events.contains_key(event) {
            event
        } else {
            "unknown"
        };
        self.events
            .get(key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|f| base.join(f))
            .collect()
    }
}

/// Updates sounds_dir in the config file and returns the resolved path.
pub fn set_sounds_dir(new_path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = config_path()?;
    let raw = std::fs::read_to_string(&path)?;

    let updated = raw
        .lines()
        .map(|line| {
            if line.starts_with("sounds_dir") {
                format!("sounds_dir = \"{}\"", new_path.replace('"', "\\\""))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let updated = if raw.ends_with('\n') {
        updated + "\n"
    } else {
        updated
    };

    std::fs::write(&path, updated)?;
    Ok(expand_tilde(new_path))
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(path)
}

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
    Ok(PathBuf::from(home).join(".config/hookplayer/config.toml"))
}
