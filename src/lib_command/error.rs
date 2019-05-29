use std::fmt;

#[derive(Debug)]
pub enum Error {
    LxdError(lib_lxd::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LxdError(err) => write!(f, "LXD error: {}", err),
        }
    }
}

impl From<lib_lxd::Error> for Error {
    fn from(err: lib_lxd::Error) -> Self {
        Error::LxdError(err)
    }
}