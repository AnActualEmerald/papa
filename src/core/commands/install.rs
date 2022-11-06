use std::fs;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use thermite::{update_index, LocalIndex, ModVersion};

use crate::core::{utils, Ctx};

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

    let mut installed = LocalIndex::load_or_create(target.join(".papa.ron"));
    let index = update_index(Some(installed.path()), None).await;
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
                base.name, base.latest
            );
            continue;
        }

        let deps = thermite::core::resolve_deps(
            &base.versions[&base.latest].deps,
            &index
                .iter()
                .map(|e| e.versions[&e.latest].clone())
                .collect::<Vec<ModVersion>>(),
        )
        .unwrap();

        valid.push(base.versions[&base.latest].clone());
        // This only covers a single layer of dependencies
        for dep in deps {
            if !dep.installed {
                valid.push(dep.clone());
            }
        }
    }

    //Gaurd against an empty list (maybe all the mods are already installed?)
    if valid.is_empty() {
        return Ok(());
    }

    let size: u64 = valid.iter().map(|f| f.file_size).sum();
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

    let mut ctx = thermite::core::Ctx::new(ctx.dirs.clone());

    thermite::install(&mut ctx, &mut installed, valid.as_slice(), force, true).await?;
    for m in valid {
        println!("Installed {}!", m.name);
    }
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
    match thermite::core::actions::download_file(url.to_string().as_str(), path.clone()).await {
        Ok(f) => {
            let _pkg = thermite::core::actions::install_mod(&f, ctx.config.mod_dir()).unwrap();
            utils::remove_file(&path)?;
            println!("Installed {}", url);
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
