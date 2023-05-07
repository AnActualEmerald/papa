use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs::read_to_string;
use std::{io::ErrorKind, path::PathBuf};

use crate::model::ModName;

use super::install;

pub fn import(file: PathBuf, yes: bool, force: bool, no_cache: bool) -> Result<()> {
    println!("Loading '{}'...", file.display().bright_cyan());
    let raw = match read_to_string(&file) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            eprintln!("Couldn't find file '{}'", file.display().bright_red());
            return Err(e.into());
        }
        Err(e) => {
            eprintln!("Unable to read file: {e}");
            return Err(e.into());
        }
    };

    println!("Parsing mod list...");
    let list: Vec<ModName> = ron::from_str::<Vec<String>>(&raw).map(|v| {
        v.into_iter()
            .filter_map(|modname| ModName::try_from(modname).ok())
            .collect()
    })?;

    install(list, yes, force, no_cache)
}
