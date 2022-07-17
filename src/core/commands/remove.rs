use crate::{
    api::model::InstalledMod,
    core::{actions, utils, Ctx},
};

use anyhow::Result;

pub fn remove(ctx: &Ctx, mod_names: Vec<String>) -> Result<()> {
    let mut installed = utils::get_installed(ctx.config.mod_dir())?;
    let valid: Vec<InstalledMod> = mod_names
        .iter()
        .filter_map(|f| {
            installed
                .mods
                .clone()
                .iter()
                .find(|e| e.package_name.trim().to_lowercase() == f.trim().to_lowercase())
                .filter(|e| installed.mods.remove(e))
                .cloned()
        })
        .collect();

    let paths = valid.iter().flat_map(|f| f.flatten_paths()).collect();

    actions::uninstall(paths)?;
    valid
        .iter()
        .for_each(|e| println!("Removed {}", e.package_name));
    utils::save_installed(ctx.config.mod_dir(), &installed)?;
    Ok(())
}
