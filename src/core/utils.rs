use crate::api;
use crate::api::model::LocalIndex;
use crate::api::model::Profile;
use crate::api::model::SubMod;
use crate::model;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::debug;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::path::Path;

use super::Ctx;

#[macro_export]
macro_rules! g2re {
    ($e:expr) => {{
        let re = $e.replace('*', ".*");
        regex::Regex::new(&re)
    }};
}

///Takes the local and global installed files to display whether a mod is installed or not
pub async fn update_index(local: &Path, global: &Path) -> Vec<model::Mod> {
    print!("Updating package index...");
    let mut index = api::get_package_index().await.unwrap().to_vec();
    //        save_file(&dirs.cache_dir().join("index.ron"), index)?;
    let installed = LocalIndex::load(local);
    let glob = LocalIndex::load(global);
    for e in index.iter_mut() {
        if let Ok(installed) = &installed {
            e.installed = installed
                .mods
                .iter()
                .any(|(n, f)| n == &e.name && f.version == e.version);
        }
        if let Ok(glob) = &glob {
            e.global = glob
                .mods
                .iter()
                .any(|(n, f)| n == &e.name && f.version == e.version);
        }
    }
    println!(" Done!");
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
    Profile::ensure_default(dirs.config_dir()).unwrap();
}

pub fn remove_file(path: &Path) -> Result<()> {
    fs::remove_file(path).context(format!("Unable to remove file {}", path.display()))
}

pub fn clear_cache(dir: &Path, force: bool) -> Result<()> {
    for entry in fs::read_dir(dir).context(format!("unable to read directory {}", dir.display()))? {
        let path = entry.context("Error reading directory entry")?.path();

        if path.is_dir() {
            clear_cache(&path, force)?;
            fs::remove_dir(&path)
                .context(format!("Unable to remove directory {}", path.display()))?;
        } else if path.extension() == Some(OsStr::new("zip")) || force {
            fs::remove_file(&path).context(format!("Unable to remove file {}", path.display()))?;
        }
    }

    Ok(())
}

pub fn disable_mod(ctx: &Ctx, m: &mut SubMod) -> Result<bool> {
    if m.disabled() {
        return Ok(false);
    }

    let old_path = ctx.local_target.join(&m.path);

    let dir = ctx.local_target.join(".disabled");
    let new_path = dir.join(&m.path);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    debug!(
        "Rename mod from {} to {}",
        old_path.display(),
        new_path.display()
    );
    fs::rename(&old_path, &new_path).context("Failed to rename mod")?;

    m.path = Path::new(".disabled").join(&m.path);

    Ok(true)
}

pub fn enable_mod(m: &mut SubMod, mods_dir: &Path) -> Result<bool> {
    if !m.disabled() {
        return Ok(false);
    }

    let old_path = mods_dir.join(&m.path);
    m.path = m.path.strip_prefix(".disabled")?.to_path_buf();
    let new_path = mods_dir.join(&m.path);

    debug!(
        "Rename mod from {} to {}",
        old_path.display(),
        new_path.display()
    );

    fs::rename(old_path, new_path).context("Failed to rename mod")?;

    Ok(true)
}
