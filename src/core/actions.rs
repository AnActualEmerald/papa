use std::{
    cmp::min,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    time::SystemTime,
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use reqwest::Client;
use zip::ZipArchive;

use crate::{
    config::Config,
    model::{Installed, Manifest},
};

use log::{debug, error, trace};

pub async fn download_file(url: &str, file_path: PathBuf) -> Result<File, String> {
    let client = Client::new();

    //send the request
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|_| (format!("Unable to GET from {}", url)))?;

    if !res.status().is_success() {
        error!("Got bad response from thunderstore");
        error!("{:?}", res);
        return Err(format!("{} at URL {}", res.status(), url));
    }

    let file_size = res.content_length().ok_or(format!(
        "Unable to read content length of response from {}",
        url
    ))?;
    debug!("file_size: {}", file_size);

    //setup the progress bar
    let pb = ProgressBar::new(file_size).with_style(ProgressStyle::default_bar().template(
        "{msg}\n{spinner:.green} [{duration}] [{bar:30.cyan}] {bytes}/{total_bytes} {bytes_per_sec}",
    ).progress_chars("=>-")).with_message(format!("Downloading {}", url));

    //start download in chunks
    let mut file = File::create(&file_path)
        .map_err(|_| (format!("Failed to create file {}", file_path.display())))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    debug!("Starting download from {}", url);
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|_| ("Error downloading file :(".to_string()))?;
        file.write_all(&chunk).map_err(|e| {
            error!("Error while writing download chunk to file");
            error!("{}", e);
            format!("Error writing to file {}", file_path.display())
        })?;
        let new = min(downloaded + (chunk.len() as u64), file_size);
        downloaded = new;
        pb.set_position(new);
    }
    let finished = File::open(&file_path)
        .map_err(|_| (format!("Unable to open finished file {}", file_path.display())))?;
    debug!("Finished download to {}", file_path.display());

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
    debug!("Starting mod insall");
    let mods_dir = config
        .mod_dir()
        .canonicalize()
        .map_err(|_| "Couldn't resolve mods directory path".to_string())?;
    //Get the package manifest
    let mut manifest = String::new();
    let temp_dir = mods_dir.join(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    );
    {
        let mut archive =
            ZipArchive::new(zip_file).map_err(|_| ("Unable to read zip archive".to_string()))?;

        archive
            .by_name("manifest.json")
            .map_err(|_| "Couldn't find manifest file".to_string())?
            .read_to_string(&mut manifest)
            .unwrap();

        fs::create_dir_all(&temp_dir).map_err(|e| {
            error!("Unable to create temp directory");
            error!("{}", e);
            "Unable to create temp directory".to_string()
        })?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let out = temp_dir.join(&file.enclosed_name().unwrap());
            debug!("Extracting file to {}", out.display());
            if (*file.name()).ends_with("/") {
                fs::create_dir_all(&out).unwrap();
                continue;
            } else if let Some(p) = out.parent() {
                fs::create_dir_all(&p).unwrap();
            }
            let mut outfile = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&out)
                .unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    let mut paths = vec![];
    if let Ok(entries) = temp_dir.read_dir() {
        for e in entries {
            let e = e.unwrap();

            if e.path().is_dir() {
                if e.path().ends_with("mods") {
                    let mut mods = e.path().read_dir().unwrap();
                    while let Some(Ok(e)) = mods.next() {
                        paths.push(e.path());
                    }
                } else {
                    paths.push(e.path());
                }
            }
        }
    }

    if paths.len() == 0 {
        error!("Didn't find any directories in extracted archive");
        return Err("Couldn't find a directory to copy".to_string());
    }

    paths
        .iter()
        .map(|p| (p, mods_dir.join(p.file_name().unwrap())))
        .for_each(|p| {
            fs::remove_dir_all(&p.1).unwrap();
            fs::rename(p.0, p.1).unwrap();
        });

    let manifest: Manifest =
        serde_json::from_str(&manifest).map_err(|_| "Unable to parse manifest".to_string())?;

    fs::remove_dir_all(&temp_dir).map_err(|e| {
        error!("Unable to remove temp directory");
        error!("{}", e);
        "Unable to remove temp directory".to_string()
    })?;

    Ok(Installed {
        package_name: manifest.name,
        version: manifest.version_number,
        path: paths
            .iter()
            .map(|p| mods_dir.join(p.file_name().unwrap()))
            .collect(),
        enabled: true,
    })
}
