use serde_json::{json, Value};
use std::io::{self, Write};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn temp_settings(content: Option<Value>) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let path = claude_dir.join("settings.json");
        if let Some(val) = content {
            fs::write(&path, serde_json::to_string_pretty(&val).unwrap()).unwrap();
        }
        (dir, path)
    }

    fn read_settings(path: &PathBuf) -> Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    // ── has_hookplayer_hook ───────────────────────────────────────────────────

    #[test]
    fn detects_present_hookplayer_hook() {
        let settings = json!({
            "hooks": {
                "Stop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer stop" }] }]
            }
        });
        assert!(has_hookplayer_hook(&settings, "Stop"));
    }

    #[test]
    fn detects_absent_hook_event() {
        let settings = json!({ "hooks": {} });
        assert!(!has_hookplayer_hook(&settings, "Stop"));
    }

    #[test]
    fn detects_absent_when_different_command() {
        let settings = json!({
            "hooks": {
                "Stop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "echo done" }] }]
            }
        });
        assert!(!has_hookplayer_hook(&settings, "Stop"));
    }

    #[test]
    fn detects_absent_when_no_hooks_key() {
        let settings = json!({});
        assert!(!has_hookplayer_hook(&settings, "Stop"));
    }

    #[test]
    fn detects_hookplayer_among_multiple_entries() {
        let settings = json!({
            "hooks": {
                "PostToolUse": [
                    { "matcher": "Write", "hooks": [{ "type": "command", "command": "echo wrote" }] },
                    { "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer notify" }] }
                ]
            }
        });
        assert!(has_hookplayer_hook(&settings, "PostToolUse"));
    }

    // ── setup::run ────────────────────────────────────────────────────────────

    #[test]
    fn yes_creates_settings_file_when_absent() {
        let (dir, path) = temp_settings(None);
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        let result = run(true);

        unsafe { std::env::set_var("HOME", orig_home) };
        result.unwrap();

        assert!(path.exists());
        let settings = read_settings(&path);
        assert!(settings["hooks"]["Stop"].is_array());
        assert!(settings["hooks"]["PreToolUse"].is_array());
        assert!(settings["hooks"]["PostToolUse"].is_array());
        assert!(settings["hooks"]["Notification"].is_array());
    }

    #[test]
    fn yes_adds_all_four_hooks() {
        let (dir, path) = temp_settings(Some(json!({})));
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        run(true).unwrap();

        unsafe { std::env::set_var("HOME", orig_home) };
        let settings = read_settings(&path);

        for event in ["PreToolUse", "PostToolUse", "Notification", "Stop"] {
            let arr = settings["hooks"][event].as_array().unwrap();
            assert!(!arr.is_empty(), "missing hook for {event}");
            let cmd = arr[0]["hooks"][0]["command"].as_str().unwrap();
            assert!(cmd.starts_with("hookplayer "), "wrong command for {event}: {cmd}");
        }
    }

    #[test]
    fn yes_preserves_existing_non_hookplayer_hooks() {
        let existing = json!({
            "hooks": {
                "PostToolUse": [
                    { "matcher": "Write", "hooks": [{ "type": "command", "command": "echo wrote" }] }
                ]
            }
        });
        let (dir, path) = temp_settings(Some(existing));
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        run(true).unwrap();

        unsafe { std::env::set_var("HOME", orig_home) };
        let settings = read_settings(&path);

        let post = settings["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 2, "should have original + hookplayer entry");
        let commands: Vec<&str> = post
            .iter()
            .map(|e| e["hooks"][0]["command"].as_str().unwrap())
            .collect();
        assert!(commands.contains(&"echo wrote"));
        assert!(commands.contains(&"hookplayer notify"));
    }

    #[test]
    fn yes_is_idempotent() {
        let (dir, path) = temp_settings(Some(json!({})));
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        run(true).unwrap();
        run(true).unwrap(); // second run should be a no-op

        unsafe { std::env::set_var("HOME", orig_home) };
        let settings = read_settings(&path);

        // Each event array should have exactly one hookplayer entry
        for event in ["PreToolUse", "PostToolUse", "Notification", "Stop"] {
            let arr = settings["hooks"][event].as_array().unwrap();
            let hp_count = arr.iter().filter(|e| {
                e["hooks"][0]["command"]
                    .as_str()
                    .map(|c| c.starts_with("hookplayer "))
                    .unwrap_or(false)
            }).count();
            assert_eq!(hp_count, 1, "duplicate hook added for {event}");
        }
    }

    #[test]
    fn yes_skips_already_configured_events() {
        let existing = json!({
            "hooks": {
                "Stop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer stop" }] }]
            }
        });
        let (dir, path) = temp_settings(Some(existing));
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        run(true).unwrap();

        unsafe { std::env::set_var("HOME", orig_home) };
        let settings = read_settings(&path);

        // Stop should still have exactly one entry
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
    }

    #[test]
    fn yes_writes_valid_json() {
        let (dir, path) = temp_settings(None);
        let orig_home = std::env::var("HOME").unwrap_or_default();
        unsafe { std::env::set_var("HOME", dir.path()) };

        run(true).unwrap();

        unsafe { std::env::set_var("HOME", orig_home) };
        let raw = fs::read_to_string(&path).unwrap();
        assert!(serde_json::from_str::<Value>(&raw).is_ok());
    }
}

struct Hook {
    event:   &'static str,
    command: &'static str,
}

const HOOKS: &[Hook] = &[
    Hook { event: "PreToolUse",   command: "hookplayer permission" },
    Hook { event: "PostToolUse",  command: "hookplayer notify"     },
    Hook { event: "Notification", command: "hookplayer notify"     },
    Hook { event: "Stop",         command: "hookplayer stop"       },
];

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".claude/settings.json")
}

/// Returns true if a hookplayer hook is already present for this event.
fn has_hookplayer_hook(settings: &Value, event: &str) -> bool {
    let Some(event_arr) = settings
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    event_arr.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|inner| {
                inner.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.starts_with("hookplayer "))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    })
}

pub fn run(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path();

    let mut settings: Value = if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        serde_json::from_str(&raw).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let pending: Vec<&Hook> = HOOKS
        .iter()
        .filter(|h| !has_hookplayer_hook(&settings, h.event))
        .collect();

    if pending.is_empty() {
        println!("hookplayer is already wired into Claude Code ({}).", path.display());
        return Ok(());
    }

    if !yes {
        println!("\nhookplayer will add the following hooks to {}:\n", path.display());
        for h in HOOKS {
            if has_hookplayer_hook(&settings, h.event) {
                println!("  {:<14}  {}  (already present, skipping)", h.event, h.command);
            } else {
                println!("  {:<14}  {}", h.event, h.command);
            }
        }
        print!("\nProceed? [y/N] ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for h in &pending {
        let entry = json!({
            "matcher": "",
            "hooks": [{ "type": "command", "command": h.command }]
        });
        settings
            .as_object_mut()
            .unwrap()
            .entry("hooks")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap()
            .entry(h.event)
            .or_insert(json!([]))
            .as_array_mut()
            .unwrap()
            .push(entry);

        if yes {
            println!("  {} → {}", h.event, h.command);
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&settings)? + "\n")?;

    println!("\nDone. Claude Code will play sounds for hook events.");
    Ok(())
}
