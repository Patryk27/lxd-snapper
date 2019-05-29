use std::{fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    IoError(PathBuf, io::Error),
    SerdeError(PathBuf, serde_yaml::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(path, err) => write!(f, "Could not open file [{}]: {}", path.to_str().unwrap(), err),
            Error::SerdeError(path, err) => write!(f, "Could not parse file [{}]: {}", path.to_str().unwrap(), err),
        }
    }
}