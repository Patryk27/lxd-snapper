use crate::Config;
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use lib_lxd::*;
use std::io::Write;

use self::summary::*;

mod summary;

crate fn backup(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    BackupCmd {
        time: Utc::now,
        stdout,
        config,
        lxd,
    }
    .run()
}

struct BackupCmd<'a> {
    time: fn() -> DateTime<Utc>,
    stdout: &'a mut dyn Write,
    config: &'a Config,
    lxd: &'a mut dyn LxdClient,
}

impl<'a> BackupCmd<'a> {
    fn run(mut self) -> Result<()> {
        writeln!(self.stdout, "Backing-up containers:")?;

        let mut summary = BackupSummary::default();

        for project in self.lxd.list_projects()? {
            for container in self.lxd.list(&project.name)? {
                self.try_backup_container(&mut summary, &project, &container)?;
            }
        }

        summary.print(self.stdout)
    }

    fn try_backup_container(
        &mut self,
        summary: &mut BackupSummary,
        project: &LxdProject,
        container: &LxdContainer,
    ) -> Result<()> {
        summary.processed_containers += 1;

        writeln!(self.stdout)?;
        writeln!(self.stdout, "- {}/{} ", project.name, container.name)?;

        if self.config.policy(&project, &container).is_some() {
            match self.backup_container(&project, &container) {
                Ok(_) => {
                    summary.created_snapshots += 1;

                    writeln!(self.stdout, "-> [ OK ]")?;
                }

                Err(err) => {
                    summary.errors += 1;

                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "{}: {:?}", "Error".red(), err)?;
                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "-> [ FAILED ]")?;
                }
            }
        } else {
            writeln!(self.stdout, "-> [ EXCLUDED ]")?;
        }

        Ok(())
    }

    fn backup_container(
        &mut self,
        project: &LxdProject,
        container: &LxdContainer,
    ) -> Result<LxdSnapshotName> {
        let snapshot_name = self.config.snapshot_name((self.time)());

        writeln!(self.stdout, "-> creating snapshot: {}", snapshot_name)?;

        self.lxd
            .create_snapshot(&project.name, &container.name, &snapshot_name)?;

        Ok(snapshot_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use chrono::TimeZone;
    use indoc::indoc;
    use lib_lxd::test_utils::*;

    const POLICY: &str = indoc!(
        r#"
        policies:
          main:
            excluded-containers: ['container-b']
            included-statuses: ['Running']
        "#
    );

    fn backup(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
        fn time() -> DateTime<Utc> {
            Utc.timestamp(0, 0)
        }

        BackupCmd {
            time,
            stdout,
            config,
            lxd,
        }
        .run()
    }

    #[test]
    fn test() {
        let mut stdout = Vec::new();

        let config = Config::from_code(POLICY);

        let mut lxd = LxdDummyClient::new(vec![
            LxdContainer {
                name: container_name("container-a"),
                status: LxdContainerStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            //
            LxdContainer {
                name: container_name("container-b"),
                status: LxdContainerStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            //
            LxdContainer {
                name: container_name("container-c"),
                status: LxdContainerStatus::Running,
                snapshots: Default::default(),
            },
            //
            LxdContainer {
                name: container_name("container-d"),
                status: LxdContainerStatus::Stopped,
                snapshots: Default::default(),
            },
        ]);

        backup(&mut stdout, &config, &mut lxd).unwrap();

        assert_out!(
            r#"
            Backing-up containers:
            
            - default/container-a 
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]
            
            - default/container-b 
            -> [ EXCLUDED ]
            
            - default/container-c 
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]
            
            - default/container-d 
            -> [ EXCLUDED ]
            
            Summary
            - processed containers: 4
            - created snapshots: 2
            "#,
            stdout
        );

        let containers = lxd.list(&LxdProjectName::default()).unwrap();

        // It's quite hard to compare containers 1:1 (like on other tests), because the
        // newly-created snapshots have `created_at` equal to `now`
        assert_eq!(4, containers.len());
        assert_eq!(2, containers[0].snapshots.len());
        assert_eq!(1, containers[1].snapshots.len());
        assert_eq!(1, containers[2].snapshots.len());
        assert_eq!(0, containers[3].snapshots.len());
    }
}
