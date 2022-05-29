use std::{
    fs::{self, File},
    path::Path,
};

use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use regex::Regex;

mod utils;

#[derive(Parser)]
#[clap(name = "Papa")]
#[clap(author = "AnAcutalEmerald <emerald_actual@proton.me>")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Command line mod manager for Northstar")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[clap(name = "MOD")]
        #[clap(help = "Mod name in Author.ModName@version format")]
        mod_name: String,
    },
}

const BASE_URL: &'static str = "https://northstar.thunderstore.io/package/download";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("me", "greenboi", "papa").unwrap();
    ensure_dirs(&dirs);

    match cli.command {
        Commands::Install { mod_name } => {
            let re = Regex::new(r"(.+)\.(.+)@v?\d.\d.\d").unwrap();
            if !re.is_match(&mod_name) {
                println!("Mod name should be in 'Author.ModName@1.2.3' format");
                return;
            }

            let url = utils::parse_mod_name(&mod_name).unwrap();
            let path = dirs.cache_dir().join(format!("{}.zip", mod_name));

            if let Some(f) = check_cache(&path) {
                println!("Using cached version of {}", mod_name);
                utils::install_mod(&f, &Path::new("./")).unwrap();
                return;
            }
            match utils::download_file(format!("{}{}", BASE_URL, url), path).await {
                Ok(f) => utils::install_mod(&f, &Path::new("./")).unwrap(),
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}

fn check_cache(path: &Path) -> Option<File> {
    let opt = fs::OpenOptions::new().read(true).open(path);
    if let Ok(f) = opt {
        Some(f)
    } else {
        None
    }
}

fn ensure_dirs(dirs: &ProjectDirs) {
    fs::create_dir_all(dirs.cache_dir()).unwrap();
    fs::create_dir_all(dirs.config_dir()).unwrap();
}
