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
    ///Show debug output
    #[clap(global = true, short, long)]
    debug: bool,
    ///Don't check cache before downloading
    #[clap(global = true, short = 'C', long = "no-cache")]
    no_cache: bool,
}

#[derive(Subcommand)]
enum Commands {
    ///Show the current config and environment info
    Env {},

    ///Export the list of currently installed mods
    Export {
        #[clap(default_value = "papa.ron")]
        file: PathBuf,
    },

    ///Import a list of mods, installing them to the current install directory
    Import {
        ///'papa.ron' file to import
        #[arg(default_value = "papa.ron")]
        file: PathBuf,

        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,

        ///Force installation
        #[clap(short, long)]
        force: bool,
    },

    ///Install a mod or mods from https://northstar.thunderstore.io/
    #[clap(alias = "i")]
    Install {
        #[clap(value_name = "MOD")]
        #[clap(help = "Mod name(s) to install")]
        #[clap(required_unless_present = "file")]
        #[clap(value_parser = validate_modname)]
        mod_names: Vec<ModName>,

        ///File to read the list of mods from
        #[arg(short = 'F', long)]
        file: Option<PathBuf>,

        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,

        ///Force installation
        #[clap(short, long)]
        force: bool,

        ///Make mod globally available (currently non-functioning)
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
    Disable {
        mods: Vec<String>,

        ///Disable all mods excluding core N* mods
        #[clap(short, long)]
        all: bool,

        ///Force disable mods including core N* mods
        #[clap(short, long)]
        force: bool,
    },
    ///Enable mod(s) or sub-mod(s)
    Enable {
        mods: Vec<String>,
        #[arg(short, long)]
        all: bool,
    },

    ///Commands for managing Northstar itself
    #[cfg(feature = "northstar")]
    #[clap(alias("ns"))]
    Northstar {
        #[clap(subcommand)]
        command: NstarCommands,
    },
}

#[derive(Subcommand)]
pub enum NstarCommands {
    ///Attempts to install Northstar to a Titanfall 2 Steam installation, or updates the configuration if it already exists.
    Init {
        /// Ignore non-fatal errors
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
        Commands::Update { yes } => core::update(yes, cli.no_cache),
        Commands::List { global, all } => core::list(global, all),
        Commands::Install {
            file, yes, force, ..
        } if file.is_some() => {
            let Some(f) = file else {
                return
            };

            core::import(f, yes, force, cli.no_cache)
        }
        Commands::Install {
            mod_names,
            yes,
            force,
            ..
        } => core::install(mod_names, yes, force, cli.no_cache),
        Commands::Disable { mods, all, force } => {
            core::disable(mods.into_iter().collect(), all, force)
        }
        Commands::Enable { mods, all } => core::enable(mods.into_iter().collect(), all),
        Commands::Search { term } => core::search(&term),
        Commands::Remove { mod_names } => core::remove(mod_names),
        Commands::Import { file, yes, force } => core::import(file, yes, force, cli.no_cache),
        Commands::Export { file } => core::export(file),
        Commands::Env {} => core::env(),
        // Commands::Clear { full } => clear(&ctx, full),
        #[cfg(feature = "northstar")]
        Commands::Northstar { command } => core::northstar(&command),
    };

    if let Err(e) = res {
        if cli.debug {
            debug!("{:#?}", e);
        }
    }
}
