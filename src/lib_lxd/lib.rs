/// This module contains a simple, minimal implementation of the LXD client (based on the `lxc`
/// application).
///
/// # License
///
/// Copyright (c) 2019, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.
/// Licensed under the MIT license.

pub use self::{
    client::*,
    clients::*,
    error::*,
    models::*,
    result::*,
};

mod client;
mod clients;
mod error;
mod models;
mod result;