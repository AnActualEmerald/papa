use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use thermite::prelude::*;
use tracing::{debug, trace};

use crate::{config::CONFIG, model::ModName};

pub fn list(global: bool, _all: bool) -> Result<()> {
    if global {
        todo!();
    }
    let mods = find_mods(CONFIG.install_dir()).context("Error finding mods")?;
    debug!("Found {} mods", mods.len());
    trace!("{:?}", mods);
    let enabled_mods = get_enabled_mods(CONFIG.install_dir().join("..")).ok();

    let mut grouped_mods: BTreeMap<ModName, BTreeSet<String>> = BTreeMap::new();
    let mut disabled: BTreeMap<ModName, BTreeSet<String>> = BTreeMap::new();
    for m in mods {
        let local_name = m.mod_json.name.clone();

        let mn = m.into();
        let process_mod = |mod_group: &mut BTreeMap<ModName, BTreeSet<String>>| {
            if let Some(group) = mod_group.get_mut(&mn) {
                debug!("Adding submod {} to group {}", local_name, mn);
                group.insert(local_name.clone());
            } else {
                debug!("Adding group {} for sdubmod {}", mn, local_name);
                let group = BTreeSet::from([local_name.clone()]);
                mod_group.insert(mn, group);
            }
        };

        if let Some(em) = enabled_mods.as_ref() {
            if em.is_enabled(&local_name) {
                process_mod(&mut grouped_mods);
            } else {
                process_mod(&mut disabled);
            }
        } else {
            process_mod(&mut grouped_mods);
        }
    }

    println!("Installed mods: ");
    for (group, names) in grouped_mods {
        if names.len() == 1 {
            println!("-  {}", group.bright_blue().bold());
        } else {
            println!("-  {}:", group.bright_blue().bold());
            for n in names {
                println!("    {}", n.bright_cyan().bold());
            }
        }
    }

    if !disabled.is_empty() {
        println!("Disabled mods: ");
        for (group, names) in disabled {
            println!("-  {}:", group.bright_red().bold());
            for n in names {
                println!("    {}", n.magenta().bold());
            }
        }
    }
    Ok(())
}
