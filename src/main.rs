use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::debug;

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
    #[clap(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    ///Install a mod or mods from https://northstar.thunderstore.io/
    #[clap(alias = "i")]
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

        ///Force installation
        #[clap(short, long)]
        force: bool,

        ///Make mod globally available
        #[clap(short, long)]
        global: bool,
    },
    ///Remove a mod or mods from the current mods directory
    #[clap(alias = "r", alias = "rm")]
    Remove {
        #[clap(value_name = "MOD")]
        #[clap(help = "Mod name(s) to remove")]
        mod_names: Vec<String>,
    },
    ///List installed mods
    #[clap(alias = "l", alias = "ls")]
    List {
        ///List only globally installed mods
        #[clap(short, long)]
        global: bool,

        ///List both local and global mods
        #[clap(short, long)]
        all: bool,
    },
    ///Clear mod cache
    #[clap(alias = "c")]
    Clear {
        #[clap(
            help = "Force removal of all files in the cahce directory, not just downloaded packages"
        )]
        #[clap(long, short)]
        full: bool,
    },
    ///Display or update the configuration
    #[clap(alias = "cfg")]
    Config {
        #[clap(long, short, value_name = "PATH")]
        ///Set the directory where 'mods/' can be found
        mods_dir: Option<String>,

        #[clap(long, short, value_name = "CACHE")]
        ///Set whether or not to cache packages
        cache: Option<bool>,
    },
    ///Update currently installed mods
    #[clap(alias = "u")]
    Update {
        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    ///Search for a mod
    #[clap(alias = "s")]
    Search {
        ///The term to search for
        term: Vec<String>,
    },

    ///Disable mod(s) or sub-mod(s)
    Disable { mods: Vec<String> },
    ///Enable mod(s) or sub-mod(s)
    Enable { mods: Vec<String> },

    //These will only be available on linux for now because symlinks on Windows are weird
    #[cfg(target_os = "linux")]
    #[clap(alias = "link", alias = "ln")]
    ///Link a global mod to the current mods folder
    Include {
        mods: Vec<String>,
        #[clap(long, short)]
        force: bool,
    },
    #[cfg(target_os = "linux")]
    #[clap(alias = "unlink")]
    ///Unlink a global mod from the current mods folder
    Exclude { mods: Vec<String> },

    ///Commands for managing Northstar itself
    #[cfg(feature = "northstar")]
    #[clap(alias("ns"))]
    Northstar {
        #[clap(subcommand)]
        command: NstarCommands,
    },
}

#[derive(Subcommand)]
enum NstarCommands {
    //    ///Installs northstar to provided path, or current directory.
    //    Install { game_path: Option<PathBuf> },
    ///Initializes a new northstar installation in the provided path, or current directory.
    Init { game_path: Option<PathBuf> },
    ///Updats the current northstar install. Must have been installed with `papa northstar init`.
    Update {},
    #[cfg(feature = "launcher")]
    ///Start the Northstar client
    Start {},
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.debug {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::builder().format_timestamp(None).init();

    let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();
    let config = config::load_config(dirs.config_dir()).unwrap();

    let rl = Editor::<()>::new();

    let mut core = core::Core::new(config, dirs, rl);

    let res = match cli.command {
        Commands::Update { yes } => core.update(yes).await,
        Commands::Config {
            mods_dir: None,
            cache: None,
        } => {
            println!(
                "Current config:\n{}",
                toml::to_string_pretty(&core.config).unwrap()
            );
            Ok(())
        }
        Commands::Config { mods_dir, cache } => core.update_config(mods_dir, cache),
        Commands::List { global, all } => core.list(global, all),
        Commands::Install {
            mod_names: _,
            url: Some(url),
            yes: _,
            force: _,
            global: _,
        } => core.install_from_url(url).await,
        Commands::Install {
            mod_names,
            url: None,
            yes,
            force,
            global,
        } => core.install(mod_names, yes, force, global).await,
        Commands::Disable { mods } => core.disable(mods),
        Commands::Enable { mods } => core.enable(mods),
        Commands::Search { term } => core.search(term).await,
        Commands::Remove { mod_names } => core.remove(mod_names),
        Commands::Clear { full } => core.clear(full),
        #[cfg(feature = "northstar")]
        Commands::Northstar { command } => match command {
            //      NstarCommands::Install { game_path } => {
            //          let game_path = if let Some(p) = game_path {
            //              p.canonicalize().unwrap()
            //          } else {
            //              std::env::current_dir().unwrap()
            //          };
            //          core.install_northstar(&game_path).await
            //      }
            NstarCommands::Init { game_path } => {
                let game_path = if let Some(p) = game_path {
                    match p.canonicalize() {
                        Ok(p) => p,
                        Err(e) => {
                            debug!("{:#?}", e);
                            println!("{}", e);
                            return;
                        }
                    }
                } else {
                    std::env::current_dir().unwrap()
                };
                core.init_northstar(&game_path).await
            }
            NstarCommands::Update {} => core.update_northstar().await,
            #[cfg(feature = "launcher")]
            NstarCommands::Start {} => core.start_northstar(),
        },
        #[cfg(target_os = "linux")]
        Commands::Include { mods, force } => core.include(mods, force),
        #[cfg(target_os = "linux")]
        Commands::Exclude { mods } => core.exclude(mods),
    };

    if let Some(e) = res.err() {
        if cli.debug {
            debug!("{:#?}", e);
        } else {
            println!("{}", e);
        }
    }
}
