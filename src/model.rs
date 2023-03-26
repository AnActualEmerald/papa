use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use thermite::model::{InstalledMod, Mod};
use tracing::{debug, warn};

use crate::utils::validate_modname;

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModName {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
}

impl ModName {
    pub fn new(
        author: impl Into<String>,
        name: impl Into<String>,
        version: Option<String>,
    ) -> Self {
        Self {
            author: author.into(),
            name: name.into(),
            version,
        }
    }
}

impl Display for ModName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.author, self.name)?;
        if let Some(version) = &self.version {
            write!(f, "@{}", version)?;
        }

        Ok(())
    }
}

impl From<InstalledMod> for ModName {
    fn from(value: InstalledMod) -> Self {
        Self {
            author: value.author,
            name: value.manifest.name,
            version: Some(value.manifest.version_number),
        }
    }
}

impl From<&InstalledMod> for ModName {
    fn from(value: &InstalledMod) -> Self {
        Self {
            author: value.author.clone(),
            name: value.manifest.name.clone(),
            version: Some(value.manifest.version_number.clone()),
        }
    }
}

impl From<Mod> for ModName {
    fn from(value: Mod) -> Self {
        Self {
            author: value.author,
            name: value.name,
            version: Some(value.latest),
        }
    }
}

impl From<&Mod> for ModName {
    fn from(value: &Mod) -> Self {
        Self {
            author: value.author.clone(),
            name: value.name.clone(),
            version: Some(value.latest.clone()),
        }
    }
}

impl AsRef<ModName> for ModName {
    fn as_ref(&self) -> &ModName {
        self
    }
}

impl TryFrom<String> for ModName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<ModName, Self::Error> {
        validate_modname(&value).map_err(|e| anyhow!("{e}"))
    }
}

impl TryFrom<&str> for ModName {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<ModName, Self::Error> {
        validate_modname(value).map_err(|e| anyhow!("{e}"))
    }
}

pub struct BorrowedModName<'a> {
    author: &'a str,
    name: &'a str,
    version: Option<&'a str>,
}

impl<'a> From<&'a InstalledMod> for BorrowedModName<'a> {
    fn from(value: &'a InstalledMod) -> Self {
        Self {
            author: &value.author,
            name: &value.manifest.name,
            version: Some(&value.manifest.version_number),
        }
    }
}

pub struct Cache {
    packages: BTreeMap<ModName, PathBuf>,
    root: PathBuf,
}

impl Cache {
    pub fn to_cache_path(&self, name: impl AsRef<ModName>) -> PathBuf {
        let name = name.as_ref();
        self.root.join(format!("{name}"))
    }

    #[inline]
    pub fn get(&self, name: impl AsRef<ModName>) -> Option<&PathBuf> {
        self.packages.get(name.as_ref())
    }

    #[inline]
    pub fn has(&self, name: impl AsRef<ModName>) -> bool {
        self.packages.contains_key(name.as_ref())
    }

    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let mut packages = BTreeMap::new();

        let path = path.as_ref();
        if !path.is_dir() {
            return Err(anyhow!("Cannot read cache from file"));
        }
        let mut rd = path.read_dir()?;
        while let Some(Ok(entry)) = rd.next() {
            // ignore any nested directories in the cache
            if entry.file_type()?.is_dir() {
                continue;
            }

            let name = entry
                .file_name()
                .into_string()
                .expect("Unable to convert from OsString");
            match validate_modname(name.trim_end_matches(".zip")) {
                Ok(name) => {
                    debug!("Adding {name} to cache");
                    packages.insert(name, entry.path());
                }
                Err(_) => {
                    warn!("Skipping invalid modname {name}");
                }
            }
        }

        Ok(Self {
            packages,
            root: path.to_owned(),
        })
    }
}
