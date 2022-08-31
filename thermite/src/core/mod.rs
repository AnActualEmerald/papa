use crate::{error::ThermiteError, model::Cache};
use directories::ProjectDirs;

mod install;

#[derive(Debug, Clone)]
pub struct Ctx {
    pub cache: Cache,
    pub cluster: Option<Cluster>,
    pub config: Config,
    pub dirs: ProjectDirs,
    // pub local_installed: Option<LocalIndex>,
    // pub global_installed: LocalIndex,
}

impl Ctx {
    pub fn with_dirs(dirs: ProjectDirs) -> Result<Self, ThermiteError> {
        utils::ensure_dirs(&dirs);
        let config = config::load_config(dirs.config_dir())?;
        let cache = Cache::build(dirs.cache_dir()).unwrap();
        Ok(Ctx {
            config,
            dirs: dirs.clone(),
            cache,
            cluster: Cluster::find().unwrap_or(None), //don't use `?` here so we don't crash everything if there's no cluster
        })
    }
}
