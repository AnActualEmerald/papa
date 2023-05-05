use anyhow::Result;
use std::path::PathBuf;
use thermite::prelude::find_mods;

use crate::{config::CONFIG, model::ModName};

pub fn export(file: PathBuf) -> Result<()> {
    let mods: Vec<String> = find_mods(CONFIG.install_dir())?
        .into_iter()
        .filter_map(|m| m.map(|v| ModName::from(v).to_string()).ok())
        .collect();

    let raw = ron::to_string(&mods)?;

    if let Err(e) = std::fs::write(&file, raw) {
        eprintln!("Failed to write mod list: {e}");
        return Err(e.into())
    }
    
    Ok(())
}
