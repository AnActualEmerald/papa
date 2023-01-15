use std::fmt::Display;

use thermite::model::InstalledMod;

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
