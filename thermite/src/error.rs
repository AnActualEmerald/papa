use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::model::Mod;

#[derive(Error, Debug)]
pub enum ThermiteError {
    #[error("Error while installing mod {0}", m.name)]
    InstallError {
        m: Box<Mod>,
        path: Box<Path>,
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
    #[error("Error downloading file: {0}")]
    DownloadError(#[from] reqwest::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    #[error("Error parsing JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Error resolving dependency {0}")]
    DepError(String),
}
