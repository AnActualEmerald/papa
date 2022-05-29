use std::{
    fs::{self, File},
    path::Path,
};

use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use directories::ProjectDirs;
use regex::Regex;

mod actions;
mod config;

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
        #[clap(help = "Mod name(s) in Author.ModName@version format")]
        mod_names: Vec<String>,
    },
    ///Remove a mod or mods from the current mods directory
    Remove {
        #[clap(value_name = "MOD")]
        #[clap(help = "Mod name(s) to remove in Author.ModName format")]
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
}

//There is an API for thunderstore but getting the download links from it is kind of annoying so this will do for now
const BASE_URL: &'static str = "https://northstar.thunderstore.io/package/download";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();
    utils::ensure_dirs(&dirs);
    let mut config = config::load_config(dirs.config_dir()).unwrap();

    match cli.command {
        Commands::List {} => {
            let mods = utils::list_dir(&config.mod_dir().join("mods/")).unwrap();
            if mods.len() > 0 {
                println!("Installed mods:\n");
                mods.into_iter().for_each(|f| println!("\t{}", f));
            } else {
                println!("No mods currently installed");
            }
        }
        Commands::Install { mod_names } => {
            let mut valid = vec![];
            for name in mod_names {
                let re = Regex::new(r"(.+)\.(.+)@(v?\d.\d.\d)").unwrap();

                if !re.is_match(&name) {
                    println!("Mod name should be in 'Author.ModName@1.2.3' format");
                    continue;
                }

                let url = actions::parse_mod_name(&name).unwrap();
                let path = dirs.cache_dir().join(format!("{}.zip", name));

                if let Some(f) = utils::check_cache(&path) {
                    println!("Using cached version of {}", name);
                    valid.push(f);
                    continue;
                }
                match actions::download_file(format!("{}{}", BASE_URL, url), path).await {
                    Ok(f) => valid.push(f),
                    Err(e) => eprintln!("{}", e),
                }
            }
            valid.iter().for_each(|f| {
                let pkg = actions::install_mod(f, &mut config).unwrap();
                println!("Installed {}", pkg);
            });

            config::save_config(dirs.config_dir(), config).unwrap();
        }
        Commands::Remove { mod_names } => {
            let re = Regex::new(r"(.+)\.(.+)").unwrap();
            let valid = mod_names
                .iter()
                .filter(|f| re.is_match(&f))
                .collect::<Vec<&String>>();
            actions::uninstall(valid, &config).unwrap();
        }
        Commands::Clear { full } => {
            if full {
                println!("Clearing cache files...");
            } else {
                println!("Clearing cached packages...");
            }
            utils::remove_dir(dirs.cache_dir(), full).unwrap();
        }
        _ => {}
    }
}

mod utils {
    use directories::ProjectDirs;
    use std::fs::{self, File, OpenOptions};
    use std::path::Path;

    pub fn check_cache(path: &Path) -> Option<File> {
        let opt = OpenOptions::new().read(true).open(path);
        if let Ok(f) = opt {
            Some(f)
        } else {
            None
        }
    }

    pub fn ensure_dirs(dirs: &ProjectDirs) {
        fs::create_dir_all(dirs.cache_dir()).unwrap();
        fs::create_dir_all(dirs.config_dir()).unwrap();
    }

    pub fn remove_dir(dir: &Path, force: bool) -> Result<(), String> {
        for entry in
            fs::read_dir(dir).or(Err(format!("unable to read directory {}", dir.display())))?
        {
            let path = entry
                .or(Err(format!("Error reading directory entry")))?
                .path();

            println!("Removing {}", path.display());

            if path.is_dir() {
                remove_dir(&path, force)?;
                fs::remove_dir(&path).or(Err(format!(
                    "Unable to remove directory {}",
                    path.display()
                )))?;
            } else {
                if !force && path.ends_with(".zip") {
                    fs::remove_file(&path)
                        .or(Err(format!("Unable to remove file {}", path.display())))?;
                } else if force {
                    fs::remove_file(&path)
                        .or(Err(format!("Unable to remove file {}", path.display())))?;
                }
            }
        }

        fs::remove_dir(&dir).or(Err(format!("Unable to remove directory {}", dir.display())))?;
        println!("Removing {}", dir.display());

        Ok(())
    }

    pub fn list_dir(dir: &Path) -> Result<Vec<String>, String> {
        Ok(fs::read_dir(dir)
            .or(Err(format!("Unable to read directory {}", dir.display())))?
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap())
            .map(|f| f.file_name().to_string_lossy().into_owned())
            .collect())
    }
}
