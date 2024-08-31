use anyhow::{anyhow, Result};
use thermite::model::ModVersion;
use tracing::warn;

use crate::get_answer;
use crate::model::ModName;
use crate::traits::{Answer, Indexed};
use crate::utils::{download_and_install, to_file_size_string};

use owo_colors::OwoColorize;
use thermite::prelude::*;

pub fn install(mods: Vec<ModName>, assume_yes: bool, force: bool, no_cache: bool) -> Result<()> {
    let remote_index = get_package_index()?;
    let mut valid: Vec<(ModName, &ModVersion)> = vec![];
    let mut should_fail = false;
    for mn in mods {
        if mn.name.to_lowercase() == "northstar" && mn.author.to_lowercase() == "northstar" {
            warn!("Can't install Northstar like a normal mod");
            println!(
                "Not installing Northstar - use {} instead",
                "papa ns init".bright_cyan()
            );
            should_fail = !force;
            continue;
        }
        if let Some(m) = remote_index.get_item(&mn) {
            if let Some(version) = &mn.version {
                let Some(mv) = m.get_version(version) else {
                    println!(
                        "Package {} has no version {}",
                        format!("{}.{}", mn.author, mn.name).bright_cyan(),
                        version.bright_cyan()
                    );
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

    let answer = get_answer!(assume_yes)?;
    if !answer.is_no() {
        download_and_install(valid, !no_cache, force)?;
    }

    Ok(())
}
