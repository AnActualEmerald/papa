use crate::{
    api::model::LocalIndex,
    core::{utils, Ctx},
};
use anyhow::Result;

pub(crate) fn disable(ctx: &Ctx, mods: Vec<String>) -> Result<()> {
    let mut installed = LocalIndex::load(ctx.config.mod_dir())?;
    for m in mods {
        let m = m.to_lowercase();
        for (n, i) in installed.mods.iter_mut() {
            if n.to_lowercase() == m {
                for sub in i.mods.iter_mut() {
                    utils::disable_mod(ctx, sub)?;
                }
                println!("Disabled {}", i.package_name);
            } else {
                for e in i.mods.iter_mut() {
                    if e.name.to_lowercase() == m {
                        utils::disable_mod(ctx, e)?;
                        println!("Disabled {}", m);
                    }
                }
            }
        }
    }

    Ok(())
}
