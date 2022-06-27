use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub(crate) struct ScorchError(pub String);

impl Display for ScorchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&dyn Error> for ScorchError {
    fn from(e: &dyn Error) -> Self {
        ScorchError(e.to_string())
    }
}

impl From<String> for ScorchError {
    fn from(e: String) -> Self {
        ScorchError(e)
    }
}

impl From<&str> for ScorchError {
    fn from(e: &str) -> Self {
        ScorchError(e.to_string())
    }
}

impl From<std::io::Error> for ScorchError {
    fn from(e: std::io::Error) -> Self {
        ScorchError(e.to_string())
    }
}
