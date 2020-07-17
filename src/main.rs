#![feature(box_syntax)]
#![feature(crate_visibility_modifier)]
#![feature(try_blocks)]

use crate::config::Config;
use anyhow::*;
use clap::Clap;
use colored::*;
use lib_lxd::{LxdClient, LxdDummyClient, LxdProcessClient};
use std::io::stdout;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};

mod cmds;
mod config;

/// LXD snapshots, automated
#[derive(Clap, Debug)]
struct Args {
    /// Runs application in a simulated safe-mode without applying any changes
    /// to the containers
    #[clap(short, long)]
    dry_run: bool,

    /// Path to the configuration file
    #[clap(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// By default, lxd-snapper tries to locate the `lxc` executable inside your
    /// PATH variable - when this fails for you, using this parameter you can
    /// provide location of the `lxc` executable by hand
    lxc_path: Option<PathBuf>,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap, Debug)]
enum Command {
    /// Creates a snapshot for each container matching the policy
    Backup,

    /// Shorthand for `backup` followed by `prune`
    BackupAndPrune,

    /// Removes *ALL* snapshots for each container matching the policy; you most
    /// likely *don't* want to use it on production
    Nuke,

    /// Removes stale snapshots for each container matching the policy
    Prune,

    /// Validates policy's syntax
    Validate,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();

    if let Command::Validate = &args.cmd {
        return cmds::validate(args);
    }

    let stdout = &mut stdout();
    let config = config(&args.config)?;
    let mut lxd = lxd(args.dry_run, args.lxc_path)?;

    match args.cmd {
        Command::Backup => cmds::backup(stdout, &config, lxd.deref_mut()),
        Command::BackupAndPrune => cmds::backup_and_prune(stdout, &config, lxd.deref_mut()),
        Command::Nuke => cmds::nuke(stdout, &config, lxd.deref_mut()),
        Command::Prune => cmds::prune(stdout, &config, lxd.deref_mut()),
        Command::Validate => unreachable!(),
    }
}

fn config(path: &Path) -> Result<Config> {
    Config::from_file(path)
}

fn lxd(dry_run: bool, lxc_path: Option<PathBuf>) -> Result<Box<dyn LxdClient>> {
    let mut lxd = if let Some(lxc_path) = lxc_path {
        LxdProcessClient::new_ex(lxc_path)
    } else {
        LxdProcessClient::new().context("Couldn't initialize LXC client")?
    };

    if !dry_run {
        return Ok(box lxd);
    }

    println!(
        "{} --dry-run is active, no changes will be applied\n",
        "Note:".green(),
    );

    Ok(box LxdDummyClient::from_other(&mut lxd)?)
}
