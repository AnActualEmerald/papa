use crate::api;
use crate::error::ThermiteError;
use crate::model;
use crate::model::LocalIndex;
use crate::model::Mod;
use crate::model::SubMod;
use directories::ProjectDirs;
use log::debug;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! g2re {
    ($e:expr) => {{
        let re = $e.replace('*', ".*");
        regex::Regex::new(&re)
    }};
}

///Takes the local and global installed files to display whether a mod is installed or not
pub async fn update_index(
    local: impl Into<Option<PathBuf>>,
    global: impl Into<Option<PathBuf>>,
) -> Vec<model::Mod> {
    let local = local.into();
    let global = global.into();
    let mut index = api::get_package_index().await.unwrap().to_vec();

    if let Some(local) = local {
        let installed = LocalIndex::load(local);
        for e in index.iter_mut() {
            if let Ok(installed) = &installed {
                e.installed = installed
                    .mods
                    .iter()
                    .any(|(n, f)| n == &e.name && f.version == e.version);
            }
        }
    }

    if let Some(global) = global {
        let glob = LocalIndex::load(global);
        for e in index.iter_mut() {
            if let Ok(glob) = &glob {
                e.global = glob
                    .mods
                    .iter()
                    .any(|(n, f)| n == &e.name && f.version == e.version);
            }
        }
    }

    index
}

#[inline]
pub fn check_cache(path: &Path) -> Option<File> {
    if let Ok(f) = OpenOptions::new().read(true).open(path) {
        Some(f)
    } else {
        None
    }
}

#[inline(always)]
pub fn ensure_dirs(dirs: &ProjectDirs) {
    fs::create_dir_all(dirs.cache_dir()).unwrap();
    fs::create_dir_all(dirs.config_dir()).unwrap();
    fs::create_dir_all(dirs.data_local_dir()).unwrap();
}

pub fn remove_file(path: &Path) -> Result<(), ThermiteError> {
    fs::remove_file(path).map_err(|e| e.into())
}

//    pub fn remove_dir(dir: &Path) -> Result<(), String> {
//        fs::remove_dir_all(dir)
//            .map_err(|_| format!("Unable to remove directory {}", dir.display()))?;
//
//        Ok(())
//    }

pub fn clear_cache(dir: &Path, force: bool) -> Result<(), ThermiteError> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            clear_cache(&path, force)?;
            fs::remove_dir(&path)?;
        } else if path.extension() == Some(OsStr::new("zip")) || force {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}

//    pub fn list_dir(dir: &Path) -> Result<Vec<String>, String> {
//        Ok(fs::read_dir(dir)
//            .map_err(|_| format!("unable to read directory {}", dir.display()))
//            .map_err(|_| format!("Unable to read directory {}", dir.display()))?
//            .filter(|f| f.is_ok())
//            .map(|f| f.unwrap())
//            .map(|f| f.file_name().to_string_lossy().into_owned())
//            .collect())
//    }

// #[inline]
// pub fn save_file(file: &Path, data: String) -> Result<()> {
//     fs::write(file, data.as_bytes())?;
//     Ok(())
// }

//    //supposing the mod name is formatted like Author.Mod@v1.0.0
//    pub fn parse_mod_name(name: &str) -> Option<String> {
//        let parts = name.split_once('.')?;
//        let author = parts.0;
//        //let parts = parts.1.split_once('@')?;
//        let m_name = parts.1;
//        //let ver = parts.1.replace('v', "");
//
//        let big_snake = Converter::new()
//            .set_delim("_")
//            .set_pattern(Pattern::Capital);
//
//        Some(format!("{}.{}", author, big_snake.convert(&m_name)))
//    }

///
pub fn resolve_deps(
    deps: &Vec<impl AsRef<str>>,
    index: &Vec<Mod>,
) -> Result<Vec<Mod>, ThermiteError> {
    let mut valid = vec![];
    for dep in deps {
        let dep_name = dep.as_ref().split('-').collect::<Vec<&str>>()[1];
        if let Some(d) = index.iter().find(|f| f.name == dep_name) {
            valid.push(d.clone());
        } else {
            return Err(ThermiteError::DepError(dep.as_ref().into()));
        }
    }
    Ok(valid)
}

pub fn disable_mod(dir: impl AsRef<Path>, m: &mut SubMod) -> Result<bool, ThermiteError> {
    if m.disabled() {
        return Ok(false);
    }

    let old_path = dir.as_ref().join(&m.path);

    let dir = dir.as_ref().join(".disabled");
    let new_path = dir.join(&m.path);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    debug!(
        "Rename mod from {} to {}",
        old_path.display(),
        new_path.display()
    );
    fs::rename(&old_path, &new_path)?;

    m.path = Path::new(".disabled").join(&m.path);

    Ok(true)
}

pub fn enable_mod(dir: impl AsRef<Path>, m: &mut SubMod) -> Result<bool, ThermiteError> {
    if !m.disabled() {
        return Ok(false);
    }

    let old_path = dir.as_ref().join(&m.path);
    m.path = m.path.strip_prefix(".disabled").unwrap().to_path_buf();
    let new_path = dir.as_ref().join(&m.path);

    debug!(
        "Rename mod from {} to {}",
        old_path.display(),
        new_path.display()
    );

    fs::rename(old_path, new_path)?;

    Ok(true)
}
