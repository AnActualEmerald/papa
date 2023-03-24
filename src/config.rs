use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref DIRS: ProjectDirs =
        ProjectDirs::from("me", "greenboi", "Papa").expect("Unable to find base dirs");
    pub static ref CONFIG: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(DIRS.config_dir().join("config.toml")))
        .merge(Env::prefixed("PAPA_"))
        .extract()
        .expect("Error reading configuration");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    game_dir: Option<PathBuf>,
    install_dir: PathBuf,
    is_server: bool,
}

impl Config {
    pub fn install_dir(&self) -> &Path {
        Path::new(&self.install_dir)
    }

    pub fn is_server(&self) -> bool {
        self.is_server
    }

    pub fn set_install_dir(&mut self, install_dir: impl Into<PathBuf>) {
        self.install_dir = install_dir.into();
    }

    pub fn set_is_server(&mut self, is_server: bool) {
        self.is_server = is_server;
    }

    pub fn set_game_dir(&mut self, game_dir: impl Into<Option<PathBuf>>) {
        self.game_dir = game_dir.into();
    }

    pub fn game_dir(&self) -> Option<&PathBuf> {
        self.game_dir.as_ref()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game_dir: None,
            install_dir: "./mods".into(),
            is_server: false,
        }
    }
}

pub fn write_config(cfg: &Config) -> Result<()> {
    let cereal = toml::to_string_pretty(cfg)?;
    fs::write(DIRS.config_dir().join("config.toml"), &cereal)?;
    Ok(())
}
