use std::result;

use crate::Error;

pub type Result<T = ()> = result::Result<T, Error>;