use std::fmt;

#[derive(Debug)]
pub enum InputError {
    UnknownCommand,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::UnknownCommand => write!(f, "Unknown command"),
        }
    }
}