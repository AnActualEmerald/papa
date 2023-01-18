// #[cfg(feature = "northstar")]
// use crate::core::northstar::{init_northstar, update_northstar};

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
use utils::validate_modnames;

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
        #[clap(required_unless_present = "url")]
        #[clap(value_parser = validate_modnames)]
        mod_names: Vec<ModName>,

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
        #[clap(value_parser = validate_modnames)]
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
    },
    ///Updats the current northstar install. Must have been installed with `papa northstar init`.
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

    // let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();

    // let rl = Editor::<()>::new().unwrap();

    let res = match cli.command {
        // Commands::Update { yes } => update(&mut ctx, yes).await,
        // Commands::Config {
        //     mods_dir: None,
        //     cache: None,
        // } => {
        //     println!(
        //         "Current config:\n{}",
        //         toml::to_string_pretty(&ctx.config).unwrap()
        //     );
        //     Ok(())
        // }
        // Commands::Config { mods_dir, cache } => update_config(&mut ctx, mods_dir, cache),
        Commands::List { global, all } => core::list(global, all),
        // Commands::Install {
        //     mod_names: _,
        //     url: Some(url),
        //     yes: _,
        //     force: _,
        //     global: _,
        // } => install_from_url(&ctx, url).await,
        Commands::Install {
            mod_names,
            url: None,
            yes,
            force,
            global,
        } => core::install(mod_names, yes, force, global),
        // Commands::Disable { mods } => disable(&ctx, mods),
        // Commands::Enable { mods } => enable(&ctx, mods),
        Commands::Search { term } => core::search(&term),
        Commands::Remove { mod_names } => core::remove(mod_names),
        // Commands::Clear { full } => clear(&ctx, full),
        #[cfg(feature = "northstar")]
        Commands::Northstar { command } => core::northstar(&command),
        //      NstarCommands::Install { game_path } => {
        //          let game_path = if let Some(p) = game_path {
        //              p.canonicalize().unwrap()
        //          } else {
        //              std::env::current_dir().unwrap()
        //          };
        //          core.install_northstar(&game_path).await
        //      }
        //     NstarCommands::Init { game_path } => {
        //         let game_path = if let Some(p) = game_path {
        //             match p.canonicalize() {
        //                 Ok(p) => p,
        //                 Err(e) => {
        //                     debug!("{:#?}", e);
        //                     println!("{}", e);
        //                     return;
        //                 }
        //             }
        //         } else {
        //             std::env::current_dir().unwrap()
        //         };
        //         init_northstar(&mut ctx, &game_path).await
        //     }
        //     NstarCommands::Update {} => update_northstar(&mut ctx).await,
        //     #[cfg(feature = "launcher")]
        //     NstarCommands::Start {} => ctx.start_northstar(&ctx),
        // },
        // #[cfg(target_os = "linux")]
        // Commands::Include { mods, force } => include(&ctx, mods, force),
        // #[cfg(target_os = "linux")]
        // Commands::Exclude { mods } => exclude(&ctx, mods),
        // #[cfg(feature = "cluster")]
        // Commands::Cluster { command } => cluster(&mut ctx, command),
        // #[cfg(feature = "profiles")]
        // Commands::Profile { command } => profile(&mut ctx, command),
        _ => todo!(),
    };

    if let Err(e) = res {
        if cli.debug {
            debug!("{:#?}", e);
        } else {
            println!("{}", e);
        }
    }
}
