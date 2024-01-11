use std::{ffi::OsString, fs};

use anyhow::{anyhow, Result};
use clap::Subcommand;
use copy_dir::copy_dir;
use owo_colors::OwoColorize;

use crate::{config::{CONFIG, write_config}, utils::init_msg};

#[derive(Subcommand)]
pub enum ProfileCommands {
    #[clap(alias = "s", alias = "choose", alias = "activate")]
    ///Select a profile
    Select {
        ///Name of the profile to select
        name: OsString,
    },
    ///Ignore a directory, preventing it from displayed as a profile
    Ignore {
        name: String,
    },
    ///Un-ignore a directory, allowing it to be displayed as a profile
    Unignore {
        name: String,
    },
    #[clap(alias("ls"))]
    ///List profiles
    List,
    ///Create an empty profile
    #[clap(alias("n"))]
    New {
        ///Name of the profile to create
        name: OsString,
        #[arg(long, short)]
        force: bool,
    },

    #[clap(alias = "dupe", alias = "cp", alias = "copy")]
    ///Clone an existing profile
    Clone {
        source: String,
        new: Option<String>,
        #[arg(long, short)]
        force: bool,
    },
}

pub fn handle(command: &ProfileCommands) -> Result<()> {
    match command {
        ProfileCommands::List => list_profiles(),
        ProfileCommands::New { name, force } => new_profile(name, *force),
        ProfileCommands::Clone { source, new, force } => clone_profile(source, new, *force),
        ProfileCommands::Select { name } => activate_profile(name),
        ProfileCommands::Ignore { name } => {
            let mut cfg = CONFIG.clone();
            cfg.add_ignored(name);
            write_config(&cfg)?;
            println!("Added {} to ignore list", name.bright_cyan());
            Ok(())
        },
        ProfileCommands::Unignore { name } => {
            let mut cfg = CONFIG.clone();
            cfg.remove_ignored(name);
            write_config(&cfg)?;
            println!("Removed {} from ignore list", name.bright_cyan());
            Ok(())
        }
    }
}

fn activate_profile(name: &OsString) -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return init_msg();
    };

    if CONFIG.is_ignored(name.to_str().expect("OsString")) {
        println!("Directory {} is on the ignore list. Please run '{}' and try again.", name.to_string_lossy().bright_red(), format!("papa profile unignore {}", name.to_string_lossy()).bright_cyan());
        return Err(anyhow!("Profile was ignored"));
    }

    let real = dir.join(name);
    if !real.try_exists()? {
        println!("Profile {} doesn't exist", name.to_string_lossy().bright_cyan());
        return Err(anyhow!("Profile not found"));
    }

    let mut cfg = CONFIG.clone();
    cfg.set_current_profile(name.to_str().expect("OsString"));
    write_config(&cfg)?;

    println!("Made {} the active profile", name.to_string_lossy().bright_cyan());

    Ok(())
}

fn list_profiles() -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return init_msg();
    };

    let mut profiles = vec![];
    for candidate in dir.read_dir()? {
        let candidate = candidate?;
        if !candidate.file_type()?.is_dir()
            || CONFIG.is_ignored(
                candidate
                    .file_name()
                    .to_str()
                    .expect("Unable to convert from OsString"),
            )
        {
            continue;
        }

        let path = candidate.path();
        profiles.push(path);
    }

    if profiles.is_empty() {
        println!("No profiles found");
        return Ok(());
    }

    println!("Available profiles:");
    for name in profiles
        .iter()
        .filter_map(|v| v.file_name().and_then(|os| os.to_str()))
    {
        println!(
            "{:<4}{}",
            if name == CONFIG.current_profile() {
                "-->"
            } else {
                ""
            }
            .bright_green(),
            name.bright_cyan()
        );
    }
    Ok(())
}

fn new_profile(name: &OsString, force: bool) -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return init_msg();
    };

    let prof = dir.join(name);
    if prof.try_exists()? {
        if force {
            fs::remove_dir_all(&prof)?;
        } else {
            println!("A folder of that name already exists, remove it first");
            return Ok(());
        }
    }
    fs::create_dir(&prof)?;

    println!("Created profile {:?}", name.bright_cyan());

    Ok(())
}

fn clone_profile(source: &String, new: &Option<String>, force: bool) -> Result<()> {
    let Some(game) = CONFIG.game_dir() else {
        return init_msg();
    };
    let source_dir = game.join(source);
    let target_dir = if let Some(target) = new {
        game.join(target)
    } else {
        game.join(format!("{}-copy", source))
    };
    let target_name = target_dir
        .file_name()
        .expect("Missing file name?")
        .to_string_lossy();

    if target_dir.try_exists()? {
        if force {
            fs::remove_dir_all(&target_dir)?;
        } else {
            println!("Profile '{}' already exists", target_name.bright_green());
        }
    }

    copy_dir(source_dir, &target_dir)?;

    println!(
        "Cloned profile '{}' to '{}'",
        source.bright_green(),
        target_name.bright_green()
    );

    Ok(())
}
