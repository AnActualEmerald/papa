use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::api::model;
use crate::core::config;
use directories::ProjectDirs;
use rustyline::Editor;

mod api;
mod core;

#[derive(Parser)]
#[clap(name = "Papa")]
#[clap(author = "AnAcutalEmerald <emerald_actual@proton.me>")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Command line mod manager for Northstar")]
#[clap(after_help = "Welcome back. Cockpit cooling reactivated.")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///Install a mod or mods from https://northstar.thunderstore.io/
    Install {
        #[clap(value_name = "MOD")]
        #[clap(help = "Mod name(s) to install")]
        #[clap(required_unless_present = "url")]
        mod_names: Vec<String>,

        ///Alternate url to use - won't be tracked or updated
        #[clap(short, long)]
        #[clap(value_name = "URL")]
        url: Option<String>,

        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    ///Remove a mod or mods from the current mods directory
    Remove {
        #[clap(value_name = "MOD")]
        #[clap(help = "Mod name(s) to remove")]
        mod_names: Vec<String>,
    },
    ///List installed mods
    List {},
    ///Clear mod cache
    Clear {
        #[clap(
            help = "Force removal of all files in the cahce directory, not just downloaded packages"
        )]
        #[clap(long, short)]
        full: bool,
    },
    ///Display or update the configuration
    Config {
        #[clap(long, short, value_name = "PATH")]
        ///Set the directory where 'mods/' can be found
        mods_dir: Option<String>,

        #[clap(long, short, value_name = "CACHE")]
        ///Set whether or not to cache packages
        cache: Option<bool>,
    },
    ///Update currently installed mods
    Update {
        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    ///Search for a mod
    Search {
        ///The term to search for
        term: Vec<String>,
    },
    Disable {
        mods: Vec<String>,
    },
    Enable {
        mods: Vec<String>,
    },

    Northstar {
        #[clap(subcommand)]
        command: NstarCommands,
    },
}

#[derive(Subcommand)]
enum NstarCommands {
    Install { game_path: PathBuf },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();
    env_logger::init();

    let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();
    let config = config::load_config(dirs.config_dir()).unwrap();

    let rl = Editor::<()>::new();

    let mut core = core::Core::new(config, dirs, rl);

    match cli.command {
        Commands::Update { yes } => core.update(yes).await?,
        Commands::Config {
            mods_dir: None,
            cache: None,
        } => {
            println!(
                "Current config:\n{}",
                toml::to_string_pretty(&core.config).unwrap()
            );
        }
        Commands::Config { mods_dir, cache } => core.update_config(mods_dir, cache)?,
        Commands::List {} => core.list()?,
        Commands::Install {
            mod_names: _,
            url: Some(url),
            yes: _,
        } => core.install_from_url(url).await?,
        Commands::Install {
            mod_names,
            url: None,
            yes,
        } => core.install(mod_names, yes).await?,
        Commands::Disable { mods } => core.disable(mods)?,
        Commands::Enable { mods } => core.enable(mods)?,
        Commands::Search { term } => core.search(term).await?,
        Commands::Remove { mod_names } => core.remove(mod_names)?,
        Commands::Clear { full } => core.clear(full)?,
        Commands::Northstar { command } => match command {
            NstarCommands::Install { game_path } => {
                core.install_northstar(&game_path.canonicalize().unwrap())
                    .await?;
            }
        },
    }

    Ok(())
}
