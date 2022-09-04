use std::{
    cmp::min,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::error::ThermiteError;

use futures_util::StreamExt;
use indicatif::ProgressBar;

use reqwest::Client;
use zip::ZipArchive;

use crate::{
    model::SubMod,
    model::{LocalMod, Manifest},
};

use log::{debug, error, trace};

/// Download a file and update a progress bar
/// # Params
/// * url - URL to download from
/// * file_path - Full path to save file to
/// * pb - `ProgressBar` to update
pub async fn download_file_with_progress(
    url: &str,
    file_path: impl AsRef<Path>,
    pb: impl Into<Option<ProgressBar>>,
) -> Result<File, ThermiteError> {
    let client = Client::new();
    let pb = pb.into();
    let file_path = file_path.as_ref();

    //send the request
    let res = client.get(url).send().await?;

    if !res.status().is_success() {
        error!("Got bad response from thunderstore");
        error!("{:?}", res);
        return Err(ThermiteError::MiscError(format!(
            "Thunderstore returned error: {:#?}",
            res
        )));
    }

    let file_size = res
        .content_length()
        .ok_or_else(|| ThermiteError::MiscError("Missing content length header".into()))?;
    debug!("Downloading file size: {}", file_size);

    //start download in chunks
    let mut file = File::create(file_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    debug!("Starting download from {}", url);
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        if let Some(pb) = &pb {
            let new = min(downloaded + (chunk.len() as u64), file_size);
            downloaded = new;
            pb.set_position(new);
        }
    }
    let finished = File::open(file_path)?;
    debug!("Finished download to {}", file_path.display());

    if let Some(pb) = &pb {
        pb.finish_with_message(format!(
            "Downloaded {}!",
            file_path.file_name().unwrap().to_string_lossy()
        ));
    }
    Ok(finished)
}

/// Wrapper for calling `download_file_with_progress` without a progress bar
/// # Params
/// * url - Url to download from
/// * file_path - Full path to save file to
pub async fn download_file(url: &str, file_path: impl AsRef<Path>) -> Result<File, ThermiteError> {
    download_file_with_progress(url, file_path.as_ref(), None).await
}

pub fn uninstall(mods: Vec<&PathBuf>) -> Result<(), ThermiteError> {
    for p in mods {
        if fs::remove_dir_all(p).is_err() {
            //try removing a file too, just in case
            debug!("Removing dir failed, attempting to remove file...");
            fs::remove_file(p)?;
        }
    }
    Ok(())
}

pub fn install_mod(zip_file: &File, target_dir: &Path) -> Result<LocalMod, ThermiteError> {
    debug!("Starting mod insall");
    let mods_dir = target_dir.canonicalize()?;
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
        let mut archive = ZipArchive::new(zip_file)?;

        archive
            .by_name("manifest.json")?
            .read_to_string(&mut manifest)
            .unwrap();

        fs::create_dir_all(&temp_dir)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let out = temp_dir.join(file.enclosed_name().unwrap());

            if file.enclosed_name().unwrap().starts_with(".") {
                debug!("Skipping hidden file {}", out.display());
                continue;
            }

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
            io::copy(&mut file, &mut outfile)?;
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
        return Err(ThermiteError::MiscError(
            "Couldn't find a directory to copy".into(),
        ));
    }

    // move the mod files from the temp dir to the real dir
    for p in mods.iter_mut() {
        let temp = temp_dir.join(&p.path);
        p.path = p.path.strip_prefix("mods").unwrap().to_path_buf();
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

    let manifest: Manifest = serde_json::from_str(&manifest)?;

    fs::remove_dir_all(&temp_dir)?;

    Ok(LocalMod {
        package_name: manifest.name,
        version: manifest.version_number,
        mods,
        depends_on: vec![],
        needed_by: vec![],
    })
}
