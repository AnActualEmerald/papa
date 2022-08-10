use std::fs;

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::{
    api::model::LocalIndex,
    core::{actions, utils, Ctx},
};

pub async fn install(
    ctx: &mut Ctx,
    mod_names: Vec<String>,
    yes: bool,
    force: bool,
    global: bool,
) -> Result<()> {
    let target = if global {
        ctx.dirs.data_local_dir()
    } else {
        ctx.config.mod_dir()
    };

    //Create the target dir if it doesn't exist
    if !target.exists() {
        log::trace!("Creating dir {}", target.display());
        fs::create_dir_all(target)?;
    }

    let index = utils::update_index(target, &ctx.global_target).await;
    let mut installed = LocalIndex::load_or_create(target);
    let mut valid = vec![];
    for name in mod_names {
        let re = Regex::new(r"(.+)@?(v?\d.\d.\d)?").unwrap();

        if !re.is_match(&name) {
            println!("{} should be in 'ModName@1.2.3' format", name);
            continue;
        }

        let parts = re.captures(&name).unwrap();

        let base = index
            .iter()
            .find(|e| e.name.to_lowercase() == parts[1].to_lowercase())
            .ok_or_else(|| anyhow!("No such package {}", &parts[1]))?;

        if base.installed && !force {
            println!(
                "Package \x1b[36m{}\x1b[0m version \x1b[36m{}\x1b[0m already installed",
                base.name, base.version
            );
            continue;
        }

        utils::resolve_deps(&mut valid, base, &installed.mods, &index)?;
        valid.push(base);
    }

    //Gaurd against an empty list (maybe all the mods are already installed?)
    if valid.is_empty() {
        return Ok(());
    }

    let size: i64 = valid.iter().map(|f| f.file_size).sum();
    println!("Installing:\n");

    print!("\t");
    valid.iter().enumerate().for_each(|(i, f)| {
        if i > 0 && i % 5 == 0 {
            println!("\n");
            print!("\t");
        }
        print!(" \x1b[36m{}@{}\x1b[0m ", f.name, f.version);
    });
    println!("\n");

    let msg = format!(
        "Will download ~{:.2} MIB (compressed), okay? [Y/n]: ",
        size as f64 / 1_048_576f64
    );

    if !yes {
        if let Ok(line) = ctx.rl.readline(&msg) {
            if line.to_lowercase() == "n" {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    let mut downloaded = vec![];
    for base in valid {
        let name = &base.name;
        let path = ctx
            .dirs
            .cache_dir()
            .join(format!("{}_{}.zip", name, base.version));

        //would love to use this in the same if as the let but it's unstable so...
        if ctx.config.cache() {
            if let Some(f) = ctx.cache.check(&path) {
                println!("Using cached version of {}", name);
                downloaded.push(f);
                continue;
            }
        }
        match actions::download_file(&base.url, path).await {
            Ok(f) => downloaded.push(f),
            Err(e) => eprintln!("{}", e),
        }
    }

    println!(
        "Extracting mod{} to {}",
        if downloaded.len() > 1 { "s" } else { "" },
        target.display()
    );
    for e in downloaded
        .iter()
        .map(|f| -> Result<()> {
            let pkg = actions::install_mod(f, target)?;
            installed.mods.insert(pkg.package_name.clone(), pkg.clone());
            ctx.cache.clean(&pkg.package_name, &pkg.version)?;
            println!("Installed {}!", pkg.package_name);
            Ok(())
        })
        .filter(|f| f.is_err())
    {
        println!("Encountered errors while installing mods:");
        println!("{:#?}", e.unwrap_err());
    }
    // utils::save_installed(target, &installed)?;
    Ok(())
}

pub async fn install_from_url(ctx: &Ctx, url: String) -> Result<()> {
    let file_name = url
        .as_str()
        .replace(':', "")
        .split('/')
        .collect::<Vec<&str>>()
        .join("");
    println!("Downloading to {}", file_name);
    let path = ctx.dirs.cache_dir().join(file_name);
    match actions::download_file(url.to_string().as_str(), path.clone()).await {
        Ok(f) => {
            let _pkg = actions::install_mod(&f, ctx.config.mod_dir()).unwrap();
            utils::remove_file(&path)?;
            println!("Installed {}", url);
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
