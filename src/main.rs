use crate::config::Config;
use anyhow::*;
use clap::Clap;
use colored::*;
use lib_lxd::{LxdClient, LxdFakeClient, LxdProcessClient};
use std::io::stdout;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};

mod cmds;
mod config;

/// LXD snapshots, automated
#[derive(Clap, Debug)]
struct Args {
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

#[derive(Clap, Debug)]
enum Command {
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

    /// Various query-oriented commands
    Query(QueryCommand),
}

#[derive(Clap, Debug)]
struct DebugCommand {
    #[clap(subcommand)]
    cmd: DebugSubcommand,
}

#[derive(Clap, Debug)]
enum DebugSubcommand {
    /// Removes *ALL* snapshots (including the ones created manually) for each
    /// instance (i.e. container & virtual machine) matching the policy; if
    /// you suddenly created tons of unnecessary snapshots, this is the way to
    /// go
    Nuke,
}

#[derive(Clap, Debug)]
struct QueryCommand {
    #[clap(subcommand)]
    cmd: QuerySubcommand,
}

#[derive(Clap, Debug)]
enum QuerySubcommand {
    /// Lists all the LXD instances together with policies associated with them
    Instances,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();

    if let Command::Validate = &args.cmd {
        return cmds::validate(args);
    }

    let stdout = &mut stdout();
    let config = load_config(&args.config)?;
    let mut lxd = init_lxd(args.dry_run, args.lxc_path)?;

    match args.cmd {
        Command::Backup => cmds::backup(stdout, &config, lxd.deref_mut()),

        Command::BackupAndPrune => cmds::backup_and_prune(stdout, &config, lxd.deref_mut()),

        Command::Prune => cmds::prune(stdout, &config, lxd.deref_mut()),

        Command::Validate => unreachable!(),

        Command::Debug(DebugCommand {
            cmd: DebugSubcommand::Nuke,
        }) => cmds::debug_nuke(stdout, &config, lxd.deref_mut()),

        Command::Query(QueryCommand {
            cmd: QuerySubcommand::Instances,
        }) => cmds::query_instances(stdout, &config, lxd.deref_mut()),
    }
}

fn load_config(path: &Path) -> Result<Config> {
    Config::from_file(path)
}

fn init_lxd(dry_run: bool, lxc_path: Option<PathBuf>) -> Result<Box<dyn LxdClient>> {
    let mut lxd = if let Some(lxc_path) = lxc_path {
        LxdProcessClient::new(lxc_path)
    } else {
        LxdProcessClient::new_from_path().context("Couldn't initialize LXC client")?
    };

    if dry_run {
        println!(
            "{} --dry-run is active, no changes will be applied\n",
            "note:".green(),
        );

        Ok(Box::new(LxdFakeClient::wrap(&mut lxd)?))
    } else {
        Ok(Box::new(lxd))
    }
}
