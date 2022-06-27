use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

use log::debug;
use zip::ZipArchive;

use crate::api::model::Mod;

use super::{actions, config, error::ScorchError, utils, Core};

impl Core {
    pub(crate) async fn init_northstar(&mut self, game_path: &Path) -> Result<(), ScorchError> {
        let version = self.install_northstar(game_path).await?;

        self.config.game_path = game_path.to_path_buf();
        self.config.nstar_version = Some(version);
        self.config
            .set_dir(game_path.join("R2Northstar").join("mods").to_str().unwrap());

        println!("Set mod directory to {}", self.config.mod_dir().display());
        config::save_config(self.dirs.config_dir(), &self.config)?;

        Ok(())
    }

    #[cfg(feature = "launcher")]
    pub fn start_northstar(&self) -> Result<(), String> {
        let game = self.config.game_path.join("NorthstarLauncher.exe");

        std::process::Command::new(game)
            // .stderr(Stdio::null())
            // .stdin(Stdio::null())
            // .stdout(Stdio::null())
            .spawn()
            .expect("Unable to start game");

        Ok(())
    }

    ///Update N* at the path that was initialized
    ///
    ///Returns OK if the path isn't set, but notifies the user
    pub async fn update_northstar(&mut self) -> Result<(), ScorchError> {
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

            self.do_install(nmod, &self.config.game_path).await?;
            self.config.nstar_version = Some(nmod.version.clone());
            config::save_config(self.dirs.config_dir(), &self.config)?;

            Ok(())
        } else {
            println!("Only Northstar installations done with `papa northstar init` can be updated this way");
            Ok(())
        }
    }

    ///Install N* to the provided path
    ///
    ///Returns the version that was installed
    pub async fn install_northstar(&self, game_path: &Path) -> Result<String, ScorchError> {
        let index = utils::update_index(self.config.mod_dir()).await;
        let nmod = index
            .iter()
            .find(|f| f.name.to_lowercase() == "northstar")
            .ok_or("Couldn't find Northstar on thunderstore???")?;

        self.do_install(nmod, game_path).await?;

        Ok(nmod.version.clone())
    }

    ///Install N* from the provided mod
    ///
    ///Checks cache, else downloads the latest version
    async fn do_install(&self, nmod: &Mod, game_path: &Path) -> Result<(), ScorchError> {
        let filename = format!("northstar-{}.zip", nmod.version);
        let nfile = if let Some(f) = utils::check_cache(&self.dirs.cache_dir().join(&filename)) {
            println!("Using cached verision of Northstar@{}...", nmod.version);
            f
        } else {
            actions::download_file(&nmod.url, self.dirs.cache_dir().join(&filename)).await?
        };
        println!("Extracting Northstar...");
        self.extract(nfile, game_path)?;
        // println!("Copying Files...");
        // self.move_files(game_path, extracted)?;
        println!("Done!");

        Ok(())
    }

    // fn move_files(&self, game_path: &Path, extracted: PathBuf) -> Result<(), String> {
    //     let nstar = extracted.join("Northstar");

    //     self.copy_dirs(&nstar, &nstar, game_path)?;
    //     println!("Northstar installed sucessfully!");
    //     println!("Cleaning up...");

    //     fs::remove_dir_all(nstar).map_err(|e| format!("Unable to remove temp directory {}", e))?;

    //     Ok(())
    // }

    // ///Recurses through a directory and moves each entry to the target, keeping the directory structure
    // fn copy_dirs(&self, root: &Path, dir: &Path, target: &Path) -> Result<(), String> {
    //     for entry in (dir
    //         .read_dir()
    //         .map_err(|_| "Unable to read directory".to_string())?)
    //     .flatten()
    //     {
    //         if entry.path().is_dir() {
    //             self.copy_dirs(root, &entry.path(), target)?;
    //             continue;
    //         } else if let Some(p) = entry.path().parent() {
    //             let target = target.join(p.strip_prefix(&root).unwrap());
    //             debug!("Create dir {}", target.display());
    //             fs::create_dir_all(target).map_err(|_| "Failed to create directory".to_string())?;
    //         }
    //         let target = target.join(entry.path().strip_prefix(root).unwrap());
    //         debug!(
    //             "Moving file {} to {}",
    //             entry.path().display(),
    //             target.display()
    //         );
    //         fs::rename(entry.path(), target).map_err(|e| format!("Unable to move file: {}", e))?;
    //     }

    //     Ok(())
    // }

    ///Extract N* zip file to target game path
    fn extract(&self, zip_file: File, target: &Path) -> Result<(), ScorchError> {
        let mut archive =
            ZipArchive::new(&zip_file).map_err(|_| "Unable to open zip archive".to_string())?;
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).unwrap();

            //skip any files that have been excluded
            if let Some(n) = f.enclosed_name() {
                if self.config.exclude.iter().any(|e| Path::new(e) == n) {
                    continue;
                }
            } else {
                return Err(format!("Unable to read name of compressed file {}", f.name()).into());
            }

            //This should work fine for N* because the dir structure *should* always be the same
            if f.enclosed_name().unwrap().starts_with("Northstar") {
                let out = target.join(
                    f.enclosed_name()
                        .unwrap()
                        .strip_prefix("Northstar")
                        .unwrap(),
                );

                if (*f.name()).ends_with('/') {
                    debug!("Create directory {}", f.name());
                    fs::create_dir_all(target.join(f.name()))
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

                debug!("Write file {}", out.display());

                io::copy(&mut f, &mut outfile)
                    .map_err(|_| "Unable to write to file".to_string())?;
            }
        }

        Ok(())
    }
}
