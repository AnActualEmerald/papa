use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub deps: Vec<String>,
    pub file_size: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installed {
    pub package_name: String,
    pub version: String,
    pub path: PathBuf,
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
