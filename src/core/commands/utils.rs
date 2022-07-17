use std::{fs, path::Path};

use crate::{
    api::model::{LocalIndex, Mod},
    core::{actions, Ctx},
};

use anyhow::{Context, Result};
use log::{debug, trace};

pub(super) async fn do_update(
    ctx: &mut Ctx,
    outdated: &Vec<&Mod>,
    installed: &mut LocalIndex,
    target: &Path,
) -> Result<()> {
    let mut downloaded = vec![];
    for base in outdated {
        let name = &base.name;
        let url = &base.url;
        let path = ctx
            .dirs
            .cache_dir()
            .join(format!("{}_{}.zip", name, base.version));
        match actions::download_file(url, path).await {
            Ok(f) => downloaded.push(f),
            Err(e) => eprintln!("{}", e),
        }
    }

    for f in downloaded.into_iter() {
        let mut pkg = actions::install_mod(&f, target).unwrap();
        ctx.cache.clean(&pkg.package_name, &pkg.version)?;
        if let Some(i) = installed
            .mods
            .clone()
            .iter()
            .find(|e| e.package_name == pkg.package_name)
        {
            let mut inst = i.clone();
            inst.version = pkg.version;
            installed.mods.remove(i);
            //Don't know if sorting is needed here but seems like a good assumption
            inst.mods
                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            pkg.mods
                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            for (a, b) in inst.mods.iter().zip(pkg.mods.iter()) {
                trace!("a mod: {:#?} | b mod: {:#?}", a, b);
                if a.disabled() {
                    fs::remove_dir_all(&a.path).unwrap();
                    debug!(
                        "Moving mod from {} to {}",
                        b.path.display(),
                        a.path.display()
                    );
                    fs::rename(&b.path, &a.path).unwrap_or_else(|e| {
                        debug!("Unable to move sub-mod to old path");
                        debug!("{}", e);
                    });
                }
            }

            inst.mods = pkg.mods;
            installed.mods.insert(inst);
            println!("Updated {}", pkg.package_name);
        }
    }

    Ok(())
}

pub(super) fn link_dir(original: &Path, target: &Path) -> Result<()> {
    debug!("Linking dir {} to {}", original.display(), target.display());
    for e in original.read_dir()? {
        let e = e?;
        if e.path().is_dir() {
            link_dir(&e.path(), &target.join(e.file_name()))?;
            continue;
        }

        let target = target.join(e.file_name());
        if let Some(p) = target.parent() {
            if !p.exists() {
                fs::create_dir_all(p)?;
            }
        }

        debug!(
            "Create hardlink {} -> {}",
            e.path().display(),
            target.display()
        );
        fs::hard_link(e.path(), &target).context("Failed to create hard link")?;
    }
    Ok(())
}
