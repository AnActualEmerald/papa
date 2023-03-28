use std::{fmt::Display, string::FromUtf8Error};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SerialzeError(Box<String>),
    #[error("Error parsing bytes into UTF8 string: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        let e = format!("{}", msg);
        Self::SerialzeError(Box::new(e))
    }
}
