use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use semver::Version;
use thermite::model::{InstalledMod, Manifest, Mod, ModVersion};
use tracing::{debug, warn};

use crate::utils::validate_modname;

#[derive(Clone, Debug)]
pub struct Package {
    pub path: PathBuf,
    pub manifest: Manifest,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModName {
    pub author: String,
    pub name: String,
    pub version: Option<Version>,
}

impl ModName {
    pub fn new(
        author: impl Into<String>,
        name: impl Into<String>,
        version: Option<Version>,
    ) -> Self {
        Self {
            author: author.into(),
            name: name.into(),
            version,
        }
    }

    pub fn into_modstring(self) -> ModString {
        ModString { inner: self }
    }

    pub fn as_modstr(&self) -> ModStr<'_> {
        ModStr { inner: self }
    }
}

impl Display for ModName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.author, self.name)?;
        if let Some(version) = &self.version {
            write!(f, "@{version}")?;
        }

        Ok(())
    }
}

impl From<InstalledMod> for ModName {
    fn from(value: InstalledMod) -> Self {
        Self {
            author: value.author,
            name: value.manifest.name,
            version: value.manifest.version_number.parse().ok(),
        }
    }
}

impl From<&InstalledMod> for ModName {
    fn from(value: &InstalledMod) -> Self {
        Self {
            author: value.author.clone(),
            name: value.manifest.name.clone(),
            version: value.manifest.version_number.parse().ok(),
        }
    }
}

impl From<Mod> for ModName {
    fn from(value: Mod) -> Self {
        Self {
            author: value.author,
            name: value.name,
            version: value.latest.parse().ok(),
        }
    }
}

impl From<&Mod> for ModName {
    fn from(value: &Mod) -> Self {
        Self {
            author: value.author.clone(),
            name: value.name.clone(),
            version: value.latest.parse().ok(),
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
        validate_modname(&value)
    }
}

impl TryFrom<&str> for ModName {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<ModName, Self::Error> {
        validate_modname(value)
    }
}

impl TryFrom<&Path> for ModName {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> std::result::Result<Self, Self::Error> {
        let name = value
            .file_name()
            .ok_or_else(|| anyhow!("missing file name"))?;

        validate_modname(&name.to_string_lossy())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModString {
    inner: ModName,
}

impl Deref for ModString {
    type Target = ModName;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Into<ModName>> From<T> for ModString {
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Display for ModString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.author, self.inner.name)?;
        if let Some(version) = &self.version {
            write!(f, "-{version}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModStr<'a> {
    inner: &'a ModName,
}

impl<'a> Deref for ModStr<'a> {
    type Target = ModName;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a> From<&'a ModName> for ModStr<'a> {
    fn from(value: &'a ModName) -> Self {
        Self { inner: value }
    }
}

impl<'a> Display for ModStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.author, self.inner.name)?;
        if let Some(version) = &self.version {
            write!(f, "-{version}")?;
        }

        Ok(())
    }
}

pub struct Cache {
    packages: BTreeMap<ModName, PathBuf>,
    root: PathBuf,
}

impl Cache {
    pub fn as_cache_path(&self, name: impl AsRef<ModName>) -> PathBuf {
        let name = name.as_ref();
        self.root.join(format!("{name}"))
    }

    pub fn get_any(&self, name: impl AsRef<ModName>) -> Option<&PathBuf> {
        let name = name.as_ref();
        let mut keys = self
            .packages
            .keys()
            .filter(|k| {
                k.author.to_lowercase() == name.author.to_lowercase()
                    && k.name.to_lowercase() == name.name.to_lowercase()
            })
            .collect::<Vec<_>>();

        keys.sort_by(|a, b| {
            a.version
                .as_ref()
                .and_then(|av| b.version.as_ref().map(|bv| av.cmp(bv)))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.packages.get(keys.first()?)
    }

    #[inline]
    pub fn get(&self, name: impl AsRef<ModName>) -> Option<&PathBuf> {
        dbg!(&self.packages);
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
