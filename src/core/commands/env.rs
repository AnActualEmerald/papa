use crate::config::{CONFIG, DIRS};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn env() -> Result<()> {
    println!("Current config:\n");
    println!(
        "Mod installation directory: {}",
        CONFIG
            .install_dir()
            .map(|v| v.display().to_string())
            .unwrap_or_else(|_| "[none]".into())
            .bright_cyan()
    );
    println!("Install type: {}", CONFIG.install_type().bright_cyan());
    if let Some(dir) = CONFIG.game_dir() {
        println!("Game install directory: {}", dir.display().bright_cyan());
    }
    println!(
        "Cache directory: {}",
        DIRS.cache_dir().display().bright_cyan()
    );

    if let Some(path) = &CONFIG.config_path {
        println!("\nConfig file: {}", path.display().bright_cyan());
    }

    Ok(())
}
