use std::{cmp::min, fs::File, io::Write, path::PathBuf};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub async fn download_file(url: String, file_path: PathBuf) -> Result<(), String> {
    let client = Client::new();

    //send the request
    let res = client
        .get(&url)
        .send()
        .await
        .or(Err(format!("Unable to GET from {}", &url)))?;

    let file_size = res.content_length().ok_or(format!(
        "Unable to read content length of response from {}",
        url
    ))?;

    //setup the progress bar
    let pb = ProgressBar::new(file_size).with_style(ProgressStyle::default_bar().template(
        "{msg}\n{spinner:.green} [{duration}] {wide_bar:.cyan} {bytes}/{total_bytes} {bytes_per_sec}",
    ).progress_chars("#>-"));

    //start download in chunks
    let mut file = File::create(&file_path).or(Err(format!(
        "Failed to create file {}",
        file_path.display()
    )))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error downloading file :(")))?;
        file.write_all(&chunk).or(Err(format!(
            "Error writing to file {}",
            file_path.display()
        )))?;
        let new = min(downloaded + (chunk.len() as u64), file_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {}", url));
    Ok(())
}

//supposing the mod name is formatted like Author.Mod@v1.0.0
pub fn parse_mod_name(name: &str) -> Option<String> {
    let parts = name.split_once(".")?;
    let author = parts.0;
    let parts = parts.1.split_once("@")?;
    let m_name = parts.0;
    let ver = parts.1.replace("v", "");

    Some(format!("/{}/{}/{}", author, m_name, ver))
}
