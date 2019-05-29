use std::io::{stdout, Write};

use colored::*;

use lib_config::Config;
use lib_lxd::{LxdClient, LxdContainer};

use crate::Result;

#[derive(Default)]
struct Summary {
    processed_containers: usize,
    deleted_snapshots: usize,
    errors: usize,
}

impl Summary {
    fn print(self) {
        println!("{}", "Summary".white().underline());
        println!("- {} processed container(s)", format!("{}", self.processed_containers).white());
        println!("- {} deleted snapshot(s)", format!("{}", self.deleted_snapshots).white());
        println!("- {} error(s)", format!("{}", self.errors).white());
        println!();

        println!("{}", match self.errors {
            0 => {
                "All containers have been nuked.".green()
            }

            n if n == self.processed_containers => {
                "All containers failed to be nuked.".red()
            }

            _ => {
                "Some containers failed to be nuked.".yellow()
            }
        });
    }
}

pub fn nuke_containers(config: &Config, lxd: &mut dyn LxdClient) -> Result {
    println!("{}", "Nuking containers".white().underline());

    let containers = lxd.list()?.into_iter();

    let summary = containers.fold(Summary::default(), |mut summary, container| {
        summary.processed_containers += 1;

        if config.determine_policy_for(&container).is_some() {
            let result = nuke_container(lxd, container);

            summary.deleted_snapshots += result.0;
            summary.errors += result.1;
        }

        summary
    });

    Ok(summary.print())
}

fn nuke_container(lxd: &mut dyn LxdClient, container: LxdContainer) -> (usize, usize) {
    println!("- {}: ", container.name.inner().blue().bold());

    let (mut deleted, mut errors) = (0, 0);
    let snapshots = container.snapshots.unwrap_or_default();

    if snapshots.is_empty() {
        println!("  (no snapshots found)");
    } else {
        for snapshot in snapshots {
            print!("  - {} ", snapshot.name);
            let _ = stdout().flush();

            match lxd.delete_snapshot(&container.name, &snapshot.name) {
                Ok(()) => {
                    deleted += 1;
                    println!("{}", "[ DELETED ]".green());
                }

                Err(err) => {
                    errors += 1;
                    println!("{} {}", err, "[ FAILED ]".red());
                }
            }
        }
    }

    println!();

    (deleted, errors)
}