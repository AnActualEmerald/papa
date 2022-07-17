use crate::core::{utils, Ctx};
use anyhow::Result;

pub(crate) fn disable(ctx: &Ctx, mods: Vec<String>) -> Result<()> {
    let mut installed = utils::get_installed(ctx.config.mod_dir())?;
    for m in mods {
        let m = m.to_lowercase();
        for i in installed.mods.clone().iter() {
            installed.mods.remove(i);
            let mut i = i.clone();
            if i.package_name.to_lowercase() == m {
                for sub in i.mods.iter_mut() {
                    utils::disable_mod(sub)?;
                }
                println!("Disabled {}", m);
            } else {
                for e in i.mods.iter_mut() {
                    if e.name.to_lowercase() == m {
                        utils::disable_mod(e)?;
                        println!("Disabled {}", m);
                    }
                }
            }
            installed.mods.insert(i);
        }
    }
    utils::save_installed(ctx.config.mod_dir(), &installed)?;

    Ok(())
}
