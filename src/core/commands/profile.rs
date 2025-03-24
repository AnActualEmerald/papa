use std::{
    ffi::OsString,
    fs::{self, File},
    io::{ErrorKind, IsTerminal, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, anyhow};
use clap::{Args, Subcommand, ValueHint};
use clap_complete::ArgValueCompleter;
use copy_dir::copy_dir;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use semver::Version;
use thermite::core::manage::install_northstar_profile;

use crate::{
    config::{CONFIG, DIRS},
    get_answer,
    model::{Cache, ModName},
    traits::Answer,
    update_cfg,
    utils::{download_northstar, init_msg},
};

#[derive(Subcommand)]
pub enum ProfileCommands {
    #[clap(alias = "s", alias = "choose", alias = "activate")]
    ///Select a profile
    Select {
        ///Name of the profile to select
        #[clap(add = ArgValueCompleter::new(crate::completers::profiles))]
        name: String,
    },
    ///Ignore a directory, preventing it from being displayed as a profile
    Ignore {
        #[clap(value_hint = ValueHint::DirPath)]
        name: String,
    },
    ///Un-ignore a directory, allowing it to be displayed as a profile
    Unignore {
        #[clap(value_hint = ValueHint::DirPath)]
        name: String,
    },
    #[clap(alias("ls"))]
    ///List profiles
    List,
    ///Create a new profile
    #[clap(alias("n"))]
    New {
        ///Name of the profile to create
        #[clap(value_hint = ValueHint::DirPath)]
        name: OsString,

        #[command(flatten)]
        options: NewOptions,
    },

    #[clap(alias = "dupe", alias = "cp", alias = "copy")]
    ///Clone an existing profile
    Clone {
        #[clap(add = ArgValueCompleter::new(crate::completers::profiles))]
        source: String,
        #[clap(value_hint = ValueHint::DirPath)]
        new: Option<String>,
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Args, Clone)]
pub struct NewOptions {
    ///Don't inlcude Norhtstar core files and mods
    #[arg(long, short)]
    empty: bool,
    ///Remove any existing folder of the same name
    #[arg(long, short)]
    force: bool,
    ///Answer "yes" to any prompts
    #[arg(long, short)]
    yes: bool,
    ///The version of Northstar to use when for this profile
    ///
    /// Leave unset for latest
    #[arg(long, short, conflicts_with = "empty")]
    version: Option<Version>,
}

pub fn handle(command: &ProfileCommands, no_cache: bool) -> Result<()> {
    match command {
        ProfileCommands::List => list_profiles(),
        ProfileCommands::New { name, options } => new_profile(name, options.clone(), no_cache),
        ProfileCommands::Clone { source, new, force } => clone_profile(source, new, *force),
        ProfileCommands::Select { name } => activate_profile(name),
        ProfileCommands::Ignore { name } => {
            update_cfg!(ignore(name))?;
            println!("Added {} to ignore list", name.bright_cyan());
            Ok(())
        }
        ProfileCommands::Unignore { name } => {
            update_cfg!(unignore(name))?;
            println!("Removed {} from ignore list", name.bright_cyan());
            Ok(())
        }
    }
}

fn activate_profile(name: &String) -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return Err(init_msg());
    };

    if CONFIG.is_ignored(name) {
        println!(
            "Directory {} is on the ignore list. Please run '{}' and try again.",
            name.bright_red(),
            format!("papa profile unignore {name}").bright_cyan()
        );
        return Err(anyhow!("Profile was ignored"));
    }

    let real = dir.join(name);
    if !real.try_exists()? {
        println!("Profile {} doesn't exist", name.bright_cyan());
        return Err(anyhow!("Profile not found"));
    }

    update_cfg!(profile(name))?;

    println!("Made {} the active profile", name.bright_cyan());

    Ok(())
}

pub fn find_profiles(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut profiles = vec![];
    for candidate in dir.as_ref().read_dir()? {
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

    Ok(profiles)
}

fn list_profiles() -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return Err(init_msg());
    };

    let profiles = find_profiles(dir)?;

    // output the raw list if we're in a script or pipeline
    if !std::io::stdout().is_terminal() {
        let out = std::io::stdout();
        for p in profiles {
            if let Some(name) = p.file_name().and_then(|os| os.to_str()) {
                if let Err(e) = writeln!(out.lock(), "{name}") {
                    if e.kind() != ErrorKind::BrokenPipe {
                        return Err(e.into());
                    }
                }
            }
        }

        return Ok(());
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

fn new_profile(name: &OsString, options: NewOptions, no_cache: bool) -> Result<()> {
    let Some(dir) = CONFIG.game_dir() else {
        return Err(init_msg());
    };

    let prof = dir.join(name);
    if prof.try_exists()? {
        if options.force {
            fs::remove_dir_all(&prof)?;
        } else {
            println!("A folder of that name already exists, remove it first");
            return Ok(());
        }
    }
    fs::create_dir(&prof)?;

    if !options.empty {
        let nsname = ModName::new("northstar", "Northstar", options.version.clone());
        let cache = Cache::from_dir(DIRS.cache_dir())?;
        let file = if !no_cache
            && let Some(nstar) = if options.version.is_some() {
                dbg!(cache.get(nsname))
            } else {
                cache.get_any(nsname)
            } {
            File::open(nstar)?
        } else {
            let ans = if let Some(version) = options.version.as_ref() {
                get_answer!(options.yes, "Download Northstar {}? [Y/n] ", version)?
            } else {
                get_answer!(options.yes, "Download latest Northstar? [Y/n] ")?
            };

            if ans.is_no() {
                println!("Not downloading Northstar, aborting");
                return Ok(());
            } else {
                download_northstar(options.version)?
            }
        };

        let bar = ProgressBar::new_spinner()
            .with_style(
                ProgressStyle::with_template("{prefix}{spinner:.cyan}")?
                    .tick_strings(&["   ", ".  ", ".. ", "...", "   "]),
            )
            .with_prefix("Installing Northstar core files");
        bar.enable_steady_tick(Duration::from_millis(500));
        install_northstar_profile(file, prof)?;
        bar.finish();
    }

    println!("Created profile {}", name.display().bright_cyan());

    Ok(())
}

fn clone_profile(source: &String, new: &Option<String>, force: bool) -> Result<()> {
    let Some(game) = CONFIG.game_dir() else {
        return Err(init_msg());
    };
    let source_dir = game.join(source);
    let target_dir = if let Some(target) = new {
        game.join(target)
    } else {
        game.join(format!("{source}-copy"))
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
