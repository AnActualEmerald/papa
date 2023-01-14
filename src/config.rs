use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref DIRS: ProjectDirs =
        ProjectDirs::from("me", "greenboi", "Papa").expect("Unable to find base dirs");
    pub static ref CONFIG: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(DIRS.config_dir().join("config.toml")))
        .extract()
        .expect("Error reading configuration");
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_dir: "./mods".into(),
            is_server: false,
        }
    }
}
