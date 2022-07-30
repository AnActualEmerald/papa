use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Context, Result};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub mod_dir: PathBuf,
    cache: bool,
    #[serde(default)]
    pub game_path: PathBuf,
    pub nstar_version: Option<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "ManageMode::default")]
    pub mode: ManageMode,
    #[serde(default = "default_prof")]
    pub profile: String,
}

fn default_prof() -> String {
    "default".to_string()
}

#[derive(Serialize, Deserialize)]
pub enum ManageMode {
    Client,
    Server,
}

impl ManageMode {
    fn default() -> Self {
        ManageMode::Client
    }
}

impl Config {
    pub fn new(dir: String, cache: bool, game_path: String, nstar_version: Option<String>) -> Self {
        Config {
            mod_dir: PathBuf::from(dir),
            cache,
            game_path: PathBuf::from(game_path),
            nstar_version,
            exclude: vec![
                "ns_startup_args.txt".to_string(),
                "ns_startup_args_dedi.txt".to_string(),
            ],
            mode: ManageMode::Client,
            profile: "default".to_string(),
        }
    }

    pub fn mod_dir(&self) -> &Path {
        Path::new(&self.mod_dir)
    }

    pub fn cache(&self) -> bool {
        self.cache
    }

    pub fn set_dir(&mut self, dir: &str) {
        self.mod_dir = PathBuf::from(dir);
    }

    pub fn set_cache(&mut self, cache: &bool) {
        self.cache = *cache;
    }
}

pub fn load_config(config_dir: &Path) -> Result<Config> {
    let cfg_path = config_dir.join("config.toml");
    if cfg_path.exists() {
        let cfg = read_to_string(cfg_path).context("Unable to read config file")?;
        toml::from_str(&cfg).context("Unable to parse config")
    } else {
        let mut cfg = File::create(cfg_path).context("Unable to create config file")?;
        let def = Config::new(String::from("./mods"), true, String::new(), None);
        let parsed = toml::to_string_pretty(&def).context("Failed to serialize default config")?;
        cfg.write_all(parsed.as_bytes())
            .context("Unable to write config file")?;
        Ok(def)
    }
}

pub fn save_config(config_dir: &Path, config: &Config) -> Result<()> {
    let cfg_path = config_dir.join("config.toml");

    if cfg_path.exists() {
        let mut cfg = File::create(&cfg_path).context("Error opening config file")?;
        let parsed = toml::to_string_pretty(&config).context("Error serializing config")?;
        cfg.write_all(parsed.as_bytes())
            .context("Unable to write config file")?;
    } else {
        return Err(anyhow!("Config file does not exist to write to"));
    }
    Ok(())
}
