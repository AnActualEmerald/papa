use crate::core::{config, Ctx};
use anyhow::Result;

pub fn update_config(ctx: &mut Ctx, mods_dir: Option<String>, cache: Option<bool>) -> Result<()> {
    if let Some(dir) = mods_dir {
        ctx.config.set_dir(&dir);
        println!("Set install directory to {}", dir);
    }

    if let Some(cache) = cache {
        ctx.config.set_cache(&cache);
        if cache {
            println!("Turned caching on");
        } else {
            println!("Turned caching off");
        }
    }

    config::save_config(ctx.dirs.config_dir(), &ctx.config)?;
    Ok(())
}
