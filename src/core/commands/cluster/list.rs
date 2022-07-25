use anyhow::Result;

use crate::core::Ctx;

pub(crate) fn list(ctx: &Ctx) -> Result<()> {
    if let Some(c) = ctx.cluster.as_ref() {
        c.members.iter().for_each(|(k, v)| {
            println!("{} at {}", k, v.display());
        });
    }

    Ok(())
}
