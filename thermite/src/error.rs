use std::io;

use thiserror::Error;

use crate::model::Mod;

#[derive(Error, Debug)]
pub enum ThermiteError {
    #[error("Error while installing mod {m.name}")]
    InstallError {
        m: Mod,
        path: PathBuf,
        source: Box<dyn std::error::Error>,
    },
    #[error("No such file {0:?}")]
    MissingFile(PathBuf),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("Error parsing RON")]
    RonError(#[from] ron::Error),
    #[error("{0}")]
    MiscError(String),
}
