use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub deps: Vec<Mod>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installed {
    pub package_name: String,
    pub version: String,
    pub path: String,
}

impl Installed {
    pub fn new(package_name: &str, version: &str, path: &str) -> Self {
        Installed {
            package_name: package_name.to_string(),
            version: version.to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version_number: String,
    pub website_url: String,
    pub description: String,
    pub dependencies: Vec<String>,
}
