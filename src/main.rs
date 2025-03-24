#![feature(let_chains)]

use core::{RunOptions, profile::ProfileCommands};
use std::{fs, io::IsTerminal, path::PathBuf, process::ExitCode};

use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{ArgValueCompleter, CompleteEnv, Shell, env::Shells};
use tracing::{debug, error};
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

mod completers;
pub mod config;
mod core;
pub mod model;
pub mod traits;
pub mod utils;

#[macro_use]
mod macros;

use model::ModName;
use utils::validate_modname;

use crate::{config::DIRS, core::profile};

pub const IGNORED_DIRS: [&str; 8] = [
    "__Installer",
    "__overlay",
    "bin",
    "Core",
    "r2",
    "vpk",
    "platform",
    "Support",
];

#[derive(Parser)]
#[clap(name = "papa")]
#[clap(author = "AnAcutalEmerald <emerald@emeraldgreen.dev>")]
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
    ///File to write logs to, will truncate any existing file
    #[clap(global = true, long = "log-file")]
    log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate completions for the current shell
    Complete {
        ///Print the shell script for initializing completions
        #[arg(long, short)]
        init: bool,
        ///Shell to generate for, defaults to the value of the SHELL environment variable
        #[clap(value_name = "SHELL", value_enum)]
        shell: Option<Shell>,
    },

    ///Show the current config and environment info
    Env {},

    ///Export the list of currently installed mods
    Export {
        ///File to export to
        #[clap(default_value = "papa.ron")]
        file: PathBuf,
    },

    ///Import a list of mods, installing them to the current install directory
    Import {
        ///File to import
        #[arg(default_value = "papa.ron", value_hint = ValueHint::FilePath)]
        file: PathBuf,

        ///Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,

        ///Force installation
        #[clap(short, long)]
        force: bool,
    },

    ///Install a mod or mods from <https://northstar.thunderstore.io/>
    #[clap(alias = "i", alias = "add")]
    Install {
        #[clap(value_name = "MOD", add = ArgValueCompleter::new(completers::mod_index))]
        #[clap(help = "Mod name(s) to install")]
        #[clap(required_unless_present = "file")]
        #[clap(value_parser = validate_modname)]
        mod_names: Vec<ModName>,

        ///File to read the list of mods from
        #[arg(short = 'F', long, value_hint = ValueHint::FilePath)]
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
        #[clap(value_name = "MOD", add = ArgValueCompleter::new(completers::installed_mods))]
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
        #[clap(value_hint = ValueHint::Other)]
        term: Vec<String>,
    },

    ///Disable mod(s) or sub-mod(s)
    Disable {
        #[clap(add = ArgValueCompleter::new(completers::enabled_mods))]
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
        #[clap(add = ArgValueCompleter::new(completers::disabled_mods))]
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

    ///Start Northstar through steam or origin
    #[cfg(feature = "launcher")]
    #[clap(alias("start"))]
    Run {
        #[command(flatten)]
        options: RunOptions,
    },

    #[clap(alias = "p", alias = "profiles")]
    Profile {
        #[clap(subcommand)]
        command: ProfileCommands,
    },
}

#[derive(Subcommand)]
pub enum NstarCommands {
    ///Attempts to install Northstar to a Titanfall 2 Steam installation, or updates the configuration if it already exists.
    Init {
        /// Ignore non-fatal errors
        #[arg(short, long)]
        force: bool,

        /// The path to install Northstar into. Defaults to the local Titanfall 2 steam installation, if available.
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        #[arg(short = 'C', long = "no-cache")]
        no_cache: bool,
    },
    ///Updates the current northstar install.
    Update {},
    // #[cfg(feature = "launcher")]
    // ///Start the Northstar client
    // Start {},
    ///Uninstalls Northstar and all related files
    Reset {
        ///Skip confirmation (MAKE SURE YOU WANT TO DO THIS)
        #[arg(long, short)]
        yes: bool,
    },
}

fn main() -> ExitCode {
    CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::try_parse();
    if let Err(e) = cli {
        e.exit();
    }
    let cli = cli.expect("cli");
    if cli.debug {
        unsafe {
            // always safe to call from single threaded programs
            std::env::set_var("RUST_LOG", "DEBUG");
        }
    }

    let (writer, _handle) = if let Some(file) = cli.log_file {
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .expect("log file");
        tracing_appender::non_blocking(file)
    } else {
        let file_logger = tracing_appender::rolling::never(DIRS.config_dir(), "papa.log");
        tracing_appender::non_blocking(file_logger)
    };

    let reg = Registry::default().with(
        fmt::layer()
            .compact()
            .without_time()
            .with_writer(writer)
            .with_ansi(false)
            .with_filter(EnvFilter::from_default_env()),
    );

    if cli.debug {
        reg.with(
            fmt::layer()
                .compact()
                .with_ansi(true)
                .without_time()
                .with_filter(EnvFilter::from_default_env()),
        )
        .init();
    } else {
        reg.init();
    }

    // FmtSubscriber::builder()
    //     .without_time()
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .init();

    debug!("Config: {:#?}", *config::CONFIG);

    let res = match cli.command {
        Commands::Complete { shell, init } => {
            if let Some(shell) = shell.or_else(Shell::from_env) {
                if init {
                    if std::io::stdout().is_terminal() {
                        let file = match shell {
                            Shell::Bash => "~/.bashrc",
                            Shell::Elvish => "~/.elvish/rc.elv",
                            Shell::Fish => "~/.config/fish/conf.d/papa.fish",
                            Shell::PowerShell => "$PROFILE",
                            Shell::Zsh => "~/.zshrc",
                            _ => panic!("Unknown shell"),
                        };

                        eprintln!("Redirect this command to '{file}'");
                        eprintln!("e.g. 'papa complete {shell} >> {file}'");
                        eprintln!();
                    }

                    match shell {
                        Shell::Bash => {
                            println!("source <(papa complete bash)");
                            Ok(())
                        }
                        Shell::Elvish => {
                            println!("eval (papa complete elvish | slurp)");
                            Ok(())
                        }
                        Shell::Fish => {
                            println!("source (papa complete fish | psub)");
                            Ok(())
                        }
                        Shell::PowerShell => {
                            println!("papa complete powershell | Out-String | Invoke-Expression");
                            Ok(())
                        }
                        Shell::Zsh => {
                            println!("source <(papa complete zsh)");
                            Ok(())
                        }
                        _ => Err(anyhow::anyhow!("Unknown shell")),
                    }
                } else {
                    let cli = Cli::command();

                    Shells::builtins()
                        .completer(&shell.to_string())
                        .expect("shell completer")
                        .write_registration(
                            "COMPLETE",
                            cli.get_name(),
                            cli.get_name(),
                            cli.get_name(),
                            &mut std::io::stdout(),
                        )
                        .map_err(anyhow::Error::from)
                }
            } else {
                eprintln!("Please provide a shell to generate completions for");
                Err(anyhow::anyhow!("Unknown shell"))
            }
        }
        Commands::Update { yes } => core::update(yes, cli.no_cache),
        Commands::List { global, all } => core::list(global, all),
        Commands::Install {
            file, yes, force, ..
        } if file.is_some() => {
            let Some(f) = file else {
                return ExitCode::FAILURE;
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
        #[cfg(feature = "launcher")]
        Commands::Run { options } => core::run(options),
        Commands::Profile { command } => profile::handle(&command, cli.no_cache),
    };

    if let Err(e) = res {
        if cli.debug {
            error!("{:#?}", e);
        }
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
