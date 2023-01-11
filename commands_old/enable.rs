use crate::{
    api::model::LocalIndex,
    core::{utils, Ctx},
};
use anyhow::Result;

pub(crate) fn enable(ctx: &Ctx, mods: Vec<String>) -> Result<()> {
    let mut installed = LocalIndex::load(ctx.config.mod_dir())?;
    for m in mods {
        let m = m.to_lowercase();
        for (_i, p) in installed.mods.iter_mut() {
            if p.package_name.to_lowercase() == m {
                for sub in p.mods.iter_mut() {
                    utils::enable_mod(sub, ctx.config.mod_dir())?;
                }
                println!("Enabled {}", p.package_name);
            } else {
                for e in p.mods.iter_mut() {
                    if e.name.to_lowercase() == m {
                        utils::enable_mod(e, ctx.config.mod_dir())?;
                        println!("Enabled {}", p.package_name);
                    }
                }
            }
        }
    }

    Ok(())
}
