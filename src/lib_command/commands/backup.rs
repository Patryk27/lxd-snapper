use std::io::{stdout, Write};

use chrono::Local;
use colored::*;

use lib_config::Config;
use lib_lxd::{LxdClient, LxdContainer, LxdSnapshotName};

use crate::{Result, SNAPSHOT_NAME_FORMAT};

#[derive(Default)]
struct Summary {
    processed_containers: usize,
    created_snapshots: usize,
    errors: usize,
}

impl Summary {
    fn print(self) {
        println!();
        println!("{}", "Summary".white().underline());
        println!("- {} processed container(s)", format!("{}", self.processed_containers).white());
        println!("- {} created snapshot(s)", format!("{}", self.created_snapshots).white());
        println!("- {} error(s)", format!("{}", self.errors).white());
        println!();

        println!("{}", match self.errors {
            0 => {
                "All containers have been backed up.".green()
            }

            n if n == self.processed_containers => {
                "All containers failed to be backed up.".red()
            }

            _ => {
                "Some containers failed to be backed up.".yellow()
            }
        });
    }
}

pub fn backup_containers(config: &Config, lxd: &mut dyn LxdClient) -> Result {
    println!("{}", "Backing-up containers".white().underline());

    let containers = lxd.list()?.into_iter();

    let summary = containers.fold(Summary::default(), |mut summary, container| {
        summary.processed_containers += 1;

        if config.determine_policy_for(&container).is_some() {
            let result = backup_container(lxd, container);

            summary.created_snapshots += result.0;
            summary.errors += result.1;
        };

        summary
    });

    Ok(summary.print())
}

fn backup_container(lxd: &mut dyn LxdClient, container: LxdContainer) -> (usize, usize) {
    print!("- {}: ", container.name.inner().blue().bold());
    let _ = stdout().flush();

    let snapshot_name = LxdSnapshotName::new(
        &format!("{}", Local::now().format(SNAPSHOT_NAME_FORMAT))
    );

    match lxd.create_snapshot(&container.name, &snapshot_name) {
        Ok(_) => {
            println!("{} {}", snapshot_name, "[ CREATED ]".green());
            (1, 0)
        }

        Err(err) => {
            println!("{} {}", err, "[ FAILED ]".red());
            (0, 1)
        }
    }
}
