use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::core::utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub desc: String,
    pub deps: Vec<String>,
    pub file_size: i64,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub upgradable: bool,
}

impl Mod {
    pub fn file_size_string(&self) -> String {
        if self.file_size / 1_000_000 >= 1 {
            let size = self.file_size as f64 / 1_048_576f64;

            format!("{:.2} MB", size)
        } else {
            let size = self.file_size as f64 / 1024f64;
            format!("{:.2} KB", size)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledMod {
    pub package_name: String,
    pub version: String,
    pub mods: Vec<SubMod>,
    //TODO: Implement local dep tracking
    pub depends_on: Vec<String>,
    pub needed_by: Vec<String>,
}

impl InstalledMod {
    pub fn flatten_paths(&self) -> Vec<&PathBuf> {
        self.mods.iter().map(|m| &m.path).collect()
    }

    pub fn any_disabled(&self) -> bool {
        let b = self.mods.iter().any(|m| m.disabled());
        b
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubMod {
    pub path: PathBuf,
    pub name: String,
}

impl SubMod {
    pub fn new(name: &str, path: &Path) -> Self {
        SubMod {
            name: name.to_string(),
            path: path.to_owned(),
        }
    }

    pub fn disabled(&self) -> bool {
        self.path
            .components()
            .any(|f| f.as_os_str() == OsStr::new(".disabled"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version_number: String,
    pub website_url: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalIndex {
    pub mods: Vec<InstalledMod>,
}

impl LocalIndex {
    pub fn new() -> Self {
        Self { mods: vec![] }
    }
}

#[derive(Clone)]
struct CachedMod {
    name: String,
    version: String,
    path: PathBuf,
}

impl PartialEq for CachedMod {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl CachedMod {
    fn new(name: &str, version: &str, path: &Path) -> Self {
        CachedMod {
            name: name.to_string(),
            version: version.to_string(),
            path: path.to_owned(),
        }
    }
}

pub struct Cache {
    pkgs: Vec<CachedMod>,
}

impl Cache {
    pub fn build(dir: &Path) -> Result<Self> {
        let cache = fs::read_dir(dir)?;
        let mut pkgs = vec![];
        for e in cache {
            if let Ok(e) = e {
                if !e.path().is_dir() {
                    let file_name = e.file_name();
                    let re =
                        Regex::new(r"(.+)_(.+)(\.zip)?").context("Unable to create cache regex")?;
                    if let Some(c) = re.captures(file_name.to_str().unwrap()) {
                        let name = c.get(1).unwrap().as_str();
                        let ver = c.get(2).unwrap().as_str();
                        pkgs.push(CachedMod::new(name, ver, dir));
                    }
                }
            }
        }
        Ok(Cache { pkgs })
    }

    ///Cleans all cached versions of a package except the version provided
    pub fn clean(&mut self, name: &str, version: &str) -> Result<bool> {
        let mut res = false;

        for m in self
            .pkgs
            .clone()
            .into_iter()
            .filter(|e| e.name == name && e.version != version)
        {
            if let Some(index) = self.pkgs.iter().position(|e| e == &m) {
                utils::remove_file(&m.path)?;
                self.pkgs.swap_remove(index);
                res = true
            }
        }

        Ok(res)
    }

    ///Checks if a path is in the current cache
    pub fn check(&self, path: &Path) -> Option<File> {
        if self.has(path) {
            self.open_file(path)
        } else {
            None
        }
    }

    fn has(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            let parts: Vec<&str> = name.to_str().unwrap().split('_').collect();
            let name = parts[0];
            let ver = parts[1];
            if let Some(c) = self.pkgs.iter().find(|e| e.name == name) {
                if c.version == ver {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    fn open_file(&self, path: &Path) -> Option<File> {
        if let Ok(f) = OpenOptions::new().read(true).open(path) {
            Some(f)
        } else {
            None
        }
    }
}
