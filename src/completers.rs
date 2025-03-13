use clap_complete::CompletionCandidate;
use clap_lex::OsStrExt;
use thermite::{api::get_package_index, core::find_mods};

use crate::{config::CONFIG, model::ModName, utils::GroupedMods};

pub fn profiles(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let prefix = CONFIG
        .game_dir()
        .cloned()
        .or_else(|| std::env::current_dir().ok())
        .expect("Game dir or cwd");

    let Ok(profiles) = super::profile::find_profiles(&prefix) else {
        return vec![];
    };

    profiles
        .iter()
        .filter_map(|prof| prof.strip_prefix(&prefix).ok())
        .map(|profile| profile.as_os_str())
        .filter(|profile| profile.starts_with(&current.to_string_lossy()))
        .map(CompletionCandidate::new)
        .collect()
}

pub fn installed_mods(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let current = current.to_lowercase();

    let prefix = CONFIG
        .install_dir()
        .or_else(|_| std::env::current_dir())
        .expect("Game dir or cwd");

    let Ok(mods) = find_mods(prefix) else {
        return vec![];
    };

    mods.iter()
        .map(ModName::from)
        .map(|name| name.to_string())
        .filter(|name| name.to_lowercase().starts_with(&current))
        .map(CompletionCandidate::new)
        .collect()
}

pub fn mod_index(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let current = current.to_lowercase();

    let Ok(mods) = get_package_index() else {
        return vec![];
    };

    mods.iter()
        .map(|m| ModName::from(m).to_string())
        .filter(|name| name.to_lowercase().starts_with(&current))
        .map(CompletionCandidate::new)
        .collect()
}

pub fn enabled_mods(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let current = current.to_lowercase();

    let prefix = CONFIG
        .install_dir()
        .or_else(|_| std::env::current_dir())
        .expect("Game dir or cwd");

    let Ok(mods) = GroupedMods::try_from_dir(&prefix) else {
        return vec![];
    };

    let mods = mods.enabled;

    let mut names = vec![];
    for (modname, group) in mods {
        names.push(
            ModName {
                author: modname.author.clone(),
                name: modname.name.clone(),
                version: None,
            }
            .to_string(),
        );
        if group.len() > 1 {
            let rest = group
                .iter()
                .skip(1)
                .map(|installed| installed.mod_json.name.clone());

            names.extend(rest);
        }
    }

    names
        .iter()
        .filter(|name| name.to_lowercase().starts_with(&current))
        .map(CompletionCandidate::new)
        .collect()
}

pub fn disabled_mods(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let current = current.to_lowercase();

    let prefix = CONFIG
        .install_dir()
        .or_else(|_| std::env::current_dir())
        .expect("Game dir or cwd");

    let Ok(mods) = GroupedMods::try_from_dir(&prefix) else {
        return vec![];
    };

    let mods = mods.disabled;

    let mut names = vec![];
    for (modname, group) in mods {
        names.push(
            ModName {
                author: modname.author.clone(),
                name: modname.name.clone(),
                version: None,
            }
            .to_string(),
        );
        if group.len() > 1 {
            let rest = group
                .iter()
                .skip(1)
                .map(|installed| installed.mod_json.name.clone());

            names.extend(rest);
        }
    }

    names
        .iter()
        .filter(|name| name.to_lowercase().starts_with(&current))
        .map(CompletionCandidate::new)
        .collect()
}
