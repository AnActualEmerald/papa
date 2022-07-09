pub mod actions;
pub mod config;
#[cfg(feature = "northstar")]
pub mod northstar;

pub(crate) mod utils;

use std::fs;

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use log::{debug, trace, warn};
use regex::Regex;
use rustyline::Editor;

use self::config::Config;
use crate::api;
use crate::api::model::{self, Cache, InstalledMod, LocalIndex, Mod};

use anyhow::{anyhow, Context, Result};

pub struct Core {
    pub config: Config,
    dirs: ProjectDirs,
    rl: Editor<()>,
    cache: Cache,
    local_target: PathBuf,
    global_target: PathBuf,
}

impl Core {
    pub fn new(dirs: ProjectDirs, rl: Editor<()>) -> Self {
        utils::ensure_dirs(&dirs);
        let config = config::load_config(dirs.config_dir()).expect("Unable to load config file");
        let cache = Cache::build(dirs.cache_dir()).unwrap();
        let lt = config.mod_dir.clone();
        let gt = dirs.data_local_dir();
        Core {
            config,
            dirs: dirs.clone(),
            rl,
            cache,
            local_target: lt,
            global_target: gt.to_path_buf(),
        }
    }

    pub async fn update(&mut self, yes: bool) -> Result<()> {
        let local_target = self.local_target.clone();
        let global_target = self.global_target.clone();
        print!("Updating package index...");
        let index = &api::get_package_index().await?;
        println!(" Done!");
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        let mut global = utils::get_installed(self.dirs.data_local_dir())?;
        let outdated: Vec<&model::Mod> = index
            .iter()
            .filter(|e| {
                installed.mods.iter().any(|i| {
                    i.package_name.trim() == e.name.trim() && i.version.trim() != e.version.trim()
                })
            })
            .collect();
        let glob_outdated: Vec<&model::Mod> = index
            .iter()
            .filter(|e| {
                global.mods.iter().any(|i| {
                    i.package_name.trim() == e.name.trim() && i.version.trim() != e.version.trim()
                })
            })
            .collect();

        if outdated.is_empty() && glob_outdated.is_empty() {
            println!("Already up to date!");
        } else {
            let size: i64 = outdated.iter().map(|f| f.file_size).sum::<i64>()
                + glob_outdated.iter().map(|f| f.file_size).sum::<i64>();

            println!("Updating: \n");
            print!("\t");
            outdated
                .iter()
                .chain(glob_outdated.iter())
                .enumerate()
                .for_each(|(i, f)| {
                    if i > 0 && i % 5 == 0 {
                        println!("\n");
                        print!("\t");
                    }
                    print!(" \x1b[36m{}@{}\x1b[0m ", f.name, f.version);
                });
            println!("\n");
            if !yes {
                if let Ok(line) = self.rl.readline(&format!(
                    "Will download ~{:.2} MB (compressed), okay? (This will overwrite any changes made to mod files) [Y/n]: ",
                    size as f64 / 1_048_576f64
                )) {
                    if line.to_lowercase() == "n" {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }

            self.do_update(&outdated, &mut installed, &local_target)
                .await?;
            self.do_update(&glob_outdated, &mut global, &global_target)
                .await?;
            utils::save_installed(self.config.mod_dir(), &installed)?;
            utils::save_installed(self.dirs.data_local_dir(), &global)?;
        }
        //Would be cool to do an && on these let statements
        if let Some(current) = &self.config.nstar_version {
            if let Some(nmod) = index.iter().find(|e| e.name.to_lowercase() == "northstar") {
                if *current != nmod.version {
                    println!("An update for Northstar is available! \x1b[93m{}\x1b[0m -> \x1b[93m{}\x1b[0m", current, nmod.version);
                    println!("Run \"\x1b[96mpapa northstar update\x1b[0m\" to install it!");
                }
            }
        }
        Ok(())
    }

    async fn do_update(
        &mut self,
        outdated: &Vec<&Mod>,
        installed: &mut LocalIndex,
        target: &Path,
    ) -> Result<()> {
        let mut downloaded = vec![];
        for base in outdated {
            let name = &base.name;
            let url = &base.url;
            let path = self.dirs.cache_dir().join(format!("{}.zip", name));
            match actions::download_file(url, path).await {
                Ok(f) => downloaded.push(f),
                Err(e) => eprintln!("{}", e),
            }
        }

        println!(
            "Extracting mod{} to {}...",
            if downloaded.len() > 1 { "s" } else { "" },
            self.config.mod_dir().display()
        );
        for f in downloaded.into_iter() {
            let mut pkg = actions::install_mod(&f, target).unwrap();
            self.cache.clean(&pkg.package_name, &pkg.version)?;
            if let Some(i) = installed
                .mods
                .clone()
                .iter()
                .find(|e| e.package_name == pkg.package_name)
            {
                let mut inst = i.clone();
                inst.version = pkg.version;
                installed.mods.remove(i);
                //Don't know if sorting is needed here but seems like a good assumption
                inst.mods
                    .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                pkg.mods
                    .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                for (a, b) in inst.mods.iter().zip(pkg.mods.iter()) {
                    trace!("a mod: {:#?} | b mod: {:#?}", a, b);
                    if a.disabled() {
                        fs::remove_dir_all(&a.path).unwrap();
                        debug!(
                            "Moving mod from {} to {}",
                            b.path.display(),
                            a.path.display()
                        );
                        fs::rename(&b.path, &a.path).unwrap_or_else(|e| {
                            debug!("Unable to move sub-mod to old path");
                            debug!("{}", e);
                        });
                    }
                }

                inst.mods = pkg.mods;
                installed.mods.insert(inst);
                println!("Updated {}", pkg.package_name);
            }
        }

        Ok(())
    }

    pub fn list(&self, global: bool, all: bool) -> Result<()> {
        let do_list = |target, global| -> Result<()> {
            let index = utils::get_installed(target)?;
            let msg = if global {
                "Global mods:"
            } else {
                "Local mods:"
            };
            println!("{}", msg);
            if !index.mods.is_empty() {
                index.mods.into_iter().for_each(|m| {
                    let disabled = if !m.any_disabled() || m.mods.len() > 1 {
                        ""
                    } else {
                        "[disabled]"
                    };
                    println!(
                        "  \x1b[92m{}@{}\x1b[0m {}",
                        m.package_name, m.version, disabled
                    );
                    if m.mods.len() > 1 {
                        for (i, e) in m.mods.iter().enumerate() {
                            let character = if i + 1 < m.mods.len() { "├" } else { "└" };
                            let disabled = if e.disabled() { "[disabled]" } else { "" };
                            println!(
                                "    \x1b[92m{}─\x1b[0m \x1b[0;96m{}\x1b[0m {}",
                                character, e.name, disabled
                            );
                        }
                    }
                });
            } else {
                println!("  No mods currently installed");
            }
            println!();
            if !index.linked.is_empty() {
                println!("Linked mods:");
                index
                    .linked
                    .into_iter()
                    .for_each(|m| println!("  \x1b[92m{}@{}\x1b[0m", m.package_name, m.version));
                println!();
            }

            Ok(())
        };

        if !all {
            let target = if global {
                self.dirs.data_local_dir()
            } else {
                self.config.mod_dir()
            };

            do_list(target, global)
        } else {
            do_list(self.config.mod_dir(), false)?;
            do_list(self.dirs.data_local_dir(), true)?;
            Ok(())
        }
    }

    pub async fn install_from_url(&self, url: String) -> Result<()> {
        let file_name = url
            .as_str()
            .replace(':', "")
            .split('/')
            .collect::<Vec<&str>>()
            .join("");
        println!("Downloading to {}", file_name);
        let path = self.dirs.cache_dir().join(file_name);
        match actions::download_file(url.to_string().as_str(), path.clone()).await {
            Ok(f) => {
                let _pkg = actions::install_mod(&f, self.config.mod_dir()).unwrap();
                utils::remove_file(&path)?;
                println!("Installed {}", url);
            }
            Err(e) => eprintln!("{}", e),
        }

        Ok(())
    }

    pub async fn install(
        &mut self,
        mod_names: Vec<String>,
        yes: bool,
        force: bool,
        global: bool,
    ) -> Result<()> {
        let target = if global {
            self.dirs.data_local_dir()
        } else {
            self.config.mod_dir()
        };

        let index = utils::update_index(target).await;
        let mut installed = utils::get_installed(target)?;
        let mut valid = vec![];
        for name in mod_names {
            let re = Regex::new(r"(.+)@?(v?\d.\d.\d)?").unwrap();

            if !re.is_match(&name) {
                println!("{} should be in 'ModName@1.2.3' format", name);
                continue;
            }

            let parts = re.captures(&name).unwrap();

            let base = index
                .iter()
                .find(|e| e.name.to_lowercase() == parts[1].to_lowercase())
                .ok_or_else(|| anyhow!("No such package {}", &parts[1]))?;

            if base.installed && !force {
                println!(
                    "Package \x1b[36m{}\x1b[0m version \x1b[36m{}\x1b[0m already installed",
                    base.name, base.version
                );
                continue;
            }

            utils::resolve_deps(&mut valid, base, &installed.mods, &index)?;
            valid.push(base);
        }

        //Gaurd against an empty list (maybe all the mods are already installed?)
        if valid.is_empty() {
            return Ok(());
        }

        let size: i64 = valid.iter().map(|f| f.file_size).sum();
        println!("Installing:\n");

        print!("\t");
        valid.iter().enumerate().for_each(|(i, f)| {
            if i > 0 && i % 5 == 0 {
                println!("\n");
                print!("\t");
            }
            print!(" \x1b[36m{}@{}\x1b[0m ", f.name, f.version);
        });
        println!("\n");

        let msg = format!(
            "Will download ~{:.2} MIB (compressed), okay? [Y/n]: ",
            size as f64 / 1_048_576f64
        );

        if !yes {
            if let Ok(line) = self.rl.readline(&msg) {
                if line.to_lowercase() == "n" {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }

        let mut downloaded = vec![];
        for base in valid {
            let name = &base.name;
            let path = self
                .dirs
                .cache_dir()
                .join(format!("{}_{}.zip", name, base.version));

            //would love to use this in the same if as the let but it's unstable so...
            if self.config.cache() {
                if let Some(f) = self.cache.check(&path) {
                    println!("Using cached version of {}", name);
                    downloaded.push(f);
                    continue;
                }
            }
            match actions::download_file(&base.url, path).await {
                Ok(f) => downloaded.push(f),
                Err(e) => eprintln!("{}", e),
            }
        }

        println!(
            "Extracting mod{} to {}",
            if downloaded.len() > 1 { "s" } else { "" },
            target.display()
        );
        for e in downloaded
            .iter()
            .map(|f| -> Result<()> {
                let pkg = actions::install_mod(f, target)?;
                installed.mods.insert(pkg.clone());
                self.cache.clean(&pkg.package_name, &pkg.version)?;
                println!("Installed {}!", pkg.package_name);
                Ok(())
            })
            .filter(|f| f.is_err())
        {
            println!("Encountered errors while installing mods:");
            println!("{}", e.unwrap_err());
        }
        utils::save_installed(target, &installed)?;
        Ok(())
    }

    pub fn remove(&self, mod_names: Vec<String>) -> Result<()> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        let valid: Vec<InstalledMod> = mod_names
            .iter()
            .filter_map(|f| {
                installed
                    .mods
                    .clone()
                    .iter()
                    .find(|e| e.package_name.trim().to_lowercase() == f.trim().to_lowercase())
                    .filter(|e| installed.mods.remove(e)).cloned()
            })
            .collect();

        let paths = valid.iter().flat_map(|f| f.flatten_paths()).collect();

        actions::uninstall(paths)?;
        utils::save_installed(self.config.mod_dir(), &installed)?;
        Ok(())
    }

    pub fn clear(&self, full: bool) -> Result<()> {
        if full {
            println!("Clearing cache files...");
        } else {
            println!("Clearing cached packages...");
        }
        utils::clear_cache(self.dirs.cache_dir(), full)?;
        println!("Done!");

        Ok(())
    }

    pub fn update_config(&mut self, mods_dir: Option<String>, cache: Option<bool>) -> Result<()> {
        if let Some(dir) = mods_dir {
            self.config.set_dir(&dir);
            println!("Set install directory to {}", dir);
        }

        if let Some(cache) = cache {
            self.config.set_cache(&cache);
            if cache {
                println!("Turned caching on");
            } else {
                println!("Turned caching off");
            }
        }

        config::save_config(self.dirs.config_dir(), &self.config)?;
        Ok(())
    }

    pub(crate) async fn search(&self, term: Vec<String>) -> Result<()> {
        let index = utils::update_index(self.config.mod_dir()).await;

        let print = |f: &Mod| {
            println!(
                " \x1b[92m{}@{}\x1b[0m   [{}]{}\n\n    {}",
                f.name,
                f.version,
                f.file_size_string(),
                if f.installed { "[installed]" } else { "" },
                f.desc
            );
            println!();
        };

        println!("Searching...");
        println!();
        if !term.is_empty() {
            index
                .iter()
                .filter(|f| {
                    //TODO: Use better method to match strings
                    term.iter().any(|e| {
                        f.name.to_lowercase().contains(&e.to_lowercase())
                            || f.desc.to_lowercase().contains(&e.to_lowercase())
                    })
                })
                .for_each(print);
        } else {
            index.iter().for_each(print)
        }
        Ok(())
    }

    pub(crate) fn disable(&self, mods: Vec<String>) -> Result<()> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        for m in mods {
            let m = m.to_lowercase();
            for i in installed.mods.clone().iter() {
                installed.mods.remove(i);
                let mut i = i.clone();
                if i.package_name.to_lowercase() == m {
                    for sub in i.mods.iter_mut() {
                        utils::disable_mod(sub)?;
                    }
                    println!("Disabled {}", m);
                } else {
                    for e in i.mods.iter_mut() {
                        if e.name.to_lowercase() == m {
                            utils::disable_mod(e)?;
                            println!("Disabled {}", m);
                        }
                    }
                }
                installed.mods.insert(i);
            }
        }
        utils::save_installed(self.config.mod_dir(), &installed)?;

        Ok(())
    }
    pub(crate) fn enable(&self, mods: Vec<String>) -> Result<()> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        for m in mods {
            let m = m.to_lowercase();
            for i in installed.mods.clone().iter() {
                installed.mods.remove(i);
                let mut i = i.clone();
                if i.package_name.to_lowercase() == m {
                    for sub in i.mods.iter_mut() {
                        utils::enable_mod(sub, self.config.mod_dir())?;
                    }
                    println!("Enabled {}", m);
                } else {
                    for e in i.mods.iter_mut() {
                        if e.name.to_lowercase() == m {
                            utils::enable_mod(e, self.config.mod_dir())?;
                            println!("Enabled {}", m);
                        }
                    }
                }
                installed.mods.insert(i);
            }
        }

        utils::save_installed(self.config.mod_dir(), &installed)?;
        Ok(())
    }

    pub(crate) fn include(&self, mods: Vec<String>, force: bool) -> Result<()> {
        let mut local = utils::get_installed(&self.local_target)?;
        let global = utils::get_installed(&self.global_target)?;
        for m in mods.iter() {
            if let Some(g) = global
                .mods
                .iter()
                .find(|e| e.package_name.trim().to_lowercase() == m.trim().to_lowercase())
            {
                if !force && local.linked.contains(g) {
                    println!("Mod '{}' already linked", m);
                    continue;
                }
                for m in g.mods.iter() {
                    self.link_dir(&m.path, &self.local_target.join(&m.name))
                        .context(format!(
                        "Unable to create link to {}... Does a file by that name already exist?",
                        self.local_target.join(&m.name).display()
                    ))?;
                }

                println!("Linked {}!", m);
                local.linked.insert(g.clone());
            } else {
                println!("No mod '{}' globally installed", m);
            }
        }

        utils::save_installed(&self.local_target, &local)?;

        Ok(())
    }

    fn link_dir(&self, original: &Path, target: &Path) -> Result<()> {
        debug!("Linking dir {} to {}", original.display(), target.display());
        for e in original.read_dir()? {
            let e = e?;
            if e.path().is_dir() {
                self.link_dir(&e.path(), &target.join(e.file_name()))?;
                continue;
            }

            let target = target.join(e.file_name());
            if let Some(p) = target.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            debug!(
                "Create hardlink {} -> {}",
                e.path().display(),
                target.display()
            );
            fs::hard_link(e.path(), &target).context("Failed to create hard link")?;
        }
        Ok(())
    }

    pub(crate) fn exclude(&self, mods: Vec<String>) -> Result<()> {
        let mut local = utils::get_installed(&self.local_target)?;
        for m in mods {
            if let Some(g) = local
                .linked
                .clone()
                .iter()
                .find(|e| e.package_name.trim().to_lowercase() == m.trim().to_lowercase())
            {
                for m in g.mods.iter() {
                    fs::remove_dir_all(self.local_target.join(&m.name))?;
                }

                println!("Removed link to {}", m);
                local.linked.remove(g);
            } else {
                warn!(
                    "Coudln't find link to {} in directory {}",
                    m,
                    self.local_target.display()
                );
                println!("No mod '{}' linked to current mod directory", m);
            }
        }

        utils::save_installed(&self.local_target, &local)?;

        Ok(())
    }
}
