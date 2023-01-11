use crate::core::{utils, Ctx};
use anyhow::Result;

pub fn clear(ctx: &Ctx, full: bool) -> Result<()> {
    if full {
        println!("Clearing cache files...");
    } else {
        println!("Clearing cached packages...");
    }
    utils::clear_cache(ctx.dirs.cache_dir(), full)?;
    println!("Done!");

    Ok(())
}
