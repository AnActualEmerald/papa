use crate::{error::ThermiteError, model::Cache};
use directories::ProjectDirs;

pub mod actions;
mod install;
mod update;
pub(crate) mod utils;

pub use install::install;
pub use utils::{resolve_deps, update_index};

#[derive(Debug, Clone)]
pub struct Ctx {
    pub cache: Cache,
    pub dirs: ProjectDirs,
}

impl Ctx {
    pub fn with_dirs(dirs: ProjectDirs) -> Result<Self, ThermiteError> {
        utils::ensure_dirs(&dirs);
        let cache = Cache::build(dirs.cache_dir()).unwrap();
        Ok(Ctx { dirs, cache })
    }
}
