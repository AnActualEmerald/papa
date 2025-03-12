use std::collections::HashSet;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::anyhow;
use anyhow::Result;
use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::IGNORED_DIRS;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("me", "greenboi", "Papa").expect("Unable to find base dirs")
});
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let path = DIRS.config_dir().join("config.toml");
    let mut cfg: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&path))
        .merge(Env::prefixed("PAPA_"))
        .extract()
        .expect("Error reading configuration");
    cfg.config_path = Some(path);
    cfg
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
    game_dir: Option<PathBuf>,
    install_dir: Option<PathBuf>,
    #[serde(default = "default_profile")]
    current_profile: String,
    #[serde(default = "default_ignore_list")]
    ignore: HashSet<String>,
    #[serde(default)]
    install_type: InstallType,
    is_server: bool,
}

impl Config {
    pub fn install_dir(&self) -> Result<PathBuf> {
        // let the explicit install dir override the game dir + profile
        Ok(if let Some(dir) = &self.install_dir {
            dir.clone()
        } else if let Some(dir) = &self.game_dir {
            dir.join(&self.current_profile).join("packages")
        } else {
            println!(
                "Please run '{}' or set '{}' in the config",
                "papa ns init".bright_cyan(),
                "install_dir".bright_cyan()
            );
            return Err(anyhow!("Unintialized config"));
        })
    }

    pub fn is_server(&self) -> bool {
        self.is_server
    }

    pub fn set_install_dir(&mut self, install_dir: impl Into<PathBuf>) {
        self.install_dir = install_dir.into().into();
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

    pub fn install_type(&self) -> &InstallType {
        &self.install_type
    }

    pub fn set_install_type(&mut self, install_type: InstallType) {
        self.install_type = install_type;
    }

    pub fn current_profile(&self) -> &str {
        self.current_profile.as_ref()
    }

    pub fn set_current_profile(&mut self, current_profile: impl Into<String>) {
        self.current_profile = current_profile.into();
    }

    pub fn current_profile_dir(&self) -> Option<PathBuf> {
        self.game_dir().map(|d| d.join(&self.current_profile))
    }

    pub fn is_ignored(&self, val: &str) -> bool {
        self.ignore.contains(val)
    }

    pub fn add_ignored(&mut self, val: impl Into<String>) -> bool {
        self.ignore.insert(val.into())
    }

    pub fn remove_ignored(&mut self, val: impl AsRef<str>) -> bool {
        self.ignore.remove(val.as_ref())
    }

    pub fn save(&self) -> Result<()> {
        let cereal = toml::to_string_pretty(self)?;
        fs::create_dir_all(DIRS.config_dir())?;
        fs::write(DIRS.config_dir().join("config.toml"), cereal)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game_dir: None,
            install_dir: None,
            is_server: false,
            config_path: None,
            current_profile: default_profile(),
            ignore: default_ignore_list(),
            install_type: InstallType::Other,
        }
    }
}

pub fn default_profile() -> String {
    "R2Northstar".into()
}

pub fn default_ignore_list() -> HashSet<String> {
    IGNORED_DIRS.into_iter().map(String::from).collect()
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InstallType {
    Steam,
    Origin,
    EA,
    #[default]
    Other,
}

impl Display for InstallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
