use std::{fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    ClientError(String),
    FailedToAutodetect,
    FailedToExecute(PathBuf, io::Error),
    FailedToParse(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ClientError(err) => write!(f, "{}", err),
            Error::FailedToAutodetect => write!(f, "Could not auto-detect your LXC client\'s installation path - please try specifying path with the `lxc-path` configuration key"),
            Error::FailedToExecute(path, err) => write!(f, "Failed to execute the LXC client [{}]: {}. Make sure the path you've specified is correct.", path.to_str().unwrap(), err),
            Error::FailedToParse(err) => write!(f, "Failed to parse LXC client\'s response: {}. This is most likely a bug in the `lxd-snapper`.", err),
        }
    }
}