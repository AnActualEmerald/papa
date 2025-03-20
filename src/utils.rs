use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use crate::{
    config::{CONFIG, DIRS},
    model::{Cache, ModName},
    modfile,
};
use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use owo_colors::OwoColorize;
use regex::Regex;
use thermite::{
    core::{find_mods, get_enabled_mods},
    model::{EnabledMods, InstalledMod, ModVersion},
    prelude::{download_with_progress, install_mod},
};
use tracing::{debug, error, trace};

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S\w+)[\.-](\w+)(?:[@-](\d+\.\d+\.\d+))?$").expect("ModName regex")
});

pub(crate) fn validate_modname(input: &str) -> Result<ModName> {
    if let Some(captures) = RE.captures(input) {
        let mut name = ModName::default();
        if let Some(author) = captures.get(1) {
            name.author = author.as_str().to_string();
        }

        if let Some(n) = captures.get(2) {
            name.name = n.as_str().to_string();
        }

        name.version = captures.get(3).map(|v| v.as_str().to_string());

        Ok(name)
    } else {
        Err(anyhow!(
            "Mod name '{input}' should be in 'Author.ModName' format"
        ))
    }
}

#[must_use]
pub(crate) fn to_file_size_string(size: u64) -> String {
    if size / 1_000_000 >= 1 {
        let size = size as f64 / 1_048_576f64;

        format!("{size:.2} MB")
    } else {
        let size = size as f64 / 1024f64;
        format!("{size:.2} KB")
    }
}

pub(crate) fn ensure_dir(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();

    debug!("Checking if path '{}' exists", dir.display());
    if dir.try_exists()? {
        debug!("Path '{}' already exists", dir.display());
    } else {
        debug!("Path '{}' doesn't exist, creating it", dir.display());
        fs::create_dir_all(dir)?;
    }

    Ok(())
}

pub fn find_enabled_mods(start: impl AsRef<Path>) -> Option<EnabledMods> {
    let dir = start.as_ref();

    if let Ok(mods) = get_enabled_mods(dir) {
        Some(mods)
    } else if let Some(parent) = dir.parent() {
        find_enabled_mods(parent)
    } else {
        None
    }
}

pub(crate) fn download_and_install(
    mods: Vec<(ModName, impl AsRef<ModVersion>)>,
    check_cache: bool,
    cont: bool,
) -> Result<Vec<PathBuf>> {
    if mods.is_empty() {
        println!("Nothing to do!");
        return Ok(vec![]);
    }

    println!("Downloading packages...");
    let mut files = vec![];
    let cache_dir = DIRS.cache_dir();
    ensure_dir(cache_dir)?;
    let cache = Cache::from_dir(cache_dir)?;

    for (mn, v) in mods {
        if check_cache {
            if let Some(path) = cache.get(&mn) {
                println!("Using cached version of {}", mn.bright_cyan());
                files.push((mn, v.as_ref().full_name.clone(), modfile!(z, path)?));
                continue;
            }
        }
        let v = v.as_ref();
        // flush!()?;
        let filename = cache.as_cache_path(&mn);
        let pb = ProgressBar::new(v.file_size)
            .with_style(
                ProgressStyle::with_template("{msg}{bar} {bytes}/{total_bytes} {duration}")?
                    .progress_chars(".. "),
            )
            .with_message(format!("Downloading {}", mn.bright_cyan()));
        let mut file = modfile!(filename)?;
        download_with_progress(&mut file, &v.url, |delta, _, _| {
            pb.inc(delta);
        })
        .context(format!("Error downloading {}", mn.red()))?;
        pb.finish();
        files.push((mn, v.full_name.clone(), file));
    }

    let pb = ProgressBar::new_spinner()
        .with_style(
            ProgressStyle::with_template("{prefix}{msg}\t{spinner}\t{pos}/{len}")?
                .tick_chars("(|)|\0"),
        )
        .with_prefix("Installing ");
    pb.set_tab_width(1);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_length(files.len() as u64);

    let mut had_error = false;

    let mut installed = vec![];

    for (mn, full_name, f) in files.iter().progress_with(pb.clone()) {
        pb.set_message(format!("{}", mn.bright_cyan()));
        if CONFIG.is_server() {
            todo!();
        } else {
            ensure_dir(CONFIG.install_dir()?)?;
            let mod_path = CONFIG.install_dir()?;
            match install_mod(full_name, f, mod_path) {
                Err(e) => {
                    had_error = true;
                    pb.suspend(|| {
                        println!("Failed to install {}: {e}", mn.bright_red());
                        debug!("{e:?}");
                    });
                    if !cont {
                        pb.finish_and_clear();
                        println!("Aborted due to error");
                        return Err(e.into());
                    }
                }
                Ok(mod_path) => {
                    pb.suspend(|| println!("Installed {}", mn.bright_cyan()));
                    installed.push(mod_path);
                }
            }
        }
    }

    pb.disable_steady_tick();
    pb.set_prefix("");
    pb.set_tab_width(0);
    pb.finish_with_message("Installed ");
    if had_error {
        println!("Finished with errors");
    } else {
        println!("Done!");
    }
    Ok(installed)
}

#[derive(Default)]
pub struct GroupedMods {
    pub enabled: BTreeMap<ModName, BTreeSet<InstalledMod>>,
    pub disabled: BTreeMap<ModName, BTreeSet<InstalledMod>>,
}

impl GroupedMods {
    pub fn try_from_dir(dir: &Path) -> Result<Self> {
        let mods = match find_mods(dir) {
            Ok(mods) => mods,
            Err(e) => {
                error!("Error finding mods: {e}");
                vec![]
            }
        };

        if mods.is_empty() {
            // println!("No mods found");
            return Ok(Self::default());
        }

        debug!("Found {} mods", mods.len());
        trace!("{:?}", mods);
        let enabled_mods = find_enabled_mods(dir);

        let mut enabled = BTreeMap::new();
        let mut disabled = BTreeMap::new();
        for m in mods {
            let local_name = m.mod_json.name.clone();

            let mn = m.clone().into();
            let process_mod = |mod_group: &mut BTreeMap<ModName, BTreeSet<InstalledMod>>| {
                if let Some(group) = mod_group.get_mut(&mn) {
                    debug!("Adding submod {local_name} to group {}", mn);
                    group.insert(m);
                } else {
                    debug!("Adding group {local_name} for sdubmod {}", mn);
                    let group = BTreeSet::from([m]);
                    mod_group.insert(mn, group);
                }
            };

            if let Some(em) = enabled_mods.as_ref() {
                if em.is_enabled(&local_name) {
                    process_mod(&mut enabled);
                } else {
                    process_mod(&mut disabled);
                }
            } else {
                process_mod(&mut enabled);
            }
        }

        Ok(Self { enabled, disabled })
    }
}

/// Find the roots for all packages in the given directory
pub fn find_package_roots(dir: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();

    let mut res = vec![];

    for entry in fs::read_dir(dir)? {
        let child = entry?;

        if !child.file_type()?.is_dir() {
            continue;
        }

        let manifest_path = child.path().join("manifest.json");

        if manifest_path.try_exists()? {
            res.push(child.path())
        }
    }

    Ok(res)
}

#[inline]
#[must_use]
pub fn init_msg() -> anyhow::Error {
    println!("Please run '{}' first", "papa ns init".bright_cyan());
    anyhow!("Game path not set")
}

#[cfg(test)]
mod test {

    use crate::utils::validate_modname;

    #[test]
    fn suceed_validate_modname() {
        let test_name = "foo.bar@0.1.0";
        assert!(validate_modname(test_name).is_ok());
    }
}
