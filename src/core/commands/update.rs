use std::{collections::HashMap, fs};

use crate::{
    config::CONFIG,
    core::commands::northstar,
    get_answer,
    model::ModName,
    traits::{Answer, Indexed},
    utils::{download_and_install, to_file_size_string},
};
use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::{
    model::{InstalledMod, ModVersion},
    prelude::*,
};
use tracing::{debug, warn};

pub fn update(yes: bool, no_cache: bool) -> Result<()> {
    println!("Checking for outdated packages...");
    let index = get_package_index()?;
    let local: Vec<InstalledMod> = find_mods(CONFIG.install_dir()?)?;
    let mut outdated: HashMap<ModName, &ModVersion> = HashMap::new();

    for l in &local {
        debug!("Checking if mod '{}' is out of date", l.manifest.name);

        if let Some(m) = index.get_item(&l.into()) {
            if m.author.to_lowercase() == "northstar" {
                debug!("Skipping Northstar core mod");
                continue;
            }
            debug!("Checking mod {:?}", m);
            if m.latest != l.manifest.version_number {
                outdated.insert(m.into(), m.get_latest().expect("Missing latest version"));
            }
        }
    }

    let ns_update = northstar::update_check().unwrap_or(None).is_some();

    if outdated.is_empty() {
        if ns_update {
            return ns_prompt();
        } else {
            println!("All packages up to date!");
            return Ok(());
        }
    }

    let filesize = to_file_size_string(outdated.values().map(|v| v.file_size).sum());

    println!("Found {} outdated packages:\n", outdated.len().bold());
    for (name, _) in outdated.iter() {
        println!("  {}", name.bright_cyan());
    }
    println!("\nTotal download size: {}", filesize.bold());

    let answer = get_answer!(yes)?;

    if !answer.is_no() {
        let installed =
            download_and_install(outdated.clone().into_iter().collect(), !no_cache, false)?;

        //clean any old folders
        for l in &local {
            debug!("Checking if {} should be cleaned", ModName::from(l));
            if !installed.contains(&l.path)
                && outdated
                    .keys()
                    .any(|k| k.author == l.author && k.name == l.manifest.name)
            {
                if let Err(e) = fs::remove_dir_all(&l.path) {
                    warn!("Unable to remove old mod folder {}", l.path.display());
                    debug!("{e}")
                } else {
                    debug!("Cleaned old folder '{}'", l.path.display());
                }
            }
        }

        if ns_update {
            ns_prompt()?;
        }

        Ok(())
    } else {
        Ok(())
    }
}

fn ns_prompt() -> Result<()> {
    if !northstar::update_ns()? {
        println!(
            "Run {} at any time to update",
            "papa ns update".bright_cyan()
        );
    }

    Ok(())
}
