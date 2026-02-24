mod config;
mod player;
mod registry;
mod updater;

use rand::seq::SliceRandom;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("unknown");

    match cmd {
        "--version" | "-V" => {
            updater::print_version();
        }
        "update" => {
            if let Err(e) = updater::update() {
                eprintln!("hookplayer: update failed: {}", e);
                std::process::exit(1);
            }
        }
        "list" => {
            if let Err(e) = registry::list_packs() {
                eprintln!("hookplayer: {}", e);
                std::process::exit(1);
            }
        }
        "download" => {
            let pack_name = match args.get(2) {
                Some(n) => n,
                None => {
                    eprintln!("hookplayer: usage: hookplayer download <pack>");
                    std::process::exit(1);
                }
            };
            let cfg = load_config();
            let sounds_dir = cfg.sounds_dir_abs();
            if let Err(e) = registry::download_pack(pack_name, &sounds_dir) {
                eprintln!("hookplayer: {}", e);
                std::process::exit(1);
            }
        }
        "dir" => {
            let cfg = load_config();
            println!("{}", cfg.sounds_dir_abs().display());
        }
        "set-dir" => {
            let new_path = match args.get(2) {
                Some(p) => p,
                None => {
                    eprintln!("hookplayer: usage: hookplayer set-dir <path>");
                    std::process::exit(1);
                }
            };
            match config::set_sounds_dir(new_path) {
                Ok(resolved) => {
                    println!("sounds_dir set to {}", new_path);
                    if !resolved.exists() {
                        eprintln!(
                            "Warning: directory does not exist yet: {}\n\
                             Create it manually or run 'hookplayer download <pack>' to populate it.",
                            resolved.display()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("hookplayer: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "use" => {
            let packs_arg = match args.get(2) {
                Some(p) => p,
                None => {
                    eprintln!("hookplayer: usage: hookplayer use <pack>[,pack2,...]");
                    std::process::exit(1);
                }
            };
            let pack_names: Vec<&str> = packs_arg.split(',').map(|s| s.trim()).collect();
            println!("Configuring events from: {}", pack_names.join(", "));
            match registry::build_events_for_packs(&pack_names) {
                Ok(events) => {
                    if let Err(e) = config::set_events(&events) {
                        eprintln!("hookplayer: {}", e);
                        std::process::exit(1);
                    }
                    println!("Done. Config updated.");
                }
                Err(e) => {
                    eprintln!("hookplayer: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "packs" => {
            let cfg = load_config();
            let sounds_dir = cfg.sounds_dir_abs();
            match std::fs::read_dir(&sounds_dir) {
                Ok(entries) => {
                    let mut names: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                        .collect();
                    names.sort();
                    for name in &names {
                        println!("  {}", name);
                    }
                    println!("\n{} pack(s) installed", names.len());
                }
                Err(e) => {
                    eprintln!("hookplayer: could not read sounds dir: {}", e);
                    std::process::exit(1);
                }
            }
        }
        event => {
            let cfg = load_config();
            let sounds = cfg.sounds_for_event(event);

            if sounds.is_empty() {
                if cfg.events.is_empty() {
                    eprintln!(
                        "hookplayer: no sounds configured.\n\
                         Run 'hookplayer list' to browse packs, then 'hookplayer use <pack>' to get started."
                    );
                }
                return;
            }

            let chosen = sounds.choose(&mut rand::thread_rng()).unwrap();

            if let Err(e) = player::play(chosen, cfg.volume) {
                eprintln!("hookplayer: playback error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn load_config() -> config::Config {
    match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hookplayer: failed to load config: {}", e);
            std::process::exit(1);
        }
    }
}
