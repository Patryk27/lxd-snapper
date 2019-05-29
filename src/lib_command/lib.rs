/// # License
///
/// Copyright (c) 2019, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.
/// Licensed under the MIT license.

pub use self::{
    commands::*,
    error::*,
    result::*,
};

mod commands;
mod error;
mod result;

const SNAPSHOT_NAME_FORMAT: &str = "auto-%Y%m%d-%H%M%S";