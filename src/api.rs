use crate::model::Mod;
use reqwest::{Client, RequestBuilder};
use serde_json::Value;

pub async fn get_package_index() -> Result<Vec<Mod>, String> {
    let client = Client::new();
    let raw = client
        .get("https://northstar.thunderstore.io/c/northstar/api/v1/package/")
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|_| "Error making request to update package index".to_string())?;
    if raw.status().is_success() {
        let parsed: Value = serde_json::from_str(&raw.text().await.unwrap())
            .map_err(|_| "Unable to parse response body".to_string())?;
        if let Some(v) = map_response(parsed) {
            Ok(v)
        } else {
            Err("Response body was malformed?".to_string())
        }
    } else {
        Err(format!("{}", raw.status().as_str()))
    }
}

fn map_response(res: Value) -> Option<Vec<Mod>> {
    match res {
        Value::Array(v) => Some(
            v.into_iter()
                .map(|e| {
                    let name = format!(
                        "{}.{}",
                        e["owner"].as_str().unwrap(),
                        e["name"].as_str().unwrap()
                    );
                    let latest = e["versions"][0].clone();
                    let version = latest["version_number"].as_str().unwrap().to_string();
                    let url = latest["download_url"].as_str().unwrap().to_string();
                    let deps = if let Value::Array(d) = &latest["dependencies"] {
                        d.into_iter()
                            .map(|e| e.as_str().unwrap().to_string())
                            .collect()
                    } else {
                        vec![]
                    };

                    Mod {
                        name,
                        version,
                        url,
                        deps,
                    }
                })
                .collect(),
        ),
        _ => None,
    }
}
