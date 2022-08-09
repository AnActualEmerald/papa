use std::fs;

use log::debug;

use crate::{
    api::{
        self,
        model::{self, LocalIndex},
    },
    core::{
        commands::utils::{do_update, link_dir},
        config::ManageMode,
        utils, Ctx,
    },
};

use anyhow::{anyhow, Result};

pub async fn update(ctx: &mut Ctx, yes: bool) -> Result<()> {
    match ctx.config.mode {
        ManageMode::Client => client_update(ctx, yes).await,
        ManageMode::Server => cluster_update(ctx, yes).await,
    }
}

async fn client_update(ctx: &mut Ctx, yes: bool) -> Result<()> {
    let local_target = ctx.local_target.clone();
    let global_target = ctx.global_target.clone();
    print!("Updating package index...");
    let index = &api::get_package_index().await?;
    println!(" Done!");
    let mut installed = LocalIndex::load(ctx.config.mod_dir()).ok();
    let mut global = LocalIndex::load_or_create(ctx.dirs.data_local_dir());
    let outdated: Vec<&model::Mod> = if let Some(installed) = &installed {
        index
            .iter()
            .filter(|e| {
                installed
                    .mods
                    .iter()
                    .any(|(n, i)| n.trim() == e.name.trim() && i.version.trim() != e.version.trim())
            })
            .collect()
    } else {
        vec![]
    };
    let glob_outdated: Vec<&model::Mod> = index
        .iter()
        .filter(|e| {
            global
                .mods
                .iter()
                .any(|(n, i)| n.trim() == e.name.trim() && i.version.trim() != e.version.trim())
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

        if let Some(installed) = installed.as_mut() {
            do_update(ctx, &outdated, installed, &local_target).await?;
        }
        do_update(ctx, &glob_outdated, &mut global, &global_target).await?;

        if let Some(installed) = installed.as_mut() {
            //check if any link mods are being updated
            let relink = installed
                .linked
                .clone()
                .into_iter()
                .filter(|(e, _)| glob_outdated.iter().any(|f| e == &f.name));

            for (name, r) in relink {
                debug!("Relinking mod {}", name);
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
                let (n, m) = global
                    .mods
                    .iter()
                    .find(|(e, _)| *e == &r.package_name)
                    .ok_or_else(|| anyhow!("Unable to find linked mod in global index"))?;
                //Insert or update the mod in the linked set
                installed
                    .linked
                    .entry(n.to_owned())
                    .and_modify(|v| *v = m.clone())
                    .or_insert_with(|| m.clone());
            }
        }
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

async fn cluster_update(ctx: &mut Ctx, yes: bool) -> Result<()> {
    if let Some(c) = ctx.cluster.clone() {
        println!(
            "Updating server cluster{}...",
            if c.name.is_some() {
                format!(" {}", c.name.as_ref().unwrap())
            } else {
                "".to_string()
            }
        );

        let index = utils::update_index(&ctx.local_target, &ctx.global_target).await;
        //update global mods first
        let mut to_relink = vec![];
        let mut global_installed = LocalIndex::load(&ctx.global_target)?;
        for g in index.iter().filter(|m| m.global) {
            if let Some(_o) = global_installed
                .mods
                .iter()
                .find(|(n, e)| **n == g.name && e.version != g.version)
            {
                to_relink.push((&g.name, g));
            }
        }

        if !to_relink.is_empty() {
            let size: i64 = to_relink.iter().map(|f| f.1.file_size).sum::<i64>();
            println!("Updating global mod(s): \n");
            print!("\t");
            to_relink.iter().enumerate().for_each(|(i, f)| {
                //Only display 5 mods per row
                if i > 0 && i % 5 == 0 {
                    println!("\n");
                    print!("\t");
                }
                print!(" \x1b[36m{}@{}\x1b[0m ", f.0, f.1.version);
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
            do_update(
                ctx,
                &to_relink.iter().map(|(_, m)| *m).collect(),
                &mut global_installed,
                &ctx.global_target.clone(),
            )
            .await?;
        }

        for s in c.members.iter() {
            let _name = s.0;
            let path = s.1;
            let _installed = LocalIndex::load(path);
        }
    }
    Ok(())
}
