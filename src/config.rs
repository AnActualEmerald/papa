use std::fs::{self, read_to_string, File};
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    mod_dir: String,
    cache: bool,
}

impl Config {
    pub fn new(dir: String, cache: bool) -> Self {
        Config {
            mod_dir: dir,
            cache,
        }
    }

    pub fn mod_dir(&self) -> &Path {
        Path::new(&self.mod_dir)
    }

    pub fn cache(&self) -> bool {
        self.cache
    }

    pub fn set_dir(&mut self, dir: String) {
        self.mod_dir = dir;
    }

    pub fn set_cache(&mut self, cache: bool) {
        self.cache = cache;
    }
}

pub fn load_config(config_dir: &Path) -> Result<Config, String> {
    let cfg_path = config_dir.join("config.toml");
    if cfg_path.exists() {
        let cfg = read_to_string(cfg_path).or(Err(format!("Unable to read config file")))?;
        toml::from_str(&cfg).or(Err(format!("Unable to parse config")))
    } else {
        File::create(cfg_path).or(Err(format!("Unable to create config file")))?;
        Ok(Config::new(String::from("./"), true))
    }
}

pub fn save_config(config_dir: &Path, config: Config) -> Result<(), String> {
    let cfg_path = config_dir.join("config.toml");

    if cfg_path.exists() {
        let mut cfg = File::create(&cfg_path).or(Err(format!("Error opening config file")))?;
        let parsed =
            toml::to_string_pretty(&config).or(Err(format!("Error serializing config")))?;
        cfg.write_all(parsed.as_bytes())
            .or(Err(format!("Unable to write config file")))?;
    } else {
        return Err(format!("Config file does not exist to write to"));
    }
    Ok(())
}
