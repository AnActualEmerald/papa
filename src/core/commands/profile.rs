use std::{ffi::OsString, fs};

use anyhow::{anyhow, Result};
use clap::Subcommand;
use copy_dir::copy_dir;
use owo_colors::OwoColorize;

use crate::config::CONFIG;

#[derive(Subcommand)]
pub enum ProfileCommands {
    #[clap(alias("ls"))]
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
    }
}

fn list_profiles() -> Result<()> {
    let list: Vec<&'static str> = {
        let list = include_str!("../../ignore_list.csv");

        list.split('\n').collect()
    };

    let Some(dir) = CONFIG.game_dir() else {
        println!("Please run '{}' first", "papa ns init".bright_cyan());
        return Err(anyhow!("Game path not set"));
    };

    let mut profiles = vec![];
    for candidate in dir.read_dir()? {
        let candidate = candidate?;
        if !candidate.file_type()?.is_dir()
            || list.contains(
                &candidate
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
        .filter_map(|v| v.file_name().map(|os| os.to_str()).flatten())
    {
        println!(
            "{:<4}{}",
            if profiles.len() > 1 && name == CONFIG.current_profile() {
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
        println!("Please run '{}' first", "papa ns init".bright_cyan());
        return Err(anyhow!("Game path not set"));
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
        println!("Please run '{}' first", "papa ns init".bright_cyan());
        return Err(anyhow!("Game path not set"));
    };
    let source_dir = game.join(&source);
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

    copy_dir(&source_dir, &target_dir)?;

    println!(
        "Cloned profile '{}' to '{}'",
        source.bright_green(),
        target_name.bright_green()
    );

    Ok(())
}
