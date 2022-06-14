use std::{
    cmp::min,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use reqwest::Client;
use zip::ZipArchive;

use crate::{
    config::Config,
    model::{Installed, Manifest},
};

pub async fn download_file(url: &str, file_path: PathBuf) -> Result<File, String> {
    let client = Client::new();

    //send the request
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|_| (format!("Unable to GET from {}", url)))?;

    if !res.status().is_success() {
        return Err(format!("{} at URL {}", res.status(), url));
    }

    let file_size = res.content_length().ok_or(format!(
        "Unable to read content length of response from {}",
        url
    ))?;

    //setup the progress bar
    let pb = ProgressBar::new(file_size).with_style(ProgressStyle::default_bar().template(
        "{msg}\n{spinner:.green} [{duration}] [{bar:30.cyan}] {bytes}/{total_bytes} {bytes_per_sec}",
    ).progress_chars("=>-")).with_message(format!("Downloading {}", url));

    //start download in chunks
    let mut file = File::create(&file_path)
        .map_err(|_| (format!("Failed to create file {}", file_path.display())))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|_| ("Error downloading file :(".to_string()))?;
        file.write_all(&chunk)
            .map_err(|_| (format!("Error writing to file {}", file_path.display())))?;
        let new = min(downloaded + (chunk.len() as u64), file_size);
        downloaded = new;
        pb.set_position(new);
    }
    let finished = File::open(&file_path)
        .map_err(|_| (format!("Unable to open finished file {}", file_path.display())))?;

    pb.finish_with_message(format!(
        "Downloaded {}!",
        file_path.file_name().unwrap().to_string_lossy()
    ));
    Ok(finished)
}

pub fn uninstall(mods: Vec<PathBuf>) -> Result<(), String> {
    mods.iter().for_each(|p| {
        fs::remove_dir_all(p).expect("Unable to remove directory");
        println!("Removed {}", p.display());
    });
    Ok(())
}

pub fn install_mod(zip_file: &File, config: &Config) -> Result<Installed, String> {
    let mods_dir = config
        .mod_dir()
        .canonicalize()
        .map_err(|_| "Couldn't resolve mods directory path".to_string())?;
    let mut archive =
        ZipArchive::new(zip_file).map_err(|_| ("Unable to read zip archive".to_string()))?;

    //Get the package manifest
    let mut manifest = String::new();
    archive
        .by_name("manifest.json")
        .map_err(|_| "Couldn't find manifest file".to_string())?
        .read_to_string(&mut manifest)
        .unwrap();

    let manifest: Manifest =
        serde_json::from_str(&manifest).map_err(|_| "Unable to parse manifest".to_string())?;

    //Extract each file in the archive that is in the mods directory
    let mut deep = false;
    let mut path = OsString::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|_| ("Unable to get file from archive".to_string()))?;

        let mut out = file
            .enclosed_name()
            .ok_or_else(|| "Unable to get file name".to_string())?;

        if out.starts_with("mods/") {
            out = out.strip_prefix("mods/").unwrap();
        }

        if let Some(p) = out.parent() {
            if p.as_os_str() == OsStr::new("") {
                continue;
            }
        }
        let mp = mods_dir.join(&out);

        if !deep {
            if let Some(p) = out.iter().next() {
                path = p.to_owned();
                deep = true;
            }
        }

        if (*file.name()).ends_with('/') {
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
                .map_err(|_| (format!("Unable to open {}", &mp.display())))?;

            io::copy(&mut file, &mut outfile)
                .map_err(|_| ("Unable to write extracted file".to_string()))?;
        }
    }

    Ok(Installed {
        package_name: manifest.name,
        version: manifest.version_number,
        path: mods_dir.join(path),
        enabled: true,
    })
}
