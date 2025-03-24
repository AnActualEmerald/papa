use std::process::Stdio;
use std::time::Duration;

use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;
use thermite::TITANFALL2_STEAM_ID;

use crate::config::CONFIG;
use crate::config::InstallType::*;

#[derive(Args)]
pub struct RunOptions {
    ///Don't specify a profile to use
    ///
    ///Otherwise, use the current profile
    #[arg(short = 'P', long = "no-profile")]
    no_profile: bool,

    ///Try to launch with `-vanilla` instead of `-northstar`
    #[arg(long = "vanilla")]
    vanilla: bool,

    ///Extra args to pass to the game
    extra: Vec<String>,
}

pub fn run(
    RunOptions {
        no_profile,
        vanilla,
        extra,
    }: RunOptions,
) -> Result<()> {
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
            //
            let mode = if vanilla { "-vanilla" } else { "-northstar" };

            t.to_launch_command()
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .arg("-applaunch")
                .arg(TITANFALL2_STEAM_ID.to_string())
                .arg(mode)
                .arg(profile)
                .args(extra)
                .spawn()?;
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
