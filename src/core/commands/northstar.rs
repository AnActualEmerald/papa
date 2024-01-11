use std::path::Path;
use std::time::Duration;

use crate::config::DIRS;
use crate::traits::{Answer, Index};
use crate::{
    config::{write_config, CONFIG},
    model::ModName,
    NstarCommands,
};
use crate::{get_answer, modfile};
use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use thermite::model::{InstalledMod, Mod};
use thermite::prelude::*;
use tracing::{debug, warn};

pub fn northstar(commands: &NstarCommands) -> Result<()> {
    match commands {
        NstarCommands::Init { force, path } => init_ns(*force, path.as_ref())?,
        NstarCommands::Update {} => {
            update_ns()?;
        }
    }

    Ok(())
}

fn init_ns(force: bool, path: Option<impl AsRef<Path>>) -> Result<()> {
    let titanfall = if let Some(path) = path {
        path.as_ref().canonicalize()?
    } else if let Some(dir) = titanfall() {
        dir
    } else {
        println!(
            "Couldn't automatically locate your Titanfall installation.\nPlease provide a path."
        );
        return Err(anyhow!("Unable to locate Titanfall 2 in Steam libraries"));
    };

    debug!("Installing N* to '{}'", titanfall.display());

    //try to detect existing installation
    if !force && titanfall.join("NorthstarLauncher.exe").try_exists()? {
        println!("Found an existing Northstar installation, updating config!");
        let mut new_config = CONFIG.clone();
        new_config.set_game_dir(titanfall.clone());
        new_config.set_install_dir(titanfall.join("R2Northstar").join("packages"));
        write_config(&new_config)?;
        return Ok(());
    }

    let index = get_package_index()?;
    let nsmod = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or(anyhow!("Couldn't find Northstar in the package index"))?;

    std::fs::create_dir_all(DIRS.cache_dir())?;
    //TODO: remove this file after install
    let mut nsfile = modfile!(DIRS
        .cache_dir()
        .join(format!("{}.zip", ModName::from(nsmod))))?;
    let nsversion = nsmod.get_latest().expect("N* mod missing latest version");

    let pb = ProgressBar::new(nsversion.file_size)
        .with_style(
            ProgressStyle::with_template("{msg}{bar:.cyan} {bytes}/{total_bytes} {duration}")?
                .progress_chars(".. "),
        )
        .with_message(format!(
            "Downloading Northstar version {}",
            nsmod.latest.bold()
        ));
    download_with_progress(&mut nsfile, &nsversion.url, |delta, _, _| {
        pb.inc(delta);
    })?;
    pb.finish();
    println!();

    let pb = ProgressBar::new_spinner()
        .with_style(ProgressStyle::with_template("{prefix} {msg} {spinner}")?)
        .with_prefix("Installing Northstar...")
        .with_message("");
    pb.enable_steady_tick(Duration::from_millis(50));
    install_northstar(&nsfile, &titanfall)?;
    pb.finish_with_message("Done!");

    let mut new_config = CONFIG.clone();
    new_config.set_game_dir(titanfall.clone());
    new_config.set_install_dir(titanfall.join("R2Northstar").join("packages"));
    write_config(&new_config)?;

    Ok(())
}

pub fn update_ns() -> Result<bool> {
    let Some((ns_client, remote_ns)) = update_check()? else {
        println!("Northstar is up to date!");
        return Ok(false);
    };

    print!(
        "Northstar v{} is available, would you like to install it? [Y/n]: ",
        remote_ns.latest
    );
    let res = get_answer!(false, "")?;

    if !res.is_no() {
        let path = if let Some(dir) = CONFIG.game_dir() {
            dir.clone()
        } else {
            // the fact that this works is kinda funny but also makes my life massively easier
            let Ok(p) = ns_client
                .path
                .join("..")
                .join("..")
                .join("..")
                .canonicalize()
            else {
                warn!("Northstar installation seems to be invalid, aborting update");
                println!(
                    "Can't update this Northstar installation. Try running {} first",
                    "papa ns init".bright_cyan()
                );
                return Ok(false);
            };
            p
        };

        init_ns(true, Some(path))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn update_check() -> Result<Option<(InstalledMod, Mod)>> {
    let index = get_package_index()?;
    let mods = find_mods(CONFIG.install_dir()?)?;
    let Some(ns_client) = mods.get_item(&ModName::new("northstar", "Northstar.Client", None))
    else {
        debug!(
            "Didn't find 'Northstar.Client' in '{}'",
            CONFIG.install_dir()?.display()
        );
        return Err(anyhow!("Unable to find Northstar.Client mod"));
    };

    let remote_ns = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or_else(|| anyhow!("Unable to find Northstar in Thunderstore index"))?;

    if ns_client.mod_json.version == remote_ns.latest {
        Ok(None)
    } else {
        Ok(Some((ns_client.clone(), remote_ns.clone())))
    }
}
