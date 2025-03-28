use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::config::{DIRS, InstallType, SteamType};
use crate::model::Cache;
use crate::traits::{Answer, Index};
use crate::utils::{ensure_dir, init_msg};
use crate::{NstarCommands, config::CONFIG, model::ModName};
use crate::{get_answer, modfile};
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use steamlocate::SteamDir;
use thermite::model::{Mod, ModJSON};
use thermite::prelude::*;
use tracing::{debug, warn};

use super::profile;

pub fn northstar(commands: &NstarCommands) -> Result<()> {
    match commands {
        NstarCommands::Init {
            force,
            path,
            no_cache,
        } => init_ns(*force, path.as_ref(), *no_cache)?,
        NstarCommands::Update {} => {
            update_ns()?;
        }
        NstarCommands::Reset { yes } => {
            reset(*yes)?;
        }
    }

    Ok(())
}

fn get_titanfall() -> Result<PathBuf> {
    let maybe = SteamDir::locate_multiple()?
        .iter()
        .filter_map(|dir| dir.find_app(TITANFALL2_STEAM_ID).ok().flatten())
        .map(|(app, lib)| lib.resolve_app_dir(&app))
        .next();

    maybe.ok_or_else(|| anyhow!("Failed to find titanfall 2"))
}

fn init_ns(force: bool, path: Option<impl AsRef<Path>>, no_cache: bool) -> Result<()> {
    let (titanfall_path, steam) = if let Some(path) = path {
        (path.as_ref().to_path_buf(), false)
    } else if let Ok(dir) = get_titanfall() {
        (dir, true)
    } else {
        eprintln!(
            "Couldn't automatically locate your Titanfall installation.\nPlease make sure it's installed or provide a path."
        );
        return Err(anyhow!("Unable to locate Titanfall 2 in Steam libraries"));
    };

    debug!("Installing N* to '{}'", titanfall_path.display());

    //try to detect existing installation
    if !force && titanfall_path.join("NorthstarLauncher.exe").try_exists()? {
        println!("Found an existing Northstar installation, updating config!");
        let mut new_config = CONFIG.clone();
        new_config.set_game_dir(titanfall_path.clone());

        if steam {
            new_config.set_install_type(InstallType::Steam(SteamType::determine()?));
        }

        new_config.save()?;
        return Ok(());
    }

    let index = get_package_index()?;
    let nsmod = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or(anyhow!("Couldn't find Northstar in the package index"))?;

    ensure_dir(DIRS.cache_dir())?;

    let cache = Cache::from_dir(DIRS.cache_dir())?;
    let ns_name = ModName::from(nsmod);

    let nsfile = if !no_cache && let Some(cached_ns) = cache.get(&ns_name) {
        println!("Using cached version of {}", ns_name.bright_cyan());

        OpenOptions::new().read(true).open(cached_ns)?
    } else {
        let mut nsfile = modfile!(
            DIRS.cache_dir()
                .join(format!("{}.zip", ModName::from(nsmod)))
        )?;
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

        nsfile
    };

    let pb = ProgressBar::new_spinner()
        .with_style(ProgressStyle::with_template("{prefix} {msg} {spinner}")?)
        .with_prefix("Installing Northstar...")
        .with_message("");
    pb.enable_steady_tick(Duration::from_millis(50));
    install_northstar(&nsfile, &titanfall_path)?;
    pb.finish_with_message("Done!");

    let mut new_config = CONFIG.clone();
    new_config.set_game_dir(titanfall_path.clone());
    if steam {
        new_config.set_install_type(InstallType::Steam(SteamType::determine()?));
    }
    new_config.save()?;

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
            let Ok(p) = ns_client.join("..").join("..").join("..").canonicalize() else {
                warn!("Northstar installation seems to be invalid, aborting update");
                println!(
                    "Can't update this Northstar installation. Try running {} first",
                    "papa ns init".bright_cyan()
                );
                return Ok(false);
            };
            p
        };

        init_ns(true, Some(path), false)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn update_check() -> Result<Option<(PathBuf, Mod)>> {
    let ns_client_path = CONFIG
        .current_profile_dir()
        .ok_or_else(|| anyhow!("Unable to get current profile directory from config"))?
        .join("mods")
        .join("Northstar.Client");
    let index = get_package_index()?;

    if !ns_client_path.try_exists()? {
        debug!(
            "Didn't find 'Northstar.Client' at '{}'",
            ns_client_path.display()
        );
        return Err(anyhow!("Unable to find Northstar.Client mod"));
    }

    let mod_json_path = ns_client_path.join("mod.json");
    let mod_json: ModJSON = serde_json::from_slice(&fs::read(mod_json_path)?)?;

    // else {
    //     debug!(
    //         "Didn't find 'Northstar.Client' in '{}'",
    //         search_dir.display()
    //     );
    //     return Err(anyhow!("Unable to find Northstar.Client mod"));
    // };

    let remote_ns = index
        .get_item(&ModName::new("northstar", "Northstar", None))
        .ok_or_else(|| anyhow!("Unable to find Northstar in Thunderstore index"))?;

    if mod_json.version == remote_ns.latest {
        Ok(None)
    } else {
        Ok(Some((ns_client_path, remote_ns.clone())))
    }
}

const NSTAR_FILES: [&str; 8] = [
    "Northstar.dll",
    "LEGAL.txt",
    "NorthstarLauncher.exe",
    "r2ds.bat",
    "bin/x64_dedi/d3d11.dll",
    "bin/x64_dedi/GFSDK_SSAO.win64.dll",
    "bin/x64_dedi/GFSDK_TXAA.win64.dll",
    "bin/x64_retail/wsock32.dll",
];

fn reset(yes: bool) -> Result<()> {
    if !yes {
        let ans = get_answer!(
            yes,
            format!(
                "{0}This action will remove Northstar and all related files{0}\n\nAre you sure you want to continue? [y/N]: ",
                "!!!".bright_red()
            )
        )?;
        if !ans.is_yes() {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    let root = CONFIG.game_dir().ok_or_else(init_msg)?;

    for file in NSTAR_FILES {
        let path = root.join(file);
        println!("Removing file {path:?}");
        fs::remove_file(path)?;
    }

    let profiles = profile::find_profiles(root)?;

    for dir in profiles {
        println!("Removing profile {:?}", dir.file_name().expect("file name"));
        fs::remove_dir_all(dir)?;
    }

    println!(
        "All known files removed. There may be extra files left over that were generated by Northstar; these are safe to remove. Run {} any time to reinstall Northstar.",
        "papa ns init".bright_cyan()
    );
    Ok(())
}
