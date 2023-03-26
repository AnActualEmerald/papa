use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::{
    model::{EnabledMods, InstalledMod},
    prelude::{find_mods, get_enabled_mods, ThermiteError},
    CORE_MODS,
};
use tracing::debug;

use crate::{config::CONFIG, get_answer, model::ModName, traits::Answer};

pub fn disable(mut mods: Vec<String>) -> Result<()> {
    for m in mods.iter() {
        if CORE_MODS.contains(&m.as_str()) {
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
        .filter(|v| {
            debug!("Checking if {} should be disabled", ModName::from(v));
            let res = mods.iter().enumerate().find_map(|(i, m)| {
                if let Ok(mn) = TryInto::<ModName>::try_into(m.as_str()) {
                    if (mn.author.to_lowercase() == v.author.to_lowercase()
                        && mn.name.to_lowercase() == v.manifest.name.to_lowercase())
                        || m.to_lowercase() == v.mod_json.name.to_lowercase()
                    {
                        Some(i)
                    } else {
                        None
                    }
                } else if m.to_lowercase() == v.mod_json.name.to_lowercase() {
                    Some(i)
                } else {
                    None
                }
            });

            if let Some(i) = res {
                debug!("Yes");
                mods.swap_remove(i);
                true
            } else {
                debug!("No");
                false
            }
        })
        .collect::<Vec<InstalledMod>>();

    let mut enabled_mods = match get_enabled_mods(dir.join("..")) {
        Ok(mods) => mods,
        Err(ThermiteError::MissingFile(path)) => EnabledMods::default_with_path(*path),
        Err(e) => return Err(e.into()),
    };

    debug!("Enabled mods: {:?}", enabled_mods.mods);

    for i in installed {
        if CORE_MODS.contains(&i.mod_json.name.as_str()) {
            match i.mod_json.name.as_str() {
                "Northstar.Client" => enabled_mods.client = false,
                "Northstar.Custom" => enabled_mods.custom = false,
                "Northstar.CustomServers" => enabled_mods.servers = false,
                _ => unimplemented!(),
            }

            println!("Disabled {}", format!("{}", i.mod_json.name).bright_red());

            continue;
        }
        enabled_mods.mods.insert(i.mod_json.name.clone(), false);
        println!(
            "Disabled {}",
            format!("{}.{}", i.author, i.mod_json.name).bright_red()
        );
    }

    if !mods.is_empty() {
        for m in mods {
            println!("Couldn't find {}", m.bright_cyan());
        }
    }

    Ok(())
}