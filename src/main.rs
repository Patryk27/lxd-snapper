mod commands;
mod config;
mod environment;

mod prelude {
    pub use crate::{config::*, environment::*};
    pub use anyhow::{bail, Context, Result};
    pub use chrono::{DateTime, Utc};
    pub use colored::Colorize;
    pub use std::io::Write;

    #[cfg(test)]
    pub use pretty_assertions as pa;

    #[cfg(test)]
    pub use indoc::indoc;
}

use self::{commands::*, config::*, environment::*};
use anyhow::*;
use chrono::Utc;
use clap::Parser;
use colored::*;
use lib_lxd::{LxdClient, LxdFakeClient, LxdProcessClient};
use std::io;
use std::path::PathBuf;

/// LXD snapshots, automated
#[derive(Parser)]
pub struct Args {
    /// Runs application in a simulated safe-mode without applying any changes
    /// to the instances
    #[clap(short, long)]
    dry_run: bool,

    /// Path to the configuration file
    #[clap(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// Path to the `lxc` executable; usually inferred automatically from the
    /// `PATH` environmental variable
    #[clap(short, long)]
    lxc_path: Option<PathBuf>,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    /// Creates a snapshot for each instance matching the policy
    Backup,

    /// Shorthand for `backup` followed by `prune`
    BackupAndPrune,

    /// Removes stale snapshots for each instance matching the policy
    Prune,

    /// Validates policy's syntax
    Validate,

    /// Various debug-oriented commands
    Debug(DebugCommand),
}

#[derive(Parser)]
pub struct DebugCommand {
    #[clap(subcommand)]
    cmd: DebugSubcommand,
}

#[derive(Parser)]
pub enum DebugSubcommand {
    /// Lists all the LXD instances together with policies associated with them
    ListInstances,

    /// Removes *ALL* snapshots (including the ones created manually) for each
    /// instance matching the policy; if you suddenly created tons of
    /// unnecessary snapshots, this is the way to go
    Nuke,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stdout = &mut io::stdout();

    if let Command::Validate = &args.cmd {
        return commands::validate(stdout, args);
    }

    if args.dry_run {
        println!(
            "{} --dry-run is active, no changes will be applied\n",
            "note:".green(),
        );
    }

    let config = init_config(&args)?;
    let mut lxd = init_lxd(&args)?;

    let mut env = Environment {
        time: Utc::now,
        stdout,
        config: &config,
        lxd: &mut *lxd,
    };

    match args.cmd {
        Command::Backup => Backup::new(&mut env).run(),
        Command::BackupAndPrune => BackupAndPrune::new(&mut env).run(),
        Command::Prune => Prune::new(&mut env).run(),

        Command::Validate => {
            // Already handled a few lines above
            unreachable!()
        }

        Command::Debug(DebugCommand {
            cmd: DebugSubcommand::ListInstances,
        }) => DebugListInstances::new(&mut env).run(),

        Command::Debug(DebugCommand {
            cmd: DebugSubcommand::Nuke,
        }) => DebugNuke::new(&mut env).run(),
    }
}

fn init_config(args: &Args) -> Result<Config> {
    let mut config = Config::load(&args.config)?;

    if args.dry_run {
        config.hooks = Default::default();
    }

    Ok(config)
}

fn init_lxd(args: &Args) -> Result<Box<dyn LxdClient>> {
    let mut lxd = if let Some(lxc_path) = &args.lxc_path {
        LxdProcessClient::new(lxc_path)
    } else {
        LxdProcessClient::find()
    }
    .context("Couldn't initialize LXC client")?;

    if args.dry_run {
        Ok(Box::new(LxdFakeClient::from(&mut lxd)?))
    } else {
        Ok(Box::new(lxd))
    }
}
