use std::fs;

use anyhow::Result;
use owo_colors::OwoColorize;
use tracing::debug;

use crate::{config::CONFIG, model::ModName, utils::find_package_roots};

pub fn remove(mods: Vec<ModName>) -> Result<()> {
    let locals = find_package_roots(CONFIG.install_dir()?)?;

    for m in mods {
        debug!("Searching for '{m}'");
        if let Some(installed) = locals.iter().find(|v| {
            let Ok(local) = ModName::try_from(v.as_path()) else {
                return false;
            };

            debug!("Testing '{local}'");
            m.name.to_lowercase() == local.name.to_lowercase()
                && m.author.to_lowercase() == local.author.to_lowercase()
        }) {
            println!("Removing package '{}'", m.bright_cyan());
            debug!("Removing mod {installed:?}");
            fs::remove_dir_all(installed)?;
        }
    }

    Ok(())
}
