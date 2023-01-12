use thermite::model::Mod;

use crate::model::ModName;

pub trait RemoteIndex {
    fn get_mod(&self, name: &ModName) -> Option<&Mod>;
}

impl RemoteIndex for Vec<Mod> {
    fn get_mod(&self, name: &ModName) -> Option<&Mod> {
        self.iter().find(|v| {
            v.name.to_lowercase() == name.name.to_lowercase()
                && v.author.to_lowercase() == name.author.to_lowercase()
        })
    }
}
