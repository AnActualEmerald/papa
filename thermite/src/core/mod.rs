use crate::{error::ThermiteError, model::Cache};
use directories::ProjectDirs;

pub mod actions;
mod install;
pub mod utils;

pub use install::install;
pub use utils::{disable_mod, enable_mod, update_index};

#[derive(Debug, Clone)]
pub struct Ctx {
    pub cache: Cache,
    pub dirs: ProjectDirs,
    // pub local_installed: Option<LocalIndex>,
    // pub global_installed: LocalIndex,
}

impl Ctx {
    pub fn with_dirs(dirs: ProjectDirs) -> Result<Self, ThermiteError> {
        utils::ensure_dirs(&dirs);
        let cache = Cache::build(dirs.cache_dir()).unwrap();
        Ok(Ctx {
            dirs: dirs.clone(),
            cache,
        })
    }
}
