use anyhow::Result;

use crate::config::InstallType::*;
use crate::config::CONFIG;

pub fn run() -> Result<()> {
    match CONFIG.install_type() {
        Steam => {
            println!("Launching Titanfall 2 using steam...");
            let profile = CONFIG.current_profile();
            open::that_detached(format!("steam://launch/{}//-profile={profile}/", thermite::TITANFALL_STEAM_ID))?;
            println!("Done!");
        }
        Origin => {
            println!("Launching Titanfall 2 using origin...");
            open::that_detached(format!(
                "origin://LaunchGame/{}",
                thermite::TITANFALL_ORIGIN_IDS[0]
            ))?;
            println!("Done!");
        }
        Other => {
            println!("Can't launch the game for this type of installation");
        }
        _ => todo!(),
    }

    Ok(())
}
