use std::fs;

use log::warn;

use crate::core::{utils, Ctx};
use anyhow::Result;

pub(crate) fn exclude(ctx: &Ctx, mods: Vec<String>) -> Result<()> {
    let mut local = utils::get_installed(&ctx.local_target)?;
    for m in mods {
        if let Some(g) = local
            .linked
            .clone()
            .iter()
            .find(|e| e.package_name.trim().to_lowercase() == m.trim().to_lowercase())
        {
            for m in g.mods.iter() {
                fs::remove_dir_all(ctx.local_target.join(&m.name))?;
            }

            println!("Removed link to {}", m);
            local.linked.remove(g);
        } else {
            warn!(
                "Coudln't find link to {} in directory {}",
                m,
                ctx.local_target.display()
            );
            println!("No mod '{}' linked to current mod directory", m);
        }
    }

    utils::save_installed(&ctx.local_target, &local)?;

    Ok(())
}
