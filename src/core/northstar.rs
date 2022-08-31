use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

use log::debug;
use zip::ZipArchive;

use crate::api::model::Mod;

use anyhow::{anyhow, Context, Result};

use super::{actions, config, utils, Ctx};

pub async fn init_northstar(ctx: &mut Ctx, game_path: &Path) -> Result<()> {
    let version = install_northstar(ctx, game_path).await?;

    ctx.config.game_path = game_path.to_path_buf();
    ctx.config.nstar_version = Some(version);
    ctx.config
        .set_dir(game_path.join("R2Northstar").join("mods").to_str().unwrap());

    println!("Set mod directory to {}", ctx.config.mod_dir().display());
    config::save_config(ctx.dirs.config_dir(), &ctx.config)?;

    Ok(())
}

#[cfg(feature = "launcher")]
pub fn start_northstar(ctx: &Ctx) -> Result<(), String> {
    let game = ctx.config.game_path.join("NorthstarLauncher.exe");

    std::process::Command::new(game)
        // .stderr(Stdio::null())
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        .spawn()
        .expect("Unable to start game");

    Ok(())
}

///Update N* at the path that was initialized
///
///Returns OK if the path isn't set, but notifies the user
pub async fn update_northstar(ctx: &mut Ctx) -> Result<()> {
    if let Some(current) = &ctx.config.nstar_version {
        let index = utils::update_index(ctx.config.mod_dir(), &ctx.global_target).await;
        let nmod = index
            .iter()
            .find(|f| f.name.to_lowercase() == "northstar")
            .ok_or_else(|| anyhow!("Couldn't find Northstar on thunderstore???"))?;

        if nmod.version == *current {
            println!("Northstar is already up to date ({})", current);
            return Ok(());
        }

        if let Ok(s) = ctx.rl.readline(&format!(
            "Update Northstar to version {}? [Y/n]",
            nmod.version
        )) {
            if &s.to_lowercase() == "n" {
                return Ok(());
            }
        }

        do_install(ctx, nmod, &ctx.config.game_path).await?;
        ctx.config.nstar_version = Some(nmod.version.clone());
        config::save_config(ctx.dirs.config_dir(), &ctx.config)?;

        Ok(())
    } else {
        println!(
            "Only Northstar installations done with `papa northstar init` can be updated this way"
        );
        Ok(())
    }
}

///Install N* to the provided path
///
///Returns the version that was installed
pub async fn install_northstar(ctx: &Ctx, game_path: &Path) -> Result<String> {
    let index = utils::update_index(ctx.config.mod_dir(), &ctx.global_target).await;
    let nmod = index
        .iter()
        .find(|f| f.name.to_lowercase() == "northstar")
        .ok_or_else(|| anyhow!("Couldn't find Northstar on thunderstore???"))?;

    do_install(ctx, nmod, game_path).await?;

    Ok(nmod.version.clone())
}

///Install N* from the provided mod
///
///Checks cache, else downloads the latest version
async fn do_install(ctx: &Ctx, nmod: &Mod, game_path: &Path) -> Result<()> {
    let filename = format!("northstar-{}.zip", nmod.version);
    let nfile = if let Some(f) = utils::check_cache(&ctx.dirs.cache_dir().join(&filename)) {
        println!("Using cached version of Northstar@{}...", nmod.version);
        f
    } else {
        actions::download_file(&nmod.url, ctx.dirs.cache_dir().join(&filename)).await?
    };
    println!("Extracting Northstar...");
    extract(ctx, nfile, game_path)?;
    // println!("Copying Files...");
    // ctx.move_files(game_path, extracted)?;
    println!("Done!");

    Ok(())
}

// fn move_files(&ctx, game_path: &Path, extracted: PathBuf) -> Result<(), String> {
//     let nstar = extracted.join("Northstar");

//     ctx.copy_dirs(&nstar, &nstar, game_path)?;
//     println!("Northstar installed sucessfully!");
//     println!("Cleaning up...");

//     fs::remove_dir_all(nstar).map_err(|e| format!("Unable to remove temp directory {}", e))?;

//     Ok(())
// }

// ///Recurses through a directory and moves each entry to the target, keeping the directory structure
// fn copy_dirs(&ctx, root: &Path, dir: &Path, target: &Path) -> Result<(), String> {
//     for entry in (dir
//         .read_dir()
//         .map_err(|_| "Unable to read directory".to_string())?)
//     .flatten()
//     {
//         if entry.path().is_dir() {
//             ctx.copy_dirs(root, &entry.path(), target)?;
//             continue;
//         } else if let Some(p) = entry.path().parent() {
//             let target = target.join(p.strip_prefix(&root).unwrap());
//             debug!("Create dir {}", target.display());
//             fs::create_dir_all(target).map_err(|_| "Failed to create directory".to_string())?;
//         }
//         let target = target.join(entry.path().strip_prefix(root).unwrap());
//         debug!(
//             "Moving file {} to {}",
//             entry.path().display(),
//             target.display()
//         );
//         fs::rename(entry.path(), target).map_err(|e| format!("Unable to move file: {}", e))?;
//     }

//     Ok(())
// }

///Extract N* zip file to target game path
fn extract(ctx: &Ctx, zip_file: File, target: &Path) -> Result<()> {
    let mut archive = ZipArchive::new(&zip_file).context("Unable to open zip archive")?;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).unwrap();

        //skip any files that have been excluded
        if let Some(n) = f.enclosed_name() {
            if ctx.config.exclude.iter().any(|e| Path::new(e) == n) {
                continue;
            }
        } else {
            return Err(anyhow!(
                "Unable to read name of compressed file {}",
                f.name()
            ));
        }

        //This should work fine for N* because the dir structure *should* always be the same
        if f.enclosed_name().unwrap().starts_with("Northstar") {
            let out = target.join(
                f.enclosed_name()
                    .unwrap()
                    .strip_prefix("Northstar")
                    .unwrap(),
            );

            if (*f.name()).ends_with('/') {
                debug!("Create directory {}", f.name());
                fs::create_dir_all(target.join(f.name())).context("Unable to create directory")?;
                continue;
            } else if let Some(p) = out.parent() {
                fs::create_dir_all(&p).context("Unable to create directory")?;
            }

            let mut outfile = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&out)?;

            debug!("Write file {}", out.display());

            io::copy(&mut f, &mut outfile).context("Unable to write to file")?;
        }
    }

    Ok(())
}
