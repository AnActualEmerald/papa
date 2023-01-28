use anyhow::{anyhow, Result};
use thermite::model::ModVersion;

use crate::model::ModName;
use crate::readln;
use crate::traits::Index;
use crate::utils::{download_and_install, to_file_size_string};

use owo_colors::OwoColorize;
use thermite::prelude::*;

pub fn install(mods: Vec<ModName>, assume_yes: bool, force: bool, _global: bool) -> Result<()> {
    let remote_index = get_package_index()?;
    let mut valid: Vec<(ModName, &ModVersion)> = vec![];
    let mut should_fail = false;
    for mn in mods {
        if let Some(m) = remote_index.get_item(&mn) {
            if let Some(version) = &mn.version {
                let Some(mv) = m.get_version(version) else {
                    println!("Package {} has no version {}", format!("{}.{}", mn.author, mn.name).bright_cyan(), version.bright_cyan());
                    should_fail = true;
                    continue;
                };
                valid.push((m.into(), mv));
            } else {
                valid.push((
                    m.into(),
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

    if !answer.to_lowercase().trim().starts_with("n") {
        download_and_install(valid)?;
    }

    Ok(())
}
