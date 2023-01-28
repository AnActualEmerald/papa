use std::path::PathBuf;

use crate::config::DIRS;
use crate::readln;
use crate::traits::Index;
use crate::{
    config::{write_config, CONFIG},
    model::ModName,
    NstarCommands,
};
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use thermite::prelude::*;
use tracing::debug;

pub fn northstar(commands: &NstarCommands) -> Result<()> {
    match commands {
        NstarCommands::Init { force, path } => init_ns(force, path)?,
        NstarCommands::Update {} => update_ns()?,
    }

    Ok(())
}

fn init_ns(force: &bool, path: &Option<PathBuf>) -> Result<()> {
    let titanfall = if let Some(path) = path {
        path.canonicalize()?
    } else if let Some(dir) = titanfall() {
        dir
    } else {
        return Err(anyhow!("Unable to locate Titanfall 2 in Steam libraries"));
    };

    debug!("Installing N* to '{}'", titanfall.display());

    //try to detect existing installation
    if !force {
        if titanfall.join("NorthstarLauncher.exe").try_exists()? {
            println!("Found an existing Northstar installation, updating config!");
            let mut new_config = CONFIG.clone();
            new_config.set_install_dir(titanfall.join("R2Northstar").join("mods"));
            write_config(&new_config)?;
            return Ok(());
        }
    }

    let index = get_package_index()?;
    let nsmod = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or(anyhow!("Couldn't find Northstar in the package index"))?;

    println!("Downloading Northstar version {}...", nsmod.latest.bold());

    let nsfile = download_file(
        &nsmod
            .get_latest()
            .expect("N* mod missing latest version")
            .url,
        DIRS.cache_dir()
            .join(format!("{}-{}.zip", nsmod.name, nsmod.latest)),
    )?;

    println!("Installing Northstar...");
    install_northstar(&nsfile, &titanfall)?;
    println!("Done!");

    let mut new_config = CONFIG.clone();
    new_config.set_install_dir(titanfall.join("R2Northstar").join("mods"));
    write_config(&new_config)?;

    Ok(())
}

fn update_ns() -> Result<()> {
    let index = get_package_index()?;
    let mods = find_mods(CONFIG.install_dir())?;
    let Some(ns_client) = mods.get_item(&ModName::new("northstar", "Northstar.Client", None)) else {
        debug!("Didn't find 'Northstar.Client' in '{}'", CONFIG.install_dir().display());
        return Err(anyhow!("Unable to find Northstar.Client mod"));
    };

    let remote_ns = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or_else(|| anyhow!("Unable to find Northstar in Thunderstore index"))?;

    if ns_client.mod_json.version == remote_ns.latest {
        println!("Northstar is up to date!");
        return Ok(());
    }

    println!(
        "Northstar v{} is available, would you like to install it?",
        remote_ns.latest
    );
    let _res = readln!("[Y/n]: ")?;

    Ok(())
}
