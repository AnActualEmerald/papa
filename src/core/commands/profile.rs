use std::{ffi::OsString, fs};

use anyhow::{Result, anyhow};
use clap::Subcommand;
use owo_colors::OwoColorize;
use thermite::CORE_MODS;
use tracing::debug;

use crate::config::CONFIG;

#[derive(Subcommand)]
pub enum ProfileCommands {
    #[clap(alias("ls"))]
    List,
    #[clap(alias("n"))]
    New {
        #[arg(long)]
        no_core: bool,
        ///Name of the profile to create
        name: OsString,
        #[arg(long, short)]
        force: bool
    }
}

pub fn handle(command: &ProfileCommands) -> Result<()> {
    match command{
        ProfileCommands::List => list_profiles(),
        ProfileCommands::New { no_core, name, force } => new_profile(!no_core, name, *force),
    }
}

fn list_profiles() -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        println!("Please run '{}' first", "papa ns init".bright_cyan());
        return Err(anyhow!("Game path not set"));
    };

    let mut profiles = vec![];
    for candidate in dir.read_dir()?  {
        let candidate = candidate?;
        if !candidate.file_type()?.is_dir() { continue }

        let path = candidate.path();
        for child in path.read_dir()? {
            if child?.path().ends_with("enabledmods.json") {
                profiles.push(path);
                break;
            }
        }
    }

    if profiles.is_empty() {
        println!("No profiles found");
        return Ok(());
    }

    println!("Available profiles:");
    for name in  profiles.iter().filter_map(|v| {
        v.file_name().map(|os| os.to_str()).flatten()
    }) {
        println!("{:<4}{}",
        if profiles.len() > 1 && name == CONFIG.current_profile() {
            "-->"
        } else {
            ""
        }.bright_green(),
        name.bright_cyan()
    );

    }
    Ok(())
}

fn new_profile(core: bool, name: &OsString, force: bool) -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        println!("Please run '{}' first", "papa ns init".bright_cyan());
        return Err(anyhow!("Game path not set"));
    };

    let prof = dir.join(name);
    if prof.try_exists()? {
        if  force {
            fs::remove_dir_all(&prof)?;
        }else {
            println!("A folder of that name already exists, remove it first");
            return Ok(());
        }

    }
    fs::create_dir(&prof)?;

    if core {
        let dir = dir.join("R2Northstar").join("packages");
        if !dir.try_exists()? {
            debug!("Expected to find existing default profile at {}", dir.display());
        } else {
            for m in ["Northstar.Client", "Northstar.Custom", "Northstar.CustomServers"] {
                let original = dir.join(m);
                if !original.try_exists()? { debug!("Expected to find {m} at {}", original.display()); continue }

                let link = prof.join(m);

                #[cfg(windows)]
                {
                    use std::os::windows::fs::symlink_dir;
                    symlink_dir(&original, &link)?;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    symlink(&original, &link)?;
                }
                debug!("Created symlink {} -> {}", original.display(), link.display());
            }
        }
    }

    fs::write(prof.join("enabledmods.json"),
r#"{
    "Northstar.Client": true,
    "Northstar.Custom": true,
    "Northstar.CustomServers": true
}"#
    )?;

    println!("Created profile {:?}", name.bright_cyan());

    Ok(())
}