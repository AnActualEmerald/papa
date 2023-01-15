use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::prelude::*;
use tracing::{debug, instrument, trace};

use crate::{config::CONFIG, model::ModName};

#[instrument]
pub fn list(global: bool, all: bool) -> Result<()> {
    if global {
        todo!();
    }
    let mods = find_mods(CONFIG.install_dir())?;
    debug!("Found {} mods", mods.len());
    trace!("{:?}", mods);
    let mut grouped_mods: BTreeMap<ModName, BTreeSet<String>> = BTreeMap::new();
    for m in mods {
        let local_name = m.mod_json.name.clone();
        let mn = m.into();
        if let Some(group) = grouped_mods.get_mut(&mn) {
            debug!("Adding submod {} to group {}", local_name, mn);
            group.insert(local_name);
        } else {
            debug!("Adding group {} for sdubmod {}", mn, local_name);
            let group = BTreeSet::from([local_name]);
            grouped_mods.insert(mn, group);
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
    Ok(())
}
