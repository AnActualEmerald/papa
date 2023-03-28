// #[cfg(feature = "northstar")]
// use crate::core::northstar::{init_northstar, update_northstar};

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub mod config;
mod core;
pub mod model;
pub mod traits;
pub mod utils;

#[macro_use]
mod macros;

use model::ModName;
use utils::validate_modname;

#[derive(Parser)]
#[clap(name = "Papa")]
#[clap(author = "AnAcutalEmerald <emerald_actual@proton.me>")]
#[clap(about = "Command line mod manager for Northstar")]
#[clap(after_help = "Welcome back. Cockpit cooling reactivated.")]
#[clap(version)]
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
        #[clap(required = true)]
        // #[clap(required_unless_present = "url")]
        #[clap(value_parser = validate_modname)]
        mod_names: Vec<ModName>,

        // ///alternate url to use - won't be tracked or updated
        // #[clap(short, long)]
        // #[clap(value_name = "url")]
        // url: option<string>,
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
        #[clap(value_parser = validate_modname)]
        #[clap(required = true)]
        mod_names: Vec<ModName>,
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
    // ///Clear mod cache
    // #[clap(alias = "c")]
    // Clear {
    //     #[clap(
    //         help = "Force removal of all files in the cahce directory, not just downloaded packages"
    //     )]
    //     #[clap(long, short)]
    //     full: bool,
    // },
    // ///Display or update the configuration
    // #[clap(alias = "cfg")]
    // Config {
    //     #[clap(long, short, value_name = "PATH")]
    //     ///Set the directory where 'mods/' can be found
    //     mods_dir: Option<String>,

    //     #[clap(long, short, value_name = "CACHE")]
    //     ///Set whether or not to cache packages
    //     cache: Option<bool>,
    // },
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
    // #[cfg(target_os = "linux")]
    // #[clap(alias = "link", alias = "ln")]
    // ///Link a global mod to the current mods folder
    // Include {
    //     mods: Vec<String>,
    //     #[clap(long, short)]
    //     force: bool,
    // },
    // #[cfg(target_os = "linux")]
    // #[clap(alias = "unlink")]
    // ///Unlink a global mod from the current mods folder
    // Exclude { mods: Vec<String> },
    ///Commands for managing Northstar itself
    #[cfg(feature = "northstar")]
    #[clap(alias("ns"))]
    Northstar {
        #[clap(subcommand)]
        command: NstarCommands,
    },
    // ///Manage clusters of Northstar servers
    // #[cfg(feature = "cluster")]
    // #[clap(alias("cl"))]
    // Cluster {
    //     #[clap(subcommand)]
    //     command: WsCommands,
    // },

    // #[cfg(feature = "profiles")]
    // ///Manage mod profiles
    // Profile {
    //     #[clap(subcommand)]
    //     command: ProfCommands,
    // },
}

#[derive(Subcommand)]
pub enum NstarCommands {
    //    ///Installs northstar to provided path, or current directory.
    //    Install { game_path: Option<PathBuf> },
    ///Attempts to install Northstar to a Titanfall 2 Steam installation, or updates the configuration if it already exists.
    Init {
        #[arg(default_value_t = false, short, long)]
        force: bool,
        /// The path to install Northstar into. Defaults to the local Titanfall 2 steam installation, if available.
        path: Option<PathBuf>,
    },
    ///Updates the current northstar install.
    Update {},
    // #[cfg(feature = "launcher")]
    // ///Start the Northstar client
    // Start {},
}

fn main() {
    let cli = Cli::parse();
    if cli.debug {
        std::env::set_var("RUST_LOG", "DEBUG");
    }

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Unable to init tracing");

    debug!("Config: {:#?}", *config::CONFIG);

    let res = match cli.command {
        Commands::Update { yes } => core::update(yes),
        Commands::List { global, all } => core::list(global, all),
        Commands::Install {
            mod_names,
            yes,
            force,
            global,
        } => core::install(mod_names, yes, force, global),
        Commands::Disable { mods } => core::disable(mods.into_iter().collect()),
        Commands::Enable { mods } => core::enable(mods.into_iter().collect()),
        Commands::Search { term } => core::search(&term),
        Commands::Remove { mod_names } => core::remove(mod_names),
        // Commands::Clear { full } => clear(&ctx, full),
        #[cfg(feature = "northstar")]
        Commands::Northstar { command } => core::northstar(&command),
        // #[cfg(target_os = "linux")]
        // Commands::Include { mods, force } => include(&ctx, mods, force),
        // #[cfg(target_os = "linux")]
        // Commands::Exclude { mods } => exclude(&ctx, mods),
        // #[cfg(feature = "cluster")]
        // Commands::Cluster { command } => cluster(&mut ctx, command),
        // #[cfg(feature = "profiles")]
        // Commands::Profile { command } => profile(&mut ctx, command),
    };

    if let Err(e) = res {
        if cli.debug {
            debug!("{:#?}", e);
        } else {
            println!("{}", e);
        }
    }
}
