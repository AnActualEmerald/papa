use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs::read_to_string;
use std::{io::ErrorKind, path::PathBuf};

use crate::model::ModName;

pub fn import(file: PathBuf) -> Result<()> {
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

    let list: Vec<ModName> = ron::from_str::<Vec<String>>(&raw).map(|v| {
        v.into_iter()
            .filter_map(|modname| ModName::try_from(modname).ok())
            .collect()
    })?;

    println!("{list:?}");
    Ok(())
}
