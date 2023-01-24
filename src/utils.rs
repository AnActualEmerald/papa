use std::{fs, path::Path};

use crate::{
    config::{CONFIG, DIRS},
    model::ModName,
};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use regex::Regex;
use thermite::{
    model::ModVersion,
    prelude::{download_file_with_progress, install_mod},
};
use tracing::debug;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^(\w+)\.(\w+)(?:@(\d+\.\d+\.\d+))?$").unwrap();
}

pub fn validate_modnames(input: &str) -> Result<ModName, String> {
    if let Some(captures) = RE.captures(input) {
        let mut name = ModName::default();
        if let Some(author) = captures.get(1) {
            name.author = author.as_str().to_string();
        }

        if let Some(n) = captures.get(2) {
            name.name = n.as_str().to_string();
        }

        name.version = captures.get(3).map(|v| v.as_str().to_string());

        Ok(name)
    } else {
        Err(format!(
            "Mod name '{input}' should be in 'Author.ModName' format"
        ))
    }
}

pub fn to_file_size_string(size: u64) -> String {
    if size / 1_000_000 >= 1 {
        let size = size as f64 / 1_048_576f64;

        format!("{:.2} MB", size)
    } else {
        let size = size as f64 / 1024f64;
        format!("{:.2} KB", size)
    }
}

pub fn ensure_dir(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();

    debug!("Checking if path '{}' exists", dir.display());
    if !dir.try_exists()? {
        debug!("Path '{}' doesn't exist, creating it", dir.display());
        fs::create_dir_all(dir)?;
    } else {
        debug!("Path '{}' already exists", dir.display());
    }

    Ok(())
}

pub fn download_and_install(mods: Vec<(ModName, impl AsRef<ModVersion>)>) -> Result<()> {
    println!("Downloading packages...");
    let mut files = vec![];
    let cache_dir = DIRS.cache_dir();
    for (mn, v) in mods {
        let v = v.as_ref();
        // flush!()?;
        let filename = cache_dir.join(format!("{}.zip", mn));
        ensure_dir(&cache_dir)?;
        let pb = ProgressBar::new(v.file_size)
            .with_style(
                ProgressStyle::with_template("{msg}{bar} {bytes}/{total_bytes} {duration}")?
                    .progress_chars(".. "),
            )
            .with_message(format!("Downloading {mn}..."));
        let file = download_file_with_progress(&v.url, filename, |current, _| {
            pb.set_position(current);
        })
        .context(format!("Error downloading {}", mn))?;
        pb.finish();
        files.push((mn.author, file));
    }
    println!("Installing packages...");
    for (author, f) in files {
        if !CONFIG.is_server() {
            ensure_dir(CONFIG.install_dir())?;
            install_mod(author, &f, CONFIG.install_dir())?;
        } else {
            todo!();
        }
    }
    println!("Done!");
    Ok(())
}
