use std::fmt;

use crate::InputError;

#[derive(Debug)]
pub enum Error {
    CommandError(lib_command::Error),
    ConfigError(lib_config::Error),
    InputError(InputError),
    LxdError(lib_lxd::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CommandError(err) => write!(f, "{}", err),
            Error::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Error::InputError(err) => write!(f, "{}", err),
            Error::LxdError(err) => write!(f, "LXD error: {}", err),
        }
    }
}

impl From<lib_command::Error> for Error {
    fn from(err: lib_command::Error) -> Self {
        Error::CommandError(err)
    }
}

impl From<lib_config::Error> for Error {
    fn from(err: lib_config::Error) -> Self {
        Error::ConfigError(err)
    }
}

impl From<InputError> for Error {
    fn from(err: InputError) -> Self {
        Error::InputError(err)
    }
}

impl From<lib_lxd::Error> for Error {
    fn from(err: lib_lxd::Error) -> Self {
        Error::LxdError(err)
    }
}