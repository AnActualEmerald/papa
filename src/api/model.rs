use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub desc: String,
    pub deps: Vec<String>,
    pub file_size: i64,
    #[serde(skip)]
    pub installed: bool,
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
}

impl InstalledMod {
    pub fn flatten_paths(&self) -> Vec<PathBuf> {
        self.mods.iter().map(|m| m.path.clone()).collect()
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
