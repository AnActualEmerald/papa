use std::fmt::Display;

#[derive(Default, Clone, Debug)]
pub struct ModName {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
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
