use std::fmt::Display;

use thermite::model::{InstalledMod, Mod};

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
