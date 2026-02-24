use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

const REGISTRY_URL: &str = "https://peonping.github.io/registry/index.json";

#[derive(Debug, Deserialize)]
pub struct RegistryPack {
    pub name: String,
    pub display_name: String,
    pub source_repo: String,
    pub source_ref: String,
    pub source_path: String,
}

#[derive(Deserialize)]
struct Registry {
    packs: Vec<RegistryPack>,
}

#[derive(Deserialize)]
struct Manifest {
    categories: HashMap<String, Category>,
}

#[derive(Deserialize)]
struct Category {
    sounds: Vec<SoundEntry>,
}

#[derive(Deserialize)]
struct SoundEntry {
    file: String,
}

pub fn fetch_registry() -> Result<Vec<RegistryPack>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(REGISTRY_URL)?;
    let registry: Registry = resp.json()?;
    Ok(registry.packs)
}

pub fn list_packs() -> Result<(), Box<dyn std::error::Error>> {
    let packs = fetch_registry()?;
    for p in &packs {
        println!("  {:<28} {}", p.name, p.display_name);
    }
    println!("\n{} packs available", packs.len());
    Ok(())
}

pub fn download_pack(pack_name: &str, sounds_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let packs = fetch_registry()?;
    let pack = packs
        .iter()
        .find(|p| p.name == pack_name)
        .ok_or_else(|| format!("pack '{}' not found in registry", pack_name))?;

    let base_url = if pack.source_path.is_empty() {
        format!(
            "https://raw.githubusercontent.com/{}/{}",
            pack.source_repo, pack.source_ref
        )
    } else {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            pack.source_repo, pack.source_ref, pack.source_path
        )
    };

    println!("Fetching manifest for '{}'...", pack.display_name);
    let manifest_url = format!("{}/openpeon.json", base_url);
    let manifest: Manifest = reqwest::blocking::get(&manifest_url)?.json()?;

    // Collect unique sound filenames
    let mut filenames: HashSet<String> = HashSet::new();
    for category in manifest.categories.values() {
        for sound in &category.sounds {
            if let Some(basename) = std::path::Path::new(&sound.file).file_name() {
                filenames.insert(basename.to_string_lossy().into_owned());
            }
        }
    }

    let out_dir = sounds_dir.join(&pack.name);
    std::fs::create_dir_all(&out_dir)?;

    println!("Downloading {} sounds into sounds/{}/", filenames.len(), pack.name);

    let client = reqwest::blocking::Client::new();
    for filename in &filenames {
        let url = format!("{}/sounds/{}", base_url, filename);
        let dest = out_dir.join(filename);

        let bytes = client.get(&url).send()?.bytes()?;
        let mut f = std::fs::File::create(&dest)?;
        f.write_all(&bytes)?;
        println!("  + {}", filename);
    }

    println!("Done. Pack '{}' installed.", pack.name);
    Ok(())
}
