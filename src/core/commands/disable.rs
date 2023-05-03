use std::collections::BTreeSet;

use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::{
    model::{EnabledMods, InstalledMod},
    prelude::{find_mods, get_enabled_mods, ThermiteError},
    CORE_MODS,
};
use tracing::debug;

use crate::{config::CONFIG, get_answer, model::ModName, traits::Answer};

pub fn disable(mods: BTreeSet<String>, all: bool, force: bool) -> Result<()> {
    for m in mods.iter() {
        if force { break; }
        if CORE_MODS.contains(&m.to_lowercase().as_str()) {
            println!(
                "Disabling Northstar core mods can break things, are you sure you want to do this?"
            );
            let ans = get_answer!(false, "[y/N]: ")?;
            if ans.is_yes() {
                println!("Okay, hope you know what you're doing!");
            } else {
                return Ok(());
            }
        }
    }

    let dir = CONFIG.install_dir();
    debug!("Getting installed mods from {}", dir.display());
    let installed = find_mods(dir)?
        .into_iter()
        .filter_map(|v| v.ok())
        .filter_map(|v| {
            if all {
                let name = ModName::from(&v);
                debug!("Checking {name}");
                if name.author.to_lowercase() == "northstar" && !force {
                    debug!("Skipping mod {name} when disabling all");
                    return None;
                } else {
                    return Some((name.to_string(), v));
                }
            }

            debug!("Checking if {} should be enabled", ModName::from(&v));
            let res = mods.iter().find(|m| {
                if let Ok(mn) = TryInto::<ModName>::try_into(m.as_str()) {
                    (mn.author.to_lowercase() == v.author.to_lowercase()
                        && mn.name.to_lowercase() == v.manifest.name.to_lowercase())
                        || m.to_lowercase() == v.mod_json.name.to_lowercase()
                } else {
                    m.to_lowercase() == v.mod_json.name.to_lowercase()
                }
            });

            res.map(|m| (m.clone(), v))
        })
        .collect::<Vec<(String, InstalledMod)>>();

    let mut enabled_mods = match get_enabled_mods(dir.join("..")) {
        Ok(mods) => mods,
        Err(ThermiteError::MissingFile(path)) => EnabledMods::default_with_path(*path),
        Err(e) => return Err(e.into()),
    };

    debug!("Enabled mods: {:?}", enabled_mods.mods);

    let mut acted = BTreeSet::new();
    for (idx, i) in installed {
        enabled_mods.set(&i.mod_json.name, false);
        println!("Disabled {}", i.mod_json.name.bright_red());
        acted.insert(idx.clone());
    }

    for m in mods.difference(&acted) {
        println!("Couldn't find {}", m.bright_cyan());
    }

    Ok(())
}
