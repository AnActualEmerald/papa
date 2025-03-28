use std::collections::BTreeSet;

use anyhow::Result;
use owo_colors::OwoColorize;
use tracing::debug;

use crate::{config::CONFIG, model::ModName, utils::find_enabled_mods};
use thermite::{
    model::{EnabledMods, InstalledMod},
    prelude::find_mods,
};

pub fn enable(mods: BTreeSet<String>, all: bool) -> Result<()> {
    let dir = CONFIG.install_dir()?;
    debug!("Getting installed mods from {}", dir.display());
    let installed = find_mods(&dir)?
        .into_iter()
        .filter_map(|v| {
            if all {
                return Some((ModName::from(&v).to_string(), v));
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

    let mut enabled_mods = match find_enabled_mods(dir.join("..")) {
        Some(mods) => mods,
        None => EnabledMods::default_with_path(dir.join("..").join("enabledmods.json")),
    };

    debug!("Enabled mods: {:?}", enabled_mods.mods);

    let mut acted = BTreeSet::new();
    for (idx, i) in installed {
        enabled_mods.set(&i.mod_json.name, true);
        println!("Enabled {}", i.mod_json.name.bright_green());
        acted.insert(idx.clone());
    }

    let diff = mods.difference(&acted);
    for m in diff {
        println!("Couldn't find {}", m.bright_cyan());
    }

    enabled_mods.save()?;

    Ok(())
}
