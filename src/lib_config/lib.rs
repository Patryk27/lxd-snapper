/// This module contains all the structs and functions related to lxd-snapper's configuration files
/// (`config.yml`).
///
/// # License
///
/// Copyright (c) 2019, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.
/// Licensed under the MIT license.

pub use self::{
    config::*,
    error::*,
    policy::*,
    result::*,
};

mod config;
mod error;
mod policy;
mod result;
