use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config::{CONFIG, DIRS},
    model::{Cache, ModName},
    modfile,
};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use lazy_static::lazy_static;
use owo_colors::OwoColorize;
use regex::Regex;
use thermite::{
    model::ModVersion,
    prelude::{download_with_progress, install_mod},
};
use tracing::debug;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^(\w+)\.(\w+)(?:@(\d+\.\d+\.\d+))?$").unwrap();
}

pub fn validate_modname(input: &str) -> Result<ModName, String> {
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

pub fn download_and_install(
    mods: Vec<(ModName, impl AsRef<ModVersion>)>,
    check_cache: bool,
    cont: bool,
) -> Result<Vec<PathBuf>> {
    if mods.is_empty() {
        println!("Nothing to do!");
        return Ok(vec![]);
    }

    println!("Downloading packages...");
    let mut files = vec![];
    let cache_dir = DIRS.cache_dir();
    ensure_dir(cache_dir)?;
    let cache = Cache::from_dir(cache_dir)?;

    for (mn, v) in mods {
        if check_cache {
            if let Some(path) = cache.get(&mn) {
                println!("Using cached version of {}", mn.bright_cyan());
                files.push((mn, modfile!(o, path)?));
                continue;
            }
        }
        let v = v.as_ref();
        // flush!()?;
        let filename = cache.to_cache_path(&mn);
        let pb = ProgressBar::new(v.file_size)
            .with_style(
                ProgressStyle::with_template("{msg}{bar} {bytes}/{total_bytes} {duration}")?
                    .progress_chars(".. "),
            )
            .with_message(format!("Downloading {}", mn.bright_cyan()));
        let mut file = modfile!(filename)?;
        download_with_progress(&mut file, &v.url, |delta, _, _| {
            pb.inc(delta);
        })
        .context(format!("Error downloading {}", mn.red()))?;
        pb.finish();
        files.push((mn, file));
    }

    let mut pb = ProgressBar::new_spinner()
        .with_style(
            ProgressStyle::with_template("{prefix}{msg}\t{spinner}\t{pos}/{len}")?
                .tick_chars("(|)|\0"),
        )
        .with_prefix("Installing ");
    pb.set_tab_width(1);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_length(files.len() as u64);

    let mut had_error = false;

    let mut installed = vec![];

    for (mn, f) in files.iter().progress_with(pb.clone()) {
        pb.set_message(format!("{}", mn.bright_cyan()));
        if !CONFIG.is_server() {
            ensure_dir(CONFIG.install_dir())?;
            match install_mod(&mn.author, f, CONFIG.install_dir()) {
                Err(e) => {
                    had_error = true;
                    pb.suspend(|| {
                        println!("Failed to install {}: {e}", mn.bright_red());
                        debug!("{e:?}");
                    });
                    if !cont {
                        pb.finish_and_clear();
                        println!("Aborted due to error");
                        return Err(e.into());
                    }
                }
                Ok(mut p) => {
                    pb.suspend(|| println!("Installed {}", mn.bright_cyan()));
                    installed.append(&mut p);
                }
            }
        } else {
            todo!();
        }
    }

    pb.set_prefix("");
    pb.set_tab_width(0);
    pb.finish_with_message("Installed ");
    if had_error {
        println!("Finished with errors")
    } else {
        println!("Done!");
    }
    Ok(installed)
}
