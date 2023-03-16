mod commands;
mod config;
mod environment;
mod lxd;
mod utils;

#[cfg(test)]
mod testing;

mod prelude {
    pub(crate) use crate::utils::*;
    pub(crate) use crate::{config::*, environment::*, lxd::*};
    pub use anyhow::{bail, Context, Error, Result};
    pub use chrono::{DateTime, Utc};
    pub use colored::Colorize;
    pub use itertools::Itertools;
    pub use std::io::Write;

    #[cfg(test)]
    pub use pretty_assertions as pa;

    #[cfg(test)]
    pub use indoc::indoc;
}

use self::{commands::*, config::*, environment::*, lxd::*};
use anyhow::*;
use chrono::Utc;
use clap::{Parser, Subcommand};
use colored::*;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

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
    /// Creates a snapshot for each instance matching the configuration
    Backup,

    /// Shorthand for `backup` followed by `prune`
    BackupAndPrune,

    /// Removes stale snapshots from each instance matching the configuration
    Prune,

    /// Validates configuration syntax
    Validate,

    /// Various debug-commands
    #[clap(subcommand)]
    Debug(DebugCommand),
}

#[derive(Subcommand)]
pub enum DebugCommand {
    /// Lists all the LXD instances together with policies associated with them
    ListInstances,

    /// Removes *ALL* snapshots (including the ones created manually) from each
    /// instance matching the configuration; if you suddenly created tons of
    /// unnecessary snapshots, this is the way to go
    Nuke,
}

fn main() -> ExitCode {
    use std::result::Result::*;

    match try_main() {
        Ok(_) => ExitCode::SUCCESS,

        Err(err) => {
            println!();
            println!("{}: {:?}", "Error".red(), err);

            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<()> {
    let args = Args::parse();
    let stdout = &mut io::stdout();

    if let Command::Validate = &args.cmd {
        return commands::validate(stdout, args);
    }

    if args.dry_run {
        println!(
            "({} is active, no changes will be applied)",
            "--dry-run".yellow(),
        );
        println!();
    }

    let config = Config::load(&args.config)?;
    let mut lxd = init_lxd(&args, &config)?;

    let mut env = Environment {
        time: Utc::now,
        stdout,
        config: &config,
        lxd: &mut *lxd,
        dry_run: args.dry_run,
    };

    match args.cmd {
        Command::Backup => Backup::new(&mut env).run(),
        Command::BackupAndPrune => BackupAndPrune::new(&mut env).run(),
        Command::Prune => Prune::new(&mut env).run(),

        Command::Validate => {
            // Already handled a few lines above
            unreachable!()
        }

        Command::Debug(DebugCommand::ListInstances) => DebugListInstances::new(&mut env).run(),
        Command::Debug(DebugCommand::Nuke) => DebugNuke::new(&mut env).run(),
    }
}

fn init_lxd(args: &Args, config: &Config) -> Result<Box<dyn LxdClient>> {
    let mut lxd = if let Some(lxc_path) = &args.lxc_path {
        LxdProcessClient::new(lxc_path, config.lxd_timeout())
    } else {
        LxdProcessClient::find(config.lxd_timeout())
    }
    .context("Couldn't initialize LXC client")?;

    if args.dry_run {
        Ok(Box::new(LxdFakeClient::clone_from(
            &mut lxd,
            config.remotes().iter(),
        )?))
    } else {
        Ok(Box::new(lxd))
    }
}
