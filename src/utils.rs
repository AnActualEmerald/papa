use std::{
    cmp::min,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use futures_util::{stream::Zip, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use zip::ZipArchive;

pub async fn download_file(url: String, file_path: PathBuf) -> Result<File, String> {
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
    Ok(file)
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

pub fn install_mod(zip_file: &File, mods_dir: &Path) -> Result<(), String> {
    let mut archive = ZipArchive::new(zip_file).or(Err(format!("Unable to read zip archive")))?;
    // let outfile =

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .or(Err(format!("Unable to get file from archive")))?;
        let out = file
            .enclosed_name()
            .ok_or(format!("Unable to get file name"))?;

        if out.starts_with("mods/") {
            let mp = mods_dir.join(out);

            if (*file.name()).ends_with("/") {
                fs::create_dir_all(&mp).unwrap();
            } else {
                if let Some(p) = mp.parent() {
                    fs::create_dir_all(&p).unwrap();
                }

                let mut outfile = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&mp)
                    .or(Err(format!("Unable to open {}", &mp.display())))?;

                io::copy(&mut file, &mut outfile)
                    .or(Err(format!("Unable to write extracted file")))?;

                println!("Write file {}", &mp.display());
            }
        }
    }

    Ok(())
}
