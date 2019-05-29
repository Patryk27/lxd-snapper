/// This module is responsible for connecting all the other modules together - it's both the entry
/// point and the heart of the application.
///
/// # License
///
/// Copyright (c) 2019, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.
/// Licensed under the MIT license.

use clap::{App, load_yaml};
use colored::*;

use lib_command::{backup_containers, nuke_containers, prune_containers};

use self::{
    bootstrap::*,
    error::*,
    input::*,
    result::*,
};

mod bootstrap;
mod error;
mod input;
mod result;

pub fn main() {
    if let Err(err) = run() {
        println!(
            "{} {}",
            "Error:".red(),
            format!("{}", err).white()
        );

        println!();

        println!(
            "{} Please visit {} if you encounter any trouble.",
            "Tip:".green(),
            "https://github.com/Patryk27/lxd-snapper".green()
        );
    }
}

fn run() -> Result {
    let cli = load_yaml!("cli.yml");
    let app = App::from_yaml(cli);
    let matches = app.get_matches();

    if matches.is_present("dry-run") {
        println!("{} --dry-run is active, no changes will be applied.", "Note:".yellow());
        println!();
    }

    let config = boot_config(&matches)?;
    let mut lxd = boot_lxd(&config, &matches)?;

    Ok(match matches.subcommand() {
        ("backup", _) => {
            backup_containers(&config, lxd.as_mut())?
        }

        ("backup-prune", _) => {
            backup_containers(&config, lxd.as_mut())?;

            println!();
            println!("------");
            println!();

            prune_containers(&config, lxd.as_mut())?
        }

        ("nuke", _) => {
            nuke_containers(&config, lxd.as_mut())?
        }

        ("prune", _) => {
            prune_containers(&config, lxd.as_mut())?
        }

        ("validate", _) => {
            // No additional validation is required here, because when control gets as far as here,
            // it must have *already* parsed configuration file and started the LXD client (take a
            // look a few lines above, just before this `match`).

            println!("{}", "Great - everything seems to be working just fine.".green());
        }

        _ => Err(InputError::UnknownCommand)?,
    })
}