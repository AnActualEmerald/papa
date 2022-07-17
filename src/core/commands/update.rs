use std::fs;

use log::debug;

use crate::{core::{Ctx, utils, commands::utils::{do_update, link_dir}}, api::{self, model}};

use anyhow::{Result, anyhow};



pub async fn update(ctx: &mut Ctx, yes: bool) -> Result<()> {
    let local_target = ctx.local_target.clone();
    let global_target = ctx.global_target.clone();
    print!("Updating package index...");
    let index = &api::get_package_index().await?;
    println!(" Done!");
    let mut installed = utils::get_installed(ctx.config.mod_dir())?;
    let mut global = utils::get_installed(ctx.dirs.data_local_dir())?;
    let outdated: Vec<&model::Mod> = index
        .iter()
        .filter(|e| {
            installed.mods.iter().any(|i| {
                i.package_name.trim() == e.name.trim() && i.version.trim() != e.version.trim()
            })
        })
        .collect();
    let glob_outdated: Vec<&model::Mod> = index
        .iter()
        .filter(|e| {
            global.mods.iter().any(|i| {
                i.package_name.trim() == e.name.trim() && i.version.trim() != e.version.trim()
            })
        })
        .collect();

    if outdated.is_empty() && glob_outdated.is_empty() {
        println!("Already up to date!");
    } else {
        let size: i64 = outdated.iter().map(|f| f.file_size).sum::<i64>()
            + glob_outdated.iter().map(|f| f.file_size).sum::<i64>();

        println!("Updating: \n");
        print!("\t");
        outdated
            .iter()
            .chain(glob_outdated.iter())
            .enumerate()
            .for_each(|(i, f)| {
                if i > 0 && i % 5 == 0 {
                    println!("\n");
                    print!("\t");
                }
                print!(" \x1b[36m{}@{}\x1b[0m ", f.name, f.version);
            });

        println!("\n");
        if !yes {
            if let Ok(line) = ctx.rl.readline(&format!(
                    "Will download ~{:.2} MB (compressed), okay? (This will overwrite any changes made to mod files) [Y/n]: ",
                    size as f64 / 1_048_576f64
                )) {
                    if line.to_lowercase() == "n" {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
        }

        do_update(ctx, &outdated, &mut installed, &local_target).await?;
        do_update(ctx, &glob_outdated, &mut global, &global_target).await?;

        //check if any link mods are being updated
        let relink = installed
            .linked
            .clone()
            .into_iter()
            .filter(|e| glob_outdated.iter().any(|f| e.package_name == f.name));

        for r in relink {
            debug!("Relinking mod {}", r.package_name);
            //Update the submod links
            for p in r.mods.iter() {
                //delete the current link first
                let target = ctx.local_target.join(&p.name);
                if target.exists() {
                    fs::remove_dir_all(&target)?;
                }
                link_dir(&p.path, &target)?;
            }

            //replace the linked mod with the new mod info
            let n = global
                .mods
                .iter()
                .find(|e| e.package_name == r.package_name)
                .ok_or_else(|| anyhow!("Unable to find linked mod in global index"))?;
            if !installed.linked.remove(&r) {
                debug!("Didn't find old linked mod to remove");
            }
            if !installed.linked.insert(n.clone()) {
                debug!("Failed to add updated mod to linked set");
            }
        }
        utils::save_installed(ctx.config.mod_dir(), &installed)?;
        utils::save_installed(ctx.dirs.data_local_dir(), &global)?;
    }
    //Would be cool to do an && on these let statements
    if let Some(current) = &ctx.config.nstar_version {
        if let Some(nmod) = index.iter().find(|e| e.name.to_lowercase() == "northstar") {
            if *current != nmod.version {
                println!(
                    "An update for Northstar is available! \x1b[93m{}\x1b[0m -> \x1b[93m{}\x1b[0m",
                    current, nmod.version
                );
                println!("Run \"\x1b[96mpapa northstar update\x1b[0m\" to install it!");
            }
        }
    }
    Ok(())

}
