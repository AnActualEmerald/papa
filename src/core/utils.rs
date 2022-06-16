use crate::api;
use crate::model;
use crate::model::Installed;
use crate::model::Mod;
use directories::ProjectDirs;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub async fn update_index(path: &Path) -> Vec<model::Mod> {
    print!("Updating package index...");
    let mut index = api::get_package_index().await.unwrap().to_vec();
    //        save_file(&dirs.cache_dir().join("index.ron"), index)?;
    let installed = get_installed(path).unwrap();
    for e in index.iter_mut() {
        e.installed = installed
            .iter()
            .any(|f| f.package_name == e.name && f.version == e.version);
    }
    println!(" Done!");
    index
}

pub fn get_installed(path: &Path) -> Result<Vec<Installed>, String> {
    let path = path.join(".papa.ron");
    if path.exists() {
        let raw = fs::read_to_string(path)
            .map_err(|_| "Unable to read installed packages".to_string())?;
        Ok(ron::from_str(&raw).map_err(|_| "Unable to parse installed packages".to_string())?)
    } else {
        if let Some(p) = path.parent() {
            if !p.exists() {
                fs::create_dir_all(p)
                    .map_err(|_| format!("Unable to create directory at {}", p.display()))?;
            }
        }
        File::create(path)
            .map_err(|_| "Unable to create installed package index".to_string())?
            .write_all(ron::to_string(&Vec::<Installed>::new()).unwrap().as_bytes())
            .unwrap();

        Ok(vec![])
    }
}

pub fn save_installed(path: &Path, installed: Vec<Installed>) -> Result<(), String> {
    let path = path.join(".papa.ron");

    save_file(&path, ron::to_string(&installed).unwrap())?;

    Ok(())
}

pub fn check_cache(path: &Path) -> Option<File> {
    let opt = OpenOptions::new().read(true).open(path);
    if let Ok(f) = opt {
        Some(f)
    } else {
        None
    }
}

pub fn ensure_dirs(dirs: &ProjectDirs) {
    fs::create_dir_all(dirs.cache_dir()).unwrap();
    fs::create_dir_all(dirs.config_dir()).unwrap();
}

pub fn remove_file(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|_| format!("Unable to remove file {}", path.display()))
}

//    pub fn remove_dir(dir: &Path) -> Result<(), String> {
//        fs::remove_dir_all(dir)
//            .map_err(|_| format!("Unable to remove directory {}", dir.display()))?;
//
//        Ok(())
//    }

pub fn clear_cache(dir: &Path, force: bool) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|_| format!("unable to read directory {}", dir.display()))?
    {
        let path = entry
            .map_err(|_| "Error reading directory entry".to_string())?
            .path();

        if path.is_dir() {
            clear_cache(&path, force)?;
            fs::remove_dir(&path)
                .map_err(|_| format!("Unable to remove directory {}", path.display()))?;
        } else if path.ends_with(".zip") {
            fs::remove_file(&path)
                .map_err(|_| format!("Unable to remove file {}", path.display()))?;
        } else {
            fs::remove_file(&path)
                .map_err(|_| format!("Unable to remove file {}", path.display()))?;
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

pub fn save_file(file: &Path, data: String) -> Result<(), String> {
    fs::write(file, data.as_bytes())
        .map_err(|_| format!("Unable to write file {}", file.display()))?;
    Ok(())
}

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
pub fn resolve_deps<'a>(
    valid: &mut Vec<&'a Mod>,
    base: &'a Mod,
    installed: &'a Vec<Installed>,
    index: &'a Vec<Mod>,
) -> Result<(), String> {
    for dep in &base.deps {
        let dep_name = dep.split("-").collect::<Vec<&str>>()[1];
        if !installed.iter().any(|e| e.package_name == dep_name) {
            if let Some(d) = index.iter().find(|f| f.name == dep_name) {
                resolve_deps(valid, d, installed, index)?;
                valid.push(d);
            } else {
                return Err(format!(
                    "Unable to resolve dependency {} of {}",
                    dep, base.name
                ));
            }
        }
    }
    Ok(())
}
