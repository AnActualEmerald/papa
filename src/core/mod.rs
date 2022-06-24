pub mod actions;
pub mod config;
#[cfg(feature = "northstar")]
pub mod northstar;

pub(crate) mod utils;

mod error;

// pub use error::*;

use directories::ProjectDirs;
use regex::Regex;
use rustyline::Editor;

use self::config::Config;
use crate::api;
use crate::api::model::{self, InstalledMod};

pub struct Core {
    pub config: Config,
    dirs: ProjectDirs,
    rl: Editor<()>,
}

impl Core {
    pub fn new(config: Config, dirs: ProjectDirs, rl: Editor<()>) -> Self {
        utils::ensure_dirs(&dirs);
        Core { config, dirs, rl }
    }

    pub async fn update(&mut self, yes: bool) -> Result<(), String> {
        print!("Updating package index...");
        let index = &api::get_package_index().await?;
        println!(" Done!");
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        let outdated: Vec<&model::Mod> = index
            .iter()
            .filter(|e| {
                installed.mods.iter().any(|i| {
                    i.package_name.trim() == e.name.trim() && i.version.trim() != e.version.trim()
                })
            })
            .collect();

        if outdated.is_empty() {
            println!("Already up to date!");
        } else {
            let size: i64 = outdated.iter().map(|f| f.file_size).sum();

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
            downloaded.into_iter().for_each(|f| {
                let pkg = actions::install_mod(&f, &self.config).unwrap();
                if let Some(i) = installed
                    .mods
                    .iter()
                    .position(|e| e.package_name == pkg.package_name)
                {
                    installed.mods.get_mut(i).unwrap().version = pkg.version;
                    installed.mods.get_mut(i).unwrap().mods = pkg.mods;
                    println!("Updated {}", pkg.package_name);
                }
            });
            utils::save_installed(self.config.mod_dir(), &installed)?;
        }
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

    pub fn list(&self) -> Result<(), String> {
        let mods = utils::get_installed(self.config.mod_dir())?.mods;
        if !mods.is_empty() {
            println!("Installed mods:");
            mods.into_iter().for_each(|m| {
                let disabled = if !m.any_disabled() || m.mods.len() > 1 {
                    ""
                } else {
                    "[disabled]"
                };
                println!(
                    " \x1b[92m{}@{}\x1b[0m {}",
                    m.package_name, m.version, disabled
                );
                if m.mods.len() > 1 {
                    for (i, e) in m.mods.iter().enumerate() {
                        let character = if i + 1 < m.mods.len() { "├" } else { "└" };
                        let disabled = if e.disabled() { "[disabled]" } else { "" };
                        println!(
                            "   \x1b[92m{}─\x1b[0m \x1b[0;96m{}\x1b[0m {}",
                            character, e.name, disabled
                        );
                    }
                }
            });
        } else {
            println!("No mods currently installed");
        }

        Ok(())
    }

    pub async fn install_from_url(&self, url: String) -> Result<(), String> {
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
                let _pkg = actions::install_mod(&f, &self.config).unwrap();
                utils::remove_file(&path)?;
                println!("Installed {}", url);
            }
            Err(e) => eprintln!("{}", e),
        }

        Ok(())
    }

    pub async fn install(&mut self, mod_names: Vec<String>, yes: bool) -> Result<(), String> {
        let index = utils::update_index(self.config.mod_dir()).await;
        let mut installed = utils::get_installed(self.config.mod_dir())?;
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
                .ok_or_else(|| {
                    println!("Couldn't find package {}", name);
                    "No such package".to_string()
                })?;

            if base.installed {
                println!(
                    "Package \x1b[36m{}\x1b[0m version \x1b[36m{}\x1b[0m already installed",
                    base.name, base.version
                );
                continue;
            }

            utils::resolve_deps(&mut valid, base, &installed.mods, &index)?;
            valid.push(base);
        }

        let size: i64 = valid.iter().map(|f| f.file_size).sum();
        println!("Installing:\n");

        print!("\t");
        valid
            .iter()
            .for_each(|f| print!("\x1b[36m{}@{}\x1b[0m ", f.name, f.version));
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
            let path = self.dirs.cache_dir().join(format!("{}.zip", name));

            //would love to use this in the same if as the let but it's unstable so...
            if self.config.cache() {
                if let Some(f) = utils::check_cache(&path) {
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
            self.config.mod_dir().display()
        );
        if downloaded
            .iter()
            .map(|f| -> Result<(), String> {
                let pkg = actions::install_mod(f, &self.config)?;
                installed.mods.push(pkg.clone());
                println!("Installed {}", pkg.package_name);
                Ok(())
            })
            .filter(|f| f.is_err())
            .count()
            > 0
        {
            return Err("Errors while installing".to_string());
        }

        utils::save_installed(self.config.mod_dir(), &installed)?;
        Ok(())
    }

    pub fn remove(&self, mod_names: Vec<String>) -> Result<(), String> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        let valid: Vec<InstalledMod> = mod_names
            .iter()
            .filter_map(|f| {
                installed
                    .mods
                    .iter()
                    .position(|e| e.package_name.trim().to_lowercase() == f.trim().to_lowercase())
                    .map(|i| installed.mods.swap_remove(i))
            })
            .collect();

        let paths = valid.iter().flat_map(|f| f.flatten_paths()).collect();

        actions::uninstall(paths)?;
        utils::save_installed(self.config.mod_dir(), &installed)?;
        Ok(())
    }

    pub fn clear(&self, full: bool) -> Result<(), String> {
        if full {
            println!("Clearing cache files...");
        } else {
            println!("Clearing cached packages...");
        }
        utils::clear_cache(self.dirs.cache_dir(), full)?;
        println!("Done!");

        Ok(())
    }

    pub fn update_config(
        &mut self,
        mods_dir: Option<String>,
        cache: Option<bool>,
    ) -> Result<(), String> {
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

    pub(crate) async fn search(&self, term: Vec<String>) -> Result<(), String> {
        let index = utils::update_index(self.config.mod_dir()).await;
        println!("Searching...");
        println!();
        index
            .iter()
            .filter(|f| {
                term.iter().any(|e| {
                    f.name.to_lowercase().contains(&e.to_lowercase())
                        || f.desc.to_lowercase().contains(&e.to_lowercase())
                })
            })
            .for_each(|f| {
                println!(
                    " \x1b[92m{}@{}\x1b[0m   [{}]{}\n\n    {}",
                    f.name,
                    f.version,
                    f.file_size_string(),
                    if f.installed { "[installed]" } else { "" },
                    f.desc
                );
                println!();
            });

        Ok(())
    }

    pub(crate) fn disable(&self, mods: Vec<String>) -> Result<(), String> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        for m in mods {
            let m = m.to_lowercase();
            for i in installed.mods.iter_mut() {
                if i.package_name.to_lowercase() == m {
                    utils::disable_mod(&mut i.mods[0])?;
                    println!("Disabled {}", m);
                } else {
                    for e in i.mods.iter_mut() {
                        if e.name.to_lowercase() == m {
                            utils::disable_mod(e)?;
                            println!("Disabled {}", m);
                        }
                    }
                }
            }
        }
        utils::save_installed(self.config.mod_dir(), &installed)?;

        Ok(())
    }
    pub(crate) fn enable(&self, mods: Vec<String>) -> Result<(), String> {
        let mut installed = utils::get_installed(self.config.mod_dir())?;
        for m in mods {
            let m = m.to_lowercase();
            for i in installed.mods.iter_mut() {
                if i.package_name.to_lowercase() == m {
                    utils::enable_mod(&mut i.mods[0], self.config.mod_dir())?;
                    println!("Enabled {}", m);
                } else {
                    for e in i.mods.iter_mut() {
                        if e.name.to_lowercase() == m {
                            utils::enable_mod(e, self.config.mod_dir())?;
                            println!("Enabled {}", m);
                        }
                    }
                }
            }
        }

        utils::save_installed(self.config.mod_dir(), &installed)?;
        Ok(())
    }
}
