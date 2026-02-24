use std::io::Write;

const REPO: &str = "nickagliano/hookplayer";

pub fn print_version() {
    println!("hookplayer {}", env!("CARGO_PKG_VERSION"));
}

pub fn update() -> Result<(), Box<dyn std::error::Error>> {
    let current = env!("CARGO_PKG_VERSION");

    let client = reqwest::blocking::Client::builder()
        .user_agent("hookplayer")
        .build()?;

    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let resp: serde_json::Value = client.get(&url).send()?.json()?;

    let tag = resp["tag_name"]
        .as_str()
        .ok_or("missing tag_name in GitHub response")?;
    let latest = tag.trim_start_matches('v');

    if !is_newer(latest, current) {
        println!("hookplayer is up to date ({})", current);
        return Ok(());
    }

    println!("Update available: {} -> {}", current, latest);

    let os = match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        other => return Err(format!("unsupported OS: {}", other).into()),
    };
    let arch = std::env::consts::ARCH; // "x86_64" or "aarch64"
    let asset = format!("hookplayer-{}-{}", os, arch);
    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, tag, asset
    );

    println!("Downloading {}...", asset);
    let bytes = client.get(&download_url).send()?.bytes()?;

    let current_exe = std::env::current_exe()?;
    let tmp = current_exe.with_extension("update_tmp");

    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(&bytes)?;
    drop(f);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::rename(&tmp, &current_exe)?;
    println!("Updated to {}.", latest);
    Ok(())
}

pub(crate) fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(latest) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch() {
        assert!(is_newer("0.1.1", "0.1.0"));
    }

    #[test]
    fn newer_minor() {
        assert!(is_newer("0.2.0", "0.1.9"));
    }

    #[test]
    fn newer_major() {
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn not_newer_equal() {
        assert!(!is_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn not_newer_older() {
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn handles_double_digit_minor() {
        assert!(is_newer("0.10.0", "0.9.0"));
    }
}
