use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub desc: String,
    pub deps: Vec<String>,
    pub file_size: i64,
    #[serde(skip)]
    pub installed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installed {
    pub package_name: String,
    pub version: String,
    pub path: Vec<PathBuf>,
    pub enabled: bool,
}

impl Mod {
    pub fn file_size_string(&self) -> String {
        if self.file_size / 1_000_000 >= 1 {
            let size = self.file_size as f64 / 1_048_576f64;

            format!("{:.2} MB", size)
        } else {
            let size = self.file_size as f64 / 1024f64;
            format!("{:.2} KB", size)
        }
    }
}

//impl Installed {
//    pub fn new(package_name: &str, version: &str, path: &str) -> Self {
//        Installed {
//            package_name: package_name.to_string(),
//            version: version.to_string(),
//            path: PathBuf::from(path),
//        }
//    }
//}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version_number: String,
    pub website_url: String,
    pub description: String,
    pub dependencies: Vec<String>,
}
