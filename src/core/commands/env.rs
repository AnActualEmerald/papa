use crate::config::{CONFIG, DIRS};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn env() -> Result<()> {
    println!("Current config:\n");
    println!(
        "Mod installation directory: {}",
        CONFIG.install_dir().display().bright_cyan()
    );
    if let Some(dir) = CONFIG.game_dir() {
        println!("Game install directory: {}", dir.display().bright_cyan());
    }
    println!(
        "Cache directory: {}",
        DIRS.cache_dir().display().bright_cyan()
    );
    Ok(())
}
