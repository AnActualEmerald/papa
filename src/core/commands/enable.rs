use std::collections::BTreeSet;

use anyhow::Result;
use owo_colors::OwoColorize;
use tracing::debug;

use crate::{config::CONFIG, model::ModName};
use thermite::{
    model::{EnabledMods, InstalledMod},
    prelude::{find_mods, get_enabled_mods, ThermiteError},
};

pub fn enable(mods: BTreeSet<String>) -> Result<()> {
    let dir = CONFIG.install_dir();
    debug!("Getting installed mods from {}", dir.display());
    let installed = find_mods(dir)?
        .into_iter()
        .filter_map(|v| v.ok())
        .filter_map(|v| {
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

            if let Some(m) = res {
                Some((m, v))
            } else {
                None
            }
        })
        .collect::<Vec<(&String, InstalledMod)>>();

    let mut enabled_mods = match get_enabled_mods(dir.join("..")) {
        Ok(mods) => mods,
        Err(ThermiteError::MissingFile(path)) => EnabledMods::default_with_path(*path),
        Err(e) => return Err(e.into()),
    };

    debug!("Enabled mods: {:?}", enabled_mods.mods);

    let mut acted = BTreeSet::new();
    for (idx, i) in installed {
        enabled_mods.set(&i.mod_json.name, true);
        println!("Enabled {}", format!("{}", i.mod_json.name).bright_green());
        acted.insert(idx.clone());
    }

    let diff = mods.difference(&acted);
    for m in diff {
        println!("Couldn't find {}", m.bright_cyan());
    }

    Ok(())
}
