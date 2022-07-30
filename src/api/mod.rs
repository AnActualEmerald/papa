use reqwest::Client;
use serde_json::Value;

pub mod model;

use model::Mod;

use anyhow::{anyhow, Context, Result};

pub async fn get_package_index() -> Result<Vec<Mod>> {
    let client = Client::new();
    let raw = client
        .get("https://northstar.thunderstore.io/c/northstar/api/v1/package/")
        .header("accept", "application/json")
        .send()
        .await
        .context("Error making request to update package index")?;
    if raw.status().is_success() {
        let parsed: Value = serde_json::from_str(&raw.text().await.unwrap())
            .context("Unable to parse response body")?;
        map_response(&parsed)
            .ok_or_else(|| anyhow!("{}", serde_json::to_string(&parsed).unwrap()))
            .context("Response body was malformed?")
    } else {
        Err(anyhow!("{}", raw.status().as_str()))
    }
}

fn map_response(res: &Value) -> Option<Vec<Mod>> {
    match res {
        Value::Array(v) => Some(
            v.iter()
                .map(|e| {
                    let name = e["name"].as_str().unwrap().to_string();
                    let latest = e["versions"][0].clone();
                    let version = latest["version_number"].as_str().unwrap().to_string();
                    let url = latest["download_url"].as_str().unwrap().to_string();
                    let file_size = latest["file_size"].as_i64().unwrap();
                    let desc = latest["description"].as_str().unwrap().to_string();
                    let deps = if let Value::Array(d) = &latest["dependencies"] {
                        //TODO: Support dependencies
                        d.iter()
                            .map(|e| e.as_str().unwrap().to_string())
                            .filter(|e| !e.starts_with("northstar-Northstar")) //Don't try to install northstar for any mods that "depend" on it
                            .collect()
                    } else {
                        vec![]
                    };

                    Mod {
                        name,
                        version,
                        url,
                        deps,
                        desc,
                        file_size,
                        installed: false,
                        global: false,
                        upgradable: false,
                    }
                })
                .collect(),
        ),
        _ => None,
    }
}
