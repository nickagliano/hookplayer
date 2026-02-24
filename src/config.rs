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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config(sounds_dir: &str, events: HashMap<String, Vec<String>>) -> Config {
        Config {
            sounds_dir: sounds_dir.to_string(),
            volume: 0.5,
            events,
        }
    }

    // --- expand_tilde ---

    #[test]
    fn expand_tilde_replaces_home() {
        let home = std::env::var("HOME").unwrap();
        let result = expand_tilde("~/sounds");
        assert_eq!(result, PathBuf::from(&home).join("sounds"));
    }

    #[test]
    fn expand_tilde_leaves_absolute_path_unchanged() {
        let result = expand_tilde("/absolute/path");
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn expand_tilde_leaves_relative_path_unchanged() {
        let result = expand_tilde("relative/path");
        assert_eq!(result, PathBuf::from("relative/path"));
    }

    // --- sounds_dir_abs ---

    #[test]
    fn sounds_dir_abs_uses_env_var_override() {
        // Use a unique env var key to avoid parallel test conflicts
        unsafe { std::env::set_var("HOOKPLAYER_SOUNDS_DIR", "/override/sounds") };
        let cfg = make_config("~/default/sounds", HashMap::new());
        let result = cfg.sounds_dir_abs();
        unsafe { std::env::remove_var("HOOKPLAYER_SOUNDS_DIR") };
        assert_eq!(result, PathBuf::from("/override/sounds"));
    }

    #[test]
    fn sounds_dir_abs_falls_back_to_config() {
        unsafe { std::env::remove_var("HOOKPLAYER_SOUNDS_DIR") };
        let home = std::env::var("HOME").unwrap();
        let cfg = make_config("~/mysounds", HashMap::new());
        assert_eq!(cfg.sounds_dir_abs(), PathBuf::from(&home).join("mysounds"));
    }

    // --- sounds_for_event ---

    #[test]
    fn sounds_for_event_returns_configured_sounds() {
        let mut events = HashMap::new();
        events.insert("start".to_string(), vec!["pack/hello.mp3".to_string()]);
        let cfg = make_config("/sounds", events);

        let paths = cfg.sounds_for_event("start");
        assert_eq!(paths, vec![PathBuf::from("/sounds/pack/hello.mp3")]);
    }

    #[test]
    fn sounds_for_event_falls_back_to_unknown() {
        let mut events = HashMap::new();
        events.insert("unknown".to_string(), vec!["pack/default.mp3".to_string()]);
        let cfg = make_config("/sounds", events);

        let paths = cfg.sounds_for_event("unrecognized_event");
        assert_eq!(paths, vec![PathBuf::from("/sounds/pack/default.mp3")]);
    }

    #[test]
    fn sounds_for_event_returns_empty_when_no_match_and_no_unknown() {
        let cfg = make_config("/sounds", HashMap::new());
        let paths = cfg.sounds_for_event("start");
        assert!(paths.is_empty());
    }

    #[test]
    fn sounds_for_event_returns_multiple_sounds() {
        let mut events = HashMap::new();
        events.insert(
            "notify".to_string(),
            vec!["pack/a.mp3".to_string(), "pack/b.mp3".to_string()],
        );
        let cfg = make_config("/sounds", events);

        let paths = cfg.sounds_for_event("notify");
        assert_eq!(paths.len(), 2);
    }
}
