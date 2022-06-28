use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub(crate) enum ScorchError {
    FsError(Box<dyn Error>),
    NotFound(String),
    NetworkError(Box<dyn Error>),
    Generic(Box<dyn Error>),
}

impl Display for ScorchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            e => write!(f, "{}", e),
        }
    }
}

// impl From<String> for Error {
//     fn from(e: String) -> Self {

//     }
// }

// impl From<&str> for Error {
//     fn from(e: &str) -> Self {
//         Error(e.to_string())
//     }
// }

impl From<std::io::Error> for ScorchError {
    fn from(e: std::io::Error) -> Self {
        ScorchError::FsError(Box::new(e))
    }
}
