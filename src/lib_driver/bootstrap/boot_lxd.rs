use std::path::Path;

use clap::ArgMatches;

use lib_config::Config;
use lib_lxd::{LxdClient, LxdInMemoryClient, LxdProcessClient};

use crate::Result;

pub fn boot_lxd(config: &Config, matches: &ArgMatches) -> Result<Box<dyn LxdClient>> {
    // Initialize LXD client
    let mut lxd = if let Some(path) = &config.lxc_path {
        LxdProcessClient::new(Path::new(path))
    } else {
        LxdProcessClient::new_detect()?
    };

    // Try to list containers, just to check that the connection works
    let containers = lxd.list()?;

    // If user enabled the "dry run" mode, switch to the in-memory client
    if matches.is_present("dry-run") {
        Ok(Box::new(
            LxdInMemoryClient::new(containers)
        ))
    } else {
        Ok(Box::new(lxd))
    }
}
