use std::fs::{read_to_string, File};
use std::io::Write;
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
        let cfg =
            read_to_string(cfg_path).map_err(|_| ("Unable to read config file".to_string()))?;
        toml::from_str(&cfg).map_err(|_| ("Unable to parse config".to_string()))
    } else {
        let mut cfg =
            File::create(cfg_path).map_err(|_| ("Unable to create config file".to_string()))?;
        let def = Config::new(String::from("./"), true);
        let parsed = toml::to_string_pretty(&def)
            .map_err(|_| "Failed to serialize default config".to_string())?;
        cfg.write_all(parsed.as_bytes())
            .map_err(|_| "Unable to write config file".to_string())?;
        Ok(def)
    }
}

pub fn save_config(config_dir: &Path, config: Config) -> Result<(), String> {
    let cfg_path = config_dir.join("config.toml");

    if cfg_path.exists() {
        let mut cfg =
            File::create(&cfg_path).map_err(|_| ("Error opening config file".to_string()))?;
        let parsed = toml::to_string_pretty(&config)
            .map_err(|_| ("Error serializing config".to_string()))?;
        cfg.write_all(parsed.as_bytes())
            .map_err(|_| ("Unable to write config file".to_string()))?;
    } else {
        return Err("Config file does not exist to write to".to_string());
    }
    Ok(())
}
