use std::fs;

use log::warn;

use crate::{api::model::LocalIndex, core::Ctx};
use anyhow::Result;

pub(crate) fn exclude(ctx: &Ctx, mods: Vec<String>) -> Result<()> {
    let mut local = LocalIndex::load(&ctx.local_target)?;
    for m in mods {
        if let Some(g) = local
            .linked
            .clone()
            .iter()
            .find(|(n, _)| n.trim().to_lowercase() == m.trim().to_lowercase())
        {
            for m in g.1.mods.iter() {
                fs::remove_dir_all(ctx.local_target.join(&m.name))?;
            }

            println!("Removed link to {}", m);
            local.linked.remove(g.0);
        } else {
            warn!(
                "Coudln't find link to {} in directory {}",
                m,
                ctx.local_target.display()
            );
            println!("No mod '{}' linked to current mod directory", m);
        }
    }
    Ok(())
}
