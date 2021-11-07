use crate::config::Config;
use anyhow::Result;
use lib_lxd::*;
use std::io::Write;

pub fn backup_and_prune(
    stdout: &mut dyn Write,
    config: &Config,
    lxd: &mut dyn LxdClient,
) -> Result<()> {
    super::backup(stdout, config, lxd)?;
    println!();
    super::prune(stdout, config, lxd)
}
