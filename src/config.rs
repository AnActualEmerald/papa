use std::path::Path;

use directories::ProjectDirs;
use figment::providers::{Format, Toml};
use figment::Figment;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref DIRS: ProjectDirs =
        ProjectDirs::from("me", "greenboi", "Papa").expect("Unable to find base dirs");
    pub static ref CONFIG: Config =
        Figment::from(Toml::file(DIRS.config_dir().join("config.toml")))
            .extract()
            .expect("Error reading configuration");
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    install_dir: String,
}

impl Config {
    pub fn install_dir(&self) -> &Path {
        Path::new(&self.install_dir)
    }

    pub fn set_install_dir(&mut self, install_dir: impl Into<String>) {
        self.install_dir = install_dir.into();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_dir: "./mods".into(),
        }
    }
}
