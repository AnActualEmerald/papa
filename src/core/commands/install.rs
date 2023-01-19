use anyhow::{anyhow, Context, Result};
use tracing::instrument;

use crate::config::{CONFIG, DIRS};
use crate::model::ModName;
use crate::traits::RemoteIndex;
use crate::utils::{ensure_dir, to_file_size_string};
use crate::{flush, readln};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use thermite::prelude::*;

#[instrument]
pub fn install(mods: Vec<ModName>, assume_yes: bool, force: bool, global: bool) -> Result<()> {
    let remote_index = get_package_index()?;
    let mut valid = vec![];
    let mut should_fail = false;
    for mn in mods {
        if let Some(m) = remote_index.get_mod(&mn) {
            if let Some(version) = &mn.version {
                let Some(mv) = m.get_version(version) else {
                    println!("Package {} has no version {}", format!("{}.{}", mn.author, mn.name).bright_cyan(), version.bright_cyan());
                    should_fail = true;
                    continue;
                };
                valid.push((mn, mv));
            } else {
                valid.push((
                    mn,
                    m.get_latest()
                        .expect("Latest version of mod doesn't exist?"),
                ));
            }
        } else {
            println!("Couldn't find package {}", mn.bright_cyan());
            should_fail = true;
        }
    }

    if should_fail && !force {
        return Err(anyhow!(
            "Failed to find some packages, transaction aborted!"
        ));
    }

    let mut deps = vec![];
    for v in valid.iter() {
        deps.append(&mut resolve_deps(&v.1.deps, &remote_index)?);
    }
    let mut deps = deps
        .iter()
        .map(|v| (v.into(), v.get_latest().unwrap()))
        .collect();

    valid.append(&mut deps);

    // total download size in bytes
    let total_size = valid.iter().map(|(_, v)| v.file_size).sum::<u64>();

    println!("Preparing to download: ");
    for (n, v) in valid.iter() {
        println!(
            "    {} - {}",
            n.bright_green(),
            v.file_size_string().yellow()
        );
    }
    println!(
        "Total download size: {}",
        to_file_size_string(total_size).bright_green().bold()
    );

    let answer = if !assume_yes {
        readln!("OK? [Y/n]: ")?
    } else {
        String::new()
    };

    if answer.to_lowercase().trim() != "n" {
        println!("Downloading packages...");
        let mut files = vec![];
        let cache_dir = DIRS.cache_dir();
        for (mn, v) in valid {
            // flush!()?;
            let filename = cache_dir.join(format!("{}.zip", mn));
            ensure_dir(&cache_dir)?;
            let pb = ProgressBar::new(v.file_size)
                .with_style(
                    ProgressStyle::with_template("{msg}{bar} {bytes}/{total_bytes} {duration}")?
                        .progress_chars(".. "),
                )
                .with_message(format!("Downloading {mn}..."));
            let file = download_file_with_progress(&v.url, filename, |current, _| {
                pb.set_position(current);
            })
            .context(format!("Error downloading {}", mn))?;
            pb.finish();
            files.push((mn.author, file));
        }
        println!("Installing packages...");
        for (author, f) in files {
            if !CONFIG.is_server() {
                ensure_dir(CONFIG.install_dir())?;
                install_mod(author, &f, CONFIG.install_dir())?;
            } else {
                todo!();
            }
        }
        println!("Done!");
    }

    Ok(())
}
