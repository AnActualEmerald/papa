use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use thermite::prelude::find_mods;

use crate::{config::CONFIG, model::ModName};

pub fn export(file: PathBuf) -> Result<()> {
    let mods: Vec<String> = find_mods(CONFIG.install_dir()?)?
        .into_iter()
        .filter_map(|m| {
            let v = ModName::from(m);
            if v.author.to_lowercase() == "northstar" {
                None
            } else {
                Some(v.as_modstr().to_string())
            }
        })
        .collect();

    let raw = ron::to_string(&mods)?;

    if let Err(e) = std::fs::write(&file, raw) {
        eprintln!("Failed to write mod list: {e}");
        return Err(e.into());
    }

    println!("Exported mods to '{}'", file.display().bright_cyan());

    Ok(())
}
