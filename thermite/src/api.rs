use reqwest::Client;
use serde_json::Value;

use crate::{error::ThermiteError, model::Mod};

pub async fn get_package_index() -> Result<Vec<Mod>, ThermiteError> {
    let client = Client::new();
    let raw = client
        .get("https://northstar.thunderstore.io/c/northstar/api/v1/package/")
        .header("accept", "application/json")
        .send()
        .await?;
    if raw.status().is_success() {
        let parsed: Value = serde_json::from_str(&raw.text().await.unwrap())?;
        let index = map_response(&parsed).ok_or_else(|| {
            ThermiteError::MiscError("Thunderstore response was malformed".into())
        })?;

        Ok(index)
    } else {
        Err(ThermiteError::MiscError(raw.status().to_string()))
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
