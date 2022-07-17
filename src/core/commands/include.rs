use crate::core::{commands::utils::link_dir, utils, Ctx};
use anyhow::{Context, Result};

pub(crate) fn include(ctx: &Ctx, mods: Vec<String>, force: bool) -> Result<()> {
    let mut local = utils::get_installed(&ctx.local_target)?;
    let global = utils::get_installed(&ctx.global_target)?;
    for m in mods.iter() {
        if let Some(g) = global
            .mods
            .iter()
            .find(|e| e.package_name.trim().to_lowercase() == m.trim().to_lowercase())
        {
            if !force && local.linked.contains(g) {
                println!("Mod '{}' already linked", m);
                continue;
            }
            for m in g.mods.iter() {
                link_dir(&m.path, &ctx.local_target.join(&m.name)).context(format!(
                    "Unable to create link to {}... Does a file by that name already exist?",
                    ctx.local_target.join(&m.name).display()
                ))?;
            }

            println!("Linked {}!", m);
            local.linked.insert(g.clone());
        } else {
            println!("No mod '{}' globally installed", m);
        }
    }

    utils::save_installed(&ctx.local_target, &local)?;

    Ok(())
}
