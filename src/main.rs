use clap::{Parser, Subcommand};

use directories::ProjectDirs;
use regex::Regex;

mod actions;
mod api;
#[allow(dead_code)]
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
        #[clap(required_unless_present = "url")]
        mod_names: Vec<String>,

        ///Alternate url to use
        #[clap(short, long)]
        #[clap(value_name = "URL")]
        url: Option<String>,
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
    ///Display or update the configuration
    Config {
        #[clap(long, short, value_name = "PATH")]
        ///Set the directory where 'mods/' can be found
        mods_dir: Option<String>,

        #[clap(long, short, value_name = "CACHE")]
        ///Set whether or not to cache packages
        cache: Option<bool>,
    },
    Update {},
}

//There is an API for thunderstore but getting the download links from it is kind of annoying so this will do for now
const BASE_URL: &str = "https://northstar.thunderstore.io/package/download";

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();
    utils::ensure_dirs(&dirs);
    let mut config = config::load_config(dirs.config_dir()).unwrap();

    match cli.command {
        Commands::Update {} => {
            print!("Updating package index...");
            let index = ron::to_string(&api::get_package_index().await?)
                .map_err(|_| "Error converting index to RON".to_string())?;
            utils::save_file(&dirs.cache_dir().join("index.ron"), index)?;
            println!(" Done!");
        }
        Commands::Config {
            mods_dir: None,
            cache: None,
        } => {
            println!(
                "Current config:\n{}",
                toml::to_string_pretty(&config).unwrap()
            );
        }
        Commands::Config { mods_dir, cache } => {
            if let Some(dir) = mods_dir {
                config.set_dir(&dir);
                println!("Set mods parent directory to {}", dir);
            }

            if let Some(cache) = cache {
                config.set_cache(&cache);
                if cache {
                    println!("Turned caching on");
                } else {
                    println!("Turned caching off");
                }
            }

            config::save_config(dirs.config_dir(), config)?;
        }
        Commands::List {} => {
            let mods = utils::list_dir(&config.mod_dir().join("mods/"))?;
            if !mods.is_empty() {
                println!("Installed mods:\n");
                mods.into_iter()
                    .enumerate()
                    .for_each(|f| println!("{}. {}", f.0 + 1, f.1));
            } else {
                println!("No mods currently installed");
            }
        }
        Commands::Install {
            mod_names: _,
            url: Some(url),
        } => {
            let file_name = url
                .as_str()
                .replace(":", "")
                .split("/")
                .collect::<Vec<&str>>()
                .join("");
            println!("Downloading to {}", file_name);
            let path = dirs.cache_dir().join(file_name);
            match actions::download_file(format!("{}", url), path.clone()).await {
                Ok(f) => {
                    let pkg = actions::install_mod(&f, &config).unwrap();
                    utils::remove_file(&path)?;
                    println!("Installed {}", pkg);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        Commands::Install {
            mod_names,
            url: None,
        } => {
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
                let pkg = actions::install_mod(f, &config).unwrap();
                println!("Installed {}", pkg);
            });
        }
        Commands::Remove { mod_names } => {
            let re = Regex::new(r"(.+)\.(.+)").unwrap();
            let valid = mod_names
                .iter()
                .filter(|f| re.is_match(f))
                .collect::<Vec<&String>>();
            actions::uninstall(valid, &config)?;
        }
        Commands::Clear { full } => {
            if full {
                println!("Clearing cache files...");
            } else {
                println!("Clearing cached packages...");
            }
            utils::clear_cache(dirs.cache_dir(), full)?;
        }
    }

    Ok(())
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

    pub fn remove_file(path: &Path) -> Result<(), String> {
        fs::remove_file(path).map_err(|_| format!("Unable to remove file {}", path.display()))
    }

    pub fn remove_dir(dir: &Path) -> Result<(), String> {
        fs::remove_dir_all(dir)
            .map_err(|_| format!("Unable to remove directory {}", dir.display()))?;

        Ok(())
    }

    pub fn clear_cache(dir: &Path, force: bool) -> Result<(), String> {
        for entry in
            fs::read_dir(dir).map_err(|_| format!("unable to read directory {}", dir.display()))?
        {
            let path = entry
                .map_err(|_| "Error reading directory entry".to_string())?
                .path();

            println!("Removing {}", path.display());

            if path.is_dir() {
                clear_cache(&path, force)?;
                fs::remove_dir(&path)
                    .map_err(|_| format!("Unable to remove directory {}", path.display()))?;
            } else if path.ends_with(".zip") {
                fs::remove_file(&path)
                    .map_err(|_| format!("Unable to remove file {}", path.display()))?;
            } else if force {
                fs::remove_file(&path)
                    .map_err(|_| format!("Unable to remove file {}", path.display()))?;
            }
        }

        Ok(())
    }

    pub fn list_dir(dir: &Path) -> Result<Vec<String>, String> {
        Ok(fs::read_dir(dir)
            .map_err(|_| format!("unable to read directory {}", dir.display()))
            .map_err(|_| format!("Unable to read directory {}", dir.display()))?
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap())
            .map(|f| f.file_name().to_string_lossy().into_owned())
            .collect())
    }

    pub fn save_file(file: &Path, data: String) -> Result<(), String> {
        fs::write(file, data.as_bytes())
            .map_err(|_| format!("Unable to write file {}", file.display()))?;
        Ok(())
    }
}
