use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::TITANFALL2_STEAM_ID;

use crate::config::CONFIG;
use crate::config::InstallType::*;

pub fn run(no_profile: bool) -> Result<()> {
    match CONFIG.install_type() {
        Steam(t) => {
            println!("Launching Titanfall 2 using steam...");
            let profile = if no_profile {
                String::new()
            } else {
                println!("Using profile {}", CONFIG.current_profile().bright_cyan());
                format!("-profile={}", CONFIG.current_profile())
            };
            // open::that_detached(format!(
            //     "steam://run/{}//{profile} -northstar/",
            //     thermite::TITANFALL2_STEAM_ID
            // ))?;

            let mut child = dbg!(
                dbg!(t.to_launch_command())
                    .arg("-applaunch")
                    .arg(TITANFALL2_STEAM_ID.to_string())
                    .arg("-northstar")
            )
            // .arg(profile)
            .spawn()?;

            child.wait()?;
        }
        Origin => {
            println!("Launching Titanfall 2 using origin...");
            if CONFIG.current_profile() != "R2Northstar" {
                println!(
                    "{0} Papa doesn't support using profiles with Origin. Make sure to manually set the launch args to use your profile. {0}",
                    "!!".bright_red()
                );
            }
            open::that_detached(format!(
                "origin://LaunchGame/{}",
                thermite::TITANFALL2_ORIGIN_IDS[0]
            ))?;
        }
        Other => {
            println!(
                "Can't launch the game for this type of installation.\nIf you think this is a mistake, try running {}.",
                "papa ns init".bright_cyan()
            );
        }
        _ => todo!(),
    }

    Ok(())
}
