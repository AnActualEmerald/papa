use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use log::debug;
use zip::ZipArchive;

use crate::api::model::Mod;

use super::{actions, config, utils, Core};

impl Core {
    pub async fn update_northstar(&mut self) -> Result<(), String> {
        if let Some(current) = &self.config.nstar_version {
            let index = utils::update_index(self.config.mod_dir()).await;
            let nmod = index
                .iter()
                .find(|f| f.name.to_lowercase() == "northstar")
                .ok_or("Couldn't find Northstar on thunderstore???")?;

            if nmod.version == *current {
                println!("Northstar is already up to date ({})", current);
                return Ok(());
            }

            if let Ok(s) = self.rl.readline(&format!(
                "Update Northstar to version {}? [Y/n]",
                nmod.version
            )) {
                if &s.to_lowercase() == "n" {
                    return Ok(());
                }
            }

            self.do_install(&nmod, &self.config.game_path).await?;
            self.config.nstar_version = Some(nmod.version.clone());
            config::save_config(self.dirs.config_dir(), &self.config)?;

            Ok(())
        } else {
            println!("Only Northstar installations done with `papa northstar init` can be updated this way");
            Ok(())
        }
    }

    pub async fn install_northstar(&self, game_path: &Path) -> Result<String, String> {
        let index = utils::update_index(self.config.mod_dir()).await;
        let nmod = index
            .iter()
            .find(|f| f.name.to_lowercase() == "northstar")
            .ok_or("Couldn't find Northstar on thunderstore???")?;

        self.do_install(&nmod, game_path).await?;

        Ok(nmod.version.clone())
    }

    async fn do_install(&self, nmod: &Mod, game_path: &Path) -> Result<(), String> {
        let filename = format!("northstar-{}.zip", nmod.version);
        let nfile = if let Some(f) = utils::check_cache(&self.dirs.cache_dir().join(&filename)) {
            println!("Using cached verision of Northstar@{}...", nmod.version);
            f
        } else {
            actions::download_file(&nmod.url, self.dirs.cache_dir().join(&filename)).await?
        };
        println!("Extracting Northstar...");
        let extracted = self.extract(nfile)?;
        println!("Copying Files...");
        self.move_files(game_path, extracted)?;
        println!("Done!");

        Ok(())
    }

    fn move_files(&self, game_path: &Path, extracted: PathBuf) -> Result<(), String> {
        let nstar = extracted.join("Northstar");

        self.copy_dirs(&nstar, &nstar, game_path)?;
        println!("Northstar installed sucessfully!");
        println!("Cleaning up...");

        fs::remove_dir_all(nstar).map_err(|e| format!("Unable to remove temp directory {}", e))?;

        Ok(())
    }

    ///Recurses through a directory and moves each entry to the target, keeping the directory structure
    fn copy_dirs(&self, root: &PathBuf, dir: &PathBuf, target: &Path) -> Result<(), String> {
        for f in dir
            .read_dir()
            .map_err(|_| "Unable to read directory".to_string())?
        {
            if let Ok(entry) = f {
                if entry.path().is_dir() {
                    self.copy_dirs(root, &entry.path(), target)?;
                    continue;
                } else if let Some(p) = entry.path().parent() {
                    let target = target.join(p.strip_prefix(&root).unwrap());
                    debug!("Create dir {}", target.display());
                    fs::create_dir_all(target)
                        .map_err(|_| "Failed to create directory".to_string())?;
                }
                let target = target.join(entry.path().strip_prefix(root).unwrap());
                debug!(
                    "Moving file {} to {}",
                    entry.path().display(),
                    target.display()
                );
                fs::rename(entry.path(), target)
                    .map_err(|e| format!("Unable to move file: {}", e))?;
            }
        }

        Ok(())
    }

    fn extract(&self, zip_file: File) -> Result<PathBuf, String> {
        let mut archive =
            ZipArchive::new(&zip_file).map_err(|_| "Unable to open zip archive".to_string())?;
        let npath = self.dirs.cache_dir().join("northstar");
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).unwrap();
            let out = npath.join(f.name());

            if (*f.name()).ends_with("/") {
                fs::create_dir_all(npath.join(f.name()))
                    .map_err(|_| "Unable to create directory".to_string())?;
                continue;
            } else if let Some(p) = out.parent() {
                fs::create_dir_all(&p).map_err(|_| "Unable to create directory".to_string())?;
            }

            let mut outfile = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&out)
                .unwrap();

            io::copy(&mut f, &mut outfile).map_err(|_| "Unable to write to file".to_string())?;
        }

        Ok(npath)
    }
}
