use anyhow::{Context, Result};
use log::{debug, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
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

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
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
        self.mods.iter().any(|m| m.disabled())
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
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
    #[serde(default)]
    pub mods: HashSet<InstalledMod>,
    #[serde(default)]
    pub linked: HashSet<InstalledMod>,
}

impl LocalIndex {
    pub fn new() -> Self {
        Self {
            mods: HashSet::new(),
            linked: HashSet::new(),
        }
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
    re: Regex,
    pkgs: Vec<CachedMod>,
}

impl Cache {
    pub fn build(dir: &Path) -> Result<Self> {
        let cache = fs::read_dir(dir)?;
        let re =
            Regex::new(r"(.+)[_-](\d\.\d\.\d)(\.zip)?").context("Unable to create cache regex")?;
        let mut pkgs = vec![];
        for e in cache.flatten() {
            if !e.path().is_dir() {
                debug!("Reading {} into cache", e.path().display());
                let file_name = e.file_name();
                if let Some(c) = re.captures(file_name.to_str().unwrap()) {
                    let name = c.get(1).unwrap().as_str().trim();
                    let ver = c.get(2).unwrap().as_str().trim();
                    pkgs.push(CachedMod::new(name, ver, dir));
                    debug!("Added {} version {} to cache", name, ver);
                } else {
                    warn!(
                        "Unexpected filename in cache dir: {}",
                        file_name.to_str().unwrap()
                    );
                }
            }
        }
        Ok(Cache { pkgs, re })
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
            if let Some(parts) = self.re.captures(name.to_str().unwrap()) {
                let name = parts.get(1).unwrap().as_str();
                let ver = parts.get(2).unwrap().as_str();
                if let Some(c) = self.pkgs.iter().find(|e| e.name == name) {
                    if c.version == ver {
                        return true;
                    }
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Cluster {
    pub name: Option<String>,
    ///K: Member Name V: Member Path
    pub members: HashMap<String, PathBuf>,

    #[serde(skip)]
    path: PathBuf,
}

impl Cluster {
    pub fn new(name: Option<String>, path: PathBuf) -> Self {
        Cluster {
            name,
            members: HashMap::new(),
            path,
        }
    }

    pub fn find() -> Result<Option<Self>> {
        let has_cluster = |p: &Path| -> Result<Option<Self>> {
            for e in p.read_dir()?.flatten() {
                if e.file_name().as_os_str() == OsStr::new("cluster.ron") {
                    let raw = fs::read_to_string(e.path())?;
                    let mut clstr: Cluster = ron::from_str(&raw)?;
                    clstr.path = e.path().to_path_buf();

                    return Ok(Some(clstr));
                }
            }
            Ok(None)
        };
        let mut depth = 0;
        let mut target = std::env::current_dir()?;
        loop {
            debug!("Checking for cluster file in {}", target.display());
            let test = has_cluster(&target)?;
            if test.is_some() {
                break Ok(test);
            } else {
                if let Some(p) = target.parent() {
                    target = p.to_owned();
                    depth += 1;
                } else {
                    break Ok(None);
                }
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let pretty = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::new())?;

        fs::write(&self.path, pretty)?;

        Ok(())
    }
}
