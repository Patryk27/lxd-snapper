use std::path::Path;

use clap::ArgMatches;

use lib_config::Config;

use crate::Result;

pub fn boot_config(matches: &ArgMatches) -> Result<Config> {
    let config = Path::new(
        matches
            .value_of("config")
            .unwrap_or("config.yaml")
    );

    Ok(Config::from_yaml_file(&config)?)
}