use std::{
    cmp::min,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use reqwest::Client;
use zip::ZipArchive;

use crate::{
    api::model::SubMod,
    model::{InstalledMod, Manifest},
};

use log::{debug, error, trace};

use anyhow::{anyhow, Context, Result};

///URL to download
///file name to save to
pub async fn download_file(url: &str, file_path: PathBuf) -> Result<File> {
    let client = Client::new();

    //send the request
    let res = client
        .get(url)
        .send()
        .await
        .context(format!("Unable to GET from {}", url))?;

    if !res.status().is_success() {
        error!("Got bad response from thunderstore");
        error!("{:?}", res);
        return Err(anyhow!("{} at URL {}", res.status(), url));
    }

    let file_size = res
        .content_length()
        .ok_or_else(|| anyhow!("Unable to read content length of response from {}", url))?;
    debug!("file_size: {}", file_size);

    //setup the progress bar
    let pb = ProgressBar::new(file_size).with_style(ProgressStyle::default_bar().template(
        "{msg}\n{spinner:.green} [{duration}] [{bar:30.cyan}] {bytes}/{total_bytes} {bytes_per_sec}",
    )?.progress_chars("=>-")).with_message(format!("Downloading {}", url));

    //start download in chunks
    let mut file = File::create(&file_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    debug!("Starting download from {}", url);
    while let Some(item) = stream.next().await {
        let chunk = item.context("Error downloading file :(")?;
        file.write_all(&chunk)
            .context("Failed to write chunk to file")?;
        let new = min(downloaded + (chunk.len() as u64), file_size);
        downloaded = new;
        pb.set_position(new);
    }
    let finished = File::open(&file_path).context("Unable to open finished file")?;
    debug!("Finished download to {}", file_path.display());

    pb.finish_with_message(format!(
        "Downloaded {}!",
        file_path.file_name().unwrap().to_string_lossy()
    ));
    Ok(finished)
}

pub fn uninstall(mods: Vec<PathBuf>) -> Result<()> {
    for p in mods {
        if fs::remove_dir_all(&p).is_err() {
            //try removing a file too, just in case
            debug!("Removing dir failed, attempting to remove file...");
            fs::remove_file(&p).context(format!("Unable to remove directory {}", p.display()))?
        }
    }
    Ok(())
}

pub fn install_mod(zip_file: &File, target_dir: &Path) -> Result<InstalledMod> {
    debug!("Starting mod insall");
    let mods_dir = target_dir
        .canonicalize()
        .context("Couldn't resolve mods directory path")?;
    //Get the package manifest
    let mut manifest = String::new();
    //Extract mod to a temp directory so that we can easily see any sub-mods
    //This wouldn't be needed if the ZipArchive recreated directories, but oh well
    let temp_dir = mods_dir.join(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    );
    {
        let mut archive = ZipArchive::new(zip_file).context("Unable to read zip archive")?;

        archive
            .by_name("manifest.json")
            .context("Couldn't find manifest file")?
            .read_to_string(&mut manifest)
            .unwrap();

        fs::create_dir_all(&temp_dir).context("Unable to create temp directory")?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let out = temp_dir.join(file.enclosed_name().unwrap());

            if let Some(e) = out.extension() {
                if out.exists() && e == std::ffi::OsStr::new("cfg") {
                    debug!("Skipping existing config file {}", out.display());
                    continue;
                }
            }

            debug!("Extracting file to {}", out.display());
            if (*file.name()).ends_with('/') {
                trace!("Creating dir path in temp dir");
                fs::create_dir_all(&out)?;
                continue;
            } else if let Some(p) = out.parent() {
                trace!("Creating dir at {}", p.display());
                fs::create_dir_all(p)?;
            }
            trace!("Open file {} for writing", out.display());
            let mut outfile = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&out)?;
            io::copy(&mut file, &mut outfile).context("Unable to copy file")?;
        }
    }
    let mut mods = vec![];
    if let Ok(entries) = temp_dir.read_dir() {
        for e in entries {
            let e = e.unwrap();

            if e.path().is_dir() {
                //Get the path relative to the .papa.ron file
                // let m_path = e.path();
                // let m_path = m_path.strip_prefix(&temp_dir)?;
                if e.path().ends_with("mods") {
                    let mut dirs = e.path().read_dir().unwrap();
                    while let Some(Ok(e)) = dirs.next() {
                        let name = e.file_name();
                        let name = name.to_str().unwrap();
                        debug!("Add submod {}", name);
                        mods.push(SubMod::new(name, &Path::new("mods").join(name)));
                    }
                } else {
                    debug!(
                        "Add one submod {}",
                        e.path().file_name().unwrap().to_string_lossy()
                    );
                    mods.push(SubMod::new(
                        e.file_name().to_str().unwrap(),
                        &PathBuf::new(),
                    ));
                }
            }
        }
    }

    if mods.is_empty() {
        return Err(anyhow!("Couldn't find a directory to copy"));
    }

    // move the mod files from the temp dir to the real dir
    for p in mods.iter_mut() {
        let temp = temp_dir.join(&p.path);
        p.path = p.path.strip_prefix("mods")?.to_path_buf();
        let perm = mods_dir.join(&p.path);
        trace!(
            "Temp path: {} | Perm path: {}",
            temp.display(),
            perm.display()
        );

        if perm.exists() {
            fs::remove_dir_all(&perm)?;
        }
        fs::rename(temp, perm)?;
    }

    let manifest: Manifest = serde_json::from_str(&manifest).context("Unable to parse manifest")?;

    fs::remove_dir_all(&temp_dir).context("Unable to remove temp directory")?;

    Ok(InstalledMod {
        package_name: manifest.name,
        version: manifest.version_number,
        mods,
        depends_on: vec![],
        needed_by: vec![],
    })
}
