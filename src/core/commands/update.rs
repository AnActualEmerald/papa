use crate::{
    config::CONFIG,
    model::ModName,
    readln,
    traits::Index,
    utils::{download_and_install, to_file_size_string},
};
use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::{model::ModVersion, prelude::*};
use tracing::debug;

pub fn update(yes: bool) -> Result<()> {
    println!("Checking for outdated packages...");
    let index = get_package_index()?;
    let local = find_mods(CONFIG.install_dir())?;
    let mut outdated: Vec<(ModName, &ModVersion)> = vec![];
    for l in local {
        let Ok(l) = &l else { continue };
        debug!("Checking if mod '{}' is out of date", l.manifest.name);

        if let Some(m) = index.get_item(&l.into()) {
            debug!("Checking mod {:?}", m);
            if m.latest != l.manifest.version_number {
                outdated.push((m.into(), m.get_latest().expect("Missing latest version")));
            }
        }
    }

    if outdated.len() == 0 {
        println!("All packages up to date!");
        return Ok(());
    }

    let filesize = to_file_size_string(outdated.iter().map(|(_, v)| v.file_size).sum());

    println!("Found {} outdated packages:\n", outdated.len().bold());
    for (name, _) in outdated.iter() {
        println!("  {}", name.bright_cyan());
    }
    println!("\nTotal download size: {}", filesize.bold());

    let answer = if !yes {
        readln!("OK? [Y/n]: ")?
    } else {
        String::new()
    };

    if !answer.to_lowercase().starts_with("n") {
        download_and_install(outdated)?;
        Ok(())
    } else {
        Ok(())
    }
}
