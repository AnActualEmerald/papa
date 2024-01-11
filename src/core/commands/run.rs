use anyhow::Result;
use owo_colors::OwoColorize;

use crate::config::InstallType::*;
use crate::config::CONFIG;

pub fn run(no_profile: bool) -> Result<()> {
    match CONFIG.install_type() {
        Steam => {
            println!("Launching Titanfall 2 using steam...");
            let profile = if no_profile {
                String::new()
            } else {
                format!("//-profile={}/", CONFIG.current_profile())
            };
            open::that_detached(format!(
                "steam://launch/{}{profile}",
                thermite::TITANFALL_STEAM_ID
            ))?;
            println!("Done!");
        }
        Origin => {
            println!("Launching Titanfall 2 using origin...");
            if CONFIG.current_profile() != "R2Northstar" {
                println!("{}Papa doesn't support using profiles with Origin. Make sure to manually set the launch args to use your profile.", "!! ".bright_red());
            }
            open::that_detached(format!(
                "origin://LaunchGame/{}",
                thermite::TITANFALL_ORIGIN_IDS[0]
            ))?;
            println!("Done!");
        }
        Other => {
            println!("Can't launch the game for this type of installation.\nIf you think this is a mistake, try running {}.", "papa ns init".bright_cyan());
        }
        _ => todo!(),
    }

    Ok(())
}
