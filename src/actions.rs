use std::{
    cmp::min,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing, Converter, Pattern};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;
use zip::ZipArchive;

use crate::{config::Config, utils};

pub async fn download_file(url: String, file_path: PathBuf) -> Result<File, String> {
    let client = Client::new();

    //send the request
    let res = client
        .get(&url)
        .send()
        .await
        .or(Err(format!("Unable to GET from {}", &url)))?;

    if !res.status().is_success() {
        return Err(format!("{} at URL {}", res.status(), url));
    }

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
    let finished = File::open(&file_path).or(Err(format!(
        "Unable to open finished file {}",
        file_path.display()
    )))?;

    pb.finish_with_message(format!("Downloaded {} to {}", url, file_path.display()));
    Ok(finished)
}

pub fn uninstall(mods: Vec<&String>, config: &Config) -> Result<(), String> {
    let mut mods = mods;
    fs::read_dir(config.mod_dir().join("mods/"))
        .or(Err(format!("Unable to read mods directory")))?
        .for_each(|f| {
            if let Ok(f) = f {
                mods = mods
                    .clone()
                    .into_iter()
                    .filter(|e| {
                        if OsStr::new(e) == f.file_name() {
                            utils::remove_dir(&f.path(), true).unwrap();
                            println!("Uninstalled {}", e);
                            false
                        } else {
                            true
                        }
                    })
                    .collect();
            }
        });
    mods.into_iter()
        .for_each(|f| println!("Mod {} isn't installed", f));

    Ok(())
}

//supposing the mod name is formatted like Author.Mod@v1.0.0
pub fn parse_mod_name(name: &str) -> Option<String> {
    let parts = name.split_once(".")?;
    let author = parts.0;
    let parts = parts.1.split_once("@")?;
    let m_name = parts.0;
    let ver = parts.1.replace("v", "");

    let big_snake = Converter::new()
        .set_delim("_")
        .set_pattern(Pattern::Capital);

    Some(format!(
        "/{}/{}/{}",
        author,
        big_snake.convert(&m_name),
        ver
    ))
}

pub fn install_mod(zip_file: &File, config: &Config) -> Result<String, String> {
    let mods_dir = config.mod_dir();
    let mut archive = ZipArchive::new(zip_file).or(Err(format!("Unable to read zip archive")))?;
    let mut deep = false;
    let mut pkg = String::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .or(Err(format!("Unable to get file from archive")))?;
        let out = file
            .enclosed_name()
            .ok_or(format!("Unable to get file name"))?;

        if out.starts_with("mods/") {
            if !deep {
                //this probably isn't very robust but idk
                let stripped = out.parent().unwrap().strip_prefix("mods/").unwrap();
                pkg = stripped.to_str().unwrap().to_owned();
                deep = true;
            }
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

    Ok(pkg)
}
