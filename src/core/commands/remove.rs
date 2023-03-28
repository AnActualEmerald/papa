use std::fs;

use anyhow::Result;
use tracing::debug;

use crate::{config::CONFIG, model::ModName};
use thermite::prelude::*;

pub fn remove(mods: Vec<ModName>) -> Result<()> {
    let locals = find_mods(CONFIG.install_dir())?;

    for m in mods {
        debug!("Searching for '{m}'");
        if let Some(Ok(installed)) = locals.iter().filter(|v| v.is_ok()).find(|v| {
            let local = ModName::from(v.as_ref().unwrap());
            debug!("Testing '{local}'");
            m.name.to_lowercase() == local.name.to_lowercase()
                && m.author.to_lowercase() == local.author.to_lowercase()
        }) {
            println!("Removing package '{}'", m);
            fs::remove_dir_all(&installed.path)?;
        }
    }

    Ok(())
}
