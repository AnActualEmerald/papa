use crate::{
    api::model::{InstalledMod, LocalIndex},
    core::{actions, Ctx},
};

use anyhow::Result;

pub fn remove(ctx: &Ctx, mod_names: Vec<String>) -> Result<()> {
    let installed = LocalIndex::load(ctx.config.mod_dir())?;
    let valid: Vec<InstalledMod> = installed
        .mods
        .iter()
        .filter_map(|(n, v)| {
            if mod_names.contains(n) {
                Some(v.clone())
            } else {
                None
            }
        })
        .collect();
    let paths = valid.iter().flat_map(|f| f.flatten_paths()).collect();

    actions::uninstall(paths)?;
    valid
        .iter()
        .for_each(|e| println!("Removed {}", e.package_name));
    Ok(())
}
