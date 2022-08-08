use crate::{
    api::model::LocalIndex,
    core::{commands::utils::link_dir, Ctx},
};
use anyhow::{Context, Result};

pub(crate) fn include(ctx: &Ctx, mods: Vec<String>, force: bool) -> Result<()> {
    let mut local = LocalIndex::load(&ctx.local_target)?;
    let global = LocalIndex::load(&ctx.global_target)?;
    for m in mods.iter() {
        if let Some(g) = global
            .mods
            .iter()
            .find(|(n, _)| n.trim().to_lowercase() == m.trim().to_lowercase())
        {
            if !force && local.linked.contains_key(g.0) {
                println!("Mod '{}' already linked", m);
                continue;
            }
            for m in g.1.mods.iter() {
                link_dir(&m.path, &ctx.local_target.join(&m.name)).context(format!(
                    "Unable to create link to {}... Does a file by that name already exist?",
                    ctx.local_target.join(&m.name).display()
                ))?;
            }

            println!("Linked {}!", m);
            local.linked.insert(g.0.clone(), g.1.clone());
        } else {
            println!("No mod '{}' globally installed", m);
        }
    }

    Ok(())
}
