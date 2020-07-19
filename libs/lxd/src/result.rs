use crate::Error;
use std::result;

pub type Result<T> = result::Result<T, Error>;
