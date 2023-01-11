// use crate::api::model::{Cache, Cluster};

// use self::config::Config;

// pub struct Ctx {
//     pub config: Config,
//     pub dirs: ProjectDirs,
//     pub rl: Editor<()>,
//     pub cache: Cache,
//     pub local_target: PathBuf,
//     pub global_target: PathBuf,
//     pub cluster: Option<Cluster>,
//     // pub local_installed: Option<LocalIndex>,
//     // pub global_installed: LocalIndex,
// }

// impl Ctx {
//     pub fn new(dirs: ProjectDirs, rl: Editor<()>) -> Result<Self> {
//         utils::ensure_dirs(&dirs);
//         let config = config::load_config(dirs.config_dir()).expect("Unable to load config file");
//         let cache = Cache::build(dirs.cache_dir()).unwrap();
//         let lt = config.mod_dir.clone();
//         let gt = dirs.data_local_dir();
//         // let l_mods = LocalIndex::load(&lt).ok();
//         // let g_mods = LocalIndex::create(gt);
//         Ok(Ctx {
//             config,
//             dirs: dirs.clone(),
//             rl,
//             cache,
//             local_target: lt,
//             global_target: gt.to_path_buf(),
//             cluster: Cluster::find().unwrap_or(None), //don't use `?` here so we don't crash everything if there's no cluster
//                                                       // local_installed: l_mods,
//                                                       // global_installed: g_mods,
//         })
//     }
// }
mod commands;
