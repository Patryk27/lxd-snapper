use std::io::{stdout, Write};

use colored::*;
use indexmap::IndexSet;

use lib_config::{Config, Policy};
use lib_lxd::{LxdClient, LxdContainer, LxdContainerName, LxdSnapshot, LxdSnapshotName};

use crate::Result;

#[derive(Default)]
struct Summary {
    processed_containers: usize,
    deleted_snapshots: usize,
    kept_snapshots: usize,
    errors: usize,
}

impl Summary {
    fn print(self) {
        println!("{}", "Summary".white().underline());
        println!("- {} processed container(s)", format!("{}", self.processed_containers).white());
        println!("- {} deleted snapshot(s)", format!("{}", self.deleted_snapshots).white());
        println!("- {} kept snapshot(s)", format!("{}", self.kept_snapshots).white());
        println!("- {} error(s)", format!("{}", self.errors).white());
        println!();

        println!("{}", match self.errors {
            0 => {
                "All containers have been pruned.".green()
            }

            n if n == self.processed_containers => {
                "All containers failed to be pruned.".red()
            }

            _ => {
                "Some containers failed to be pruned.".yellow()
            }
        });
    }
}

pub fn prune_containers(config: &Config, lxd: &mut dyn LxdClient) -> Result {
    println!("{}", "Pruning containers".white().underline());

    let containers = lxd.list()?.into_iter();

    let summary = containers.fold(Summary::default(), |mut summary, container| {
        summary.processed_containers += 1;

        if let Some(policy) = config.determine_policy_for(&container) {
            let result = prune_container(lxd, container, policy);

            summary.deleted_snapshots += result.0;
            summary.kept_snapshots += result.1;
            summary.errors += result.2;
        }

        summary
    });

    // Finalize
    Ok(summary.print())
}

fn prune_container(
    lxd: &mut dyn LxdClient,
    container: LxdContainer,
    policy: Policy,
) -> (usize, usize, usize) {
    println!("- {}", container.name.inner().blue().bold());

    let snapshots = find_snapshots(container.snapshots);
    let alive_snapshots = find_alive_snapshots(&snapshots, &policy);

    prune_snapshots(lxd, &container.name, &snapshots, alive_snapshots)
}

/// Returns a list containing all the snapshots that match our automatically-generated names
/// ("auto-xxx").
///
/// # Example
///
/// Example input:
///  - snapshots: `snap0`, `auto-20190101`, `snap2`, `auto-20190102`
///
/// Example output:
///  - snapshots: `auto-20190101`, `auto-20190102`
fn find_snapshots(snapshots: Option<Vec<LxdSnapshot>>) -> Vec<LxdSnapshot> {
    let mut snapshots = snapshots.unwrap_or_default();

    snapshots.sort_by(|a, b| {
        b.created_at.cmp(&a.created_at)
    });

    snapshots
        .into_iter()
        .filter(|snapshot| {
            snapshot.name
                .inner()
                .starts_with("auto-")
        })
        .collect()
}

/// Returns a list containing names of all the snapshots that should be kept alive - all the other
/// ones should get dropped, as so states the policy.
///
/// # Input
///
/// Given snapshots *must* be sorted ascending by their creation date (that is: from the newest ones
/// to the oldest), otherwise everything will work the other way round (that is: the newest will be
/// marked to remove).
///
/// # Example
///
/// Example input:
///   - policy: `keep last = 3`
///   - snapshots: `auto-1`, `auto-2`, `auto-3`, `auto-4`, `auto-5`
///
/// Example output:
///   - snapshots: `auto-1`, `auto-2`, `auto-3`
fn find_alive_snapshots<'a>(
    snapshots: &'a [LxdSnapshot],
    policy: &'a Policy,
) -> IndexSet<&'a LxdSnapshotName> {
    const PATTERNS: &[(&str, &str)] = &[
        ("hourly", "%Y-%m-%d %H"),
        ("daily", "%Y-%m-%d"),
        ("weekly", "%Y-%m-%U"),
        ("monthly", "%Y-%m"),
        ("yearly", "%Y-%m"),
    ];

    let mut keep_hourly = policy.keep_hourly();
    let mut keep_daily = policy.keep_daily();
    let mut keep_weekly = policy.keep_weekly();
    let mut keep_monthly = policy.keep_monthly();
    let mut keep_yearly = policy.keep_yearly();
    let mut keep_last = policy.keep_last();

    let mut alive_names = IndexSet::new();
    let mut alive_dates = IndexSet::new();

    for snapshot in snapshots {
        // If user enforced limit on maximum number of overall kept snapshots, check it
        if let Some(limit) = policy.keep_limit {
            if alive_names.len() >= limit {
                break;
            }
        }

        // Otherwise check all the usual limits (hourly, daily, weekly etc.)
        for (pattern_name, pattern_format) in PATTERNS.iter() {
            let snapshot_date = format!("{}", snapshot.created_at.format(pattern_format));

            if !alive_dates.contains(&snapshot_date) {
                let keep = match pattern_name.as_ref() {
                    "hourly" => &mut keep_hourly,
                    "daily" => &mut keep_daily,
                    "weekly" => &mut keep_weekly,
                    "monthly" => &mut keep_monthly,
                    "yearly" => &mut keep_yearly,
                    _ => unreachable!(),
                };

                if *keep > 0 {
                    alive_names.insert(&snapshot.name);
                    alive_dates.insert(snapshot_date);
                    *keep -= 1;
                }
            }
        }

        // If user enforced limit on maximum number of latest kept snapshots, check it
        if keep_last > 0 && !alive_names.contains(&snapshot.name) {
            alive_names.insert(&snapshot.name);
            keep_last -= 1;
        }
    }

    alive_names
}

/// Deletes all the snapshots from specified list that are not present inside the `alive snapshots`
/// set.
fn prune_snapshots<'a>(
    lxd: &mut dyn LxdClient,
    container_name: &'a LxdContainerName,
    snapshots: &'a [LxdSnapshot],
    alive_snapshots: IndexSet<&'a LxdSnapshotName>,
) -> (usize, usize, usize) {
    let (mut deleted, mut kept, mut errors) = (0, 0, 0);

    if snapshots.is_empty() {
        println!("  (no snapshots found)");
    } else {
        for snapshot in snapshots {
            print!("  - {} ", snapshot.name);
            let _ = stdout().flush();

            if alive_snapshots.contains(&snapshot.name) {
                kept += 1;
                println!("{}", "[ KEPT ]".magenta());
            } else {
                match lxd.delete_snapshot(container_name, &snapshot.name) {
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
    }

    println!();

    (deleted, kept, errors)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local, TimeZone};

    use lib_config::*;
    use lib_lxd::*;

    use super::find_alive_snapshots;

    #[test]
    fn test_keep_hourly() {
        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-10 10:30:00"),
            snapshot("snap-4", "2000-05-10 10:25:00"),
            snapshot("snap-3", "2000-05-10 08:00:00"),
            snapshot("snap-2", "2000-05-10 07:30:00"),
            snapshot("snap-1", "2000-05-10 06:25:00"),
        ];

        let policy = Policy {
            keep_hourly: Some(4),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-5", "snap-3", "snap-2"]);
    }

    #[test]
    fn test_keep_daily() {
        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-10 12:00:00"),
            snapshot("snap-4", "2000-05-09 12:00:00"),
            snapshot("snap-3", "2000-05-09 12:00:00"),
            snapshot("snap-2", "2000-05-08 12:00:00"),
            snapshot("snap-1", "2000-05-07 12:00:00"),
        ];

        let policy = Policy {
            keep_daily: Some(4),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-4", "snap-2", "snap-1"]);
    }

    #[test]
    fn test_keep_weekly() {
        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-09 12:00:00"),
            snapshot("snap-4", "2000-05-02 12:00:00"),
            snapshot("snap-3", "2000-05-01 12:00:00"),
            snapshot("snap-2", "2000-04-25 12:00:00"),
            snapshot("snap-1", "2000-04-10 12:00:00"),
        ];

        let policy = Policy {
            keep_weekly: Some(4),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-4", "snap-2", "snap-1"]);
    }

    #[test]
    fn test_keep_monthly() {
        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-10 12:00:00"),
            snapshot("snap-4", "2000-04-01 12:00:00"),
            snapshot("snap-3", "2000-04-01 12:00:00"),
            snapshot("snap-2", "2000-03-25 12:00:00"),
            snapshot("snap-1", "2000-03-10 12:00:00"),
        ];

        let policy = Policy {
            keep_monthly: Some(2),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-4"]);
    }

    #[test]
    fn test_keep_yearly() {
        let snapshots = vec![
            snapshot("snap-6", "2010-05-10 12:00:00"),
            snapshot("snap-5", "2010-05-10 12:00:00"),
            snapshot("snap-4", "2005-04-01 12:00:00"),
            snapshot("snap-3", "2005-04-01 12:00:00"),
            snapshot("snap-2", "2000-03-25 12:00:00"),
            snapshot("snap-1", "2000-03-10 12:00:00"),
        ];

        let policy = Policy {
            keep_monthly: Some(2),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-4"]);
    }

    #[test]
    fn test_keep_last() {
        let snapshots = vec![
            snapshot("snap-6", "2010-05-10 12:00:00"),
            snapshot("snap-5", "2010-05-10 12:00:00"),
            snapshot("snap-4", "2005-04-01 12:00:00"),
            snapshot("snap-3", "2005-04-01 12:00:00"),
            snapshot("snap-2", "2000-03-25 12:00:00"),
            snapshot("snap-1", "2000-03-10 12:00:00"),
        ];

        let policy = Policy {
            keep_last: Some(3),
            ..Policy::default()
        };

        test(snapshots, policy, &["snap-6", "snap-5", "snap-4"]);
    }

    /// Performs a single test case.
    ///
    /// Checks whether for given snapshots and policy the `find_alive_snapshots` function returns
    /// correct results.
    fn test(snapshots: Vec<LxdSnapshot>, policy: Policy, expected_alive_snapshots: &[&str]) {
        let alive_snapshots: Vec<_> = find_alive_snapshots(&snapshots, &policy)
            .iter()
            .map(|snapshot_name| snapshot_name.inner().to_owned())
            .collect();

        assert_eq!(alive_snapshots, expected_alive_snapshots);
    }

    /// Creates a fake snapshot for testing purposes.
    fn snapshot(name: &str, created_at: &str) -> LxdSnapshot {
        LxdSnapshot {
            name: LxdSnapshotName::new(name),
            created_at: date(created_at),
        }
    }

    /// Creates a fake date for testing purposes.
    fn date(date: &str) -> DateTime<Local> {
        Local.datetime_from_str(date, "%Y-%m-%d %H:%M:%S").unwrap()
    }
}