mod summary;

use self::summary::*;
use crate::Config;
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use lib_lxd::*;
use std::io::Write;

pub fn backup(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    Command {
        time: Utc::now,
        stdout,
        config,
        lxd,
    }
    .run()
}

struct Command<'a> {
    time: fn() -> DateTime<Utc>,
    stdout: &'a mut dyn Write,
    config: &'a Config,
    lxd: &'a mut dyn LxdClient,
}

impl<'a> Command<'a> {
    fn run(mut self) -> Result<()> {
        writeln!(self.stdout, "Backing-up instances:")?;

        let mut summary = Summary::default();

        for project in self.lxd.list_projects()? {
            for instance in self.lxd.list(&project.name)? {
                self.try_backup_instance(&mut summary, &project, &instance)?;
            }
        }

        summary.print(self.stdout)
    }

    fn try_backup_instance(
        &mut self,
        summary: &mut Summary,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<()> {
        summary.processed_instances += 1;

        writeln!(self.stdout)?;
        writeln!(self.stdout, "- {}/{} ", project.name, instance.name)?;

        if self.config.policy(project, instance).is_some() {
            match self.backup_instance(project, instance) {
                Ok(_) => {
                    summary.created_snapshots += 1;

                    writeln!(self.stdout, "-> [ OK ]")?;
                }

                Err(err) => {
                    summary.errors += 1;

                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "{} {:?}", "error:".red(), err)?;
                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "-> [ FAILED ]")?;
                }
            }
        } else {
            writeln!(self.stdout, "-> [ EXCLUDED ]")?;
        }

        Ok(())
    }

    fn backup_instance(
        &mut self,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<LxdSnapshotName> {
        let snapshot_name = self.config.snapshot_name((self.time)());

        writeln!(self.stdout, "-> creating snapshot: {}", snapshot_name)?;

        self.lxd
            .create_snapshot(&project.name, &instance.name, &snapshot_name)?;

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
            excluded-instances: ['instance-b']
            included-statuses: ['Running']
        "#
    );

    fn backup(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
        Command {
            time: || Utc.timestamp(0, 0),
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

        let mut lxd = LxdFakeClient::new(vec![
            LxdInstance {
                name: instance_name("instance-a"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            LxdInstance {
                name: instance_name("instance-b"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            LxdInstance {
                name: instance_name("instance-c"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("instance-d"),
                status: LxdInstanceStatus::Stopped,
                snapshots: Default::default(),
            },
        ]);

        backup(&mut stdout, &config, &mut lxd).unwrap();

        assert_out!(
            r#"
            Backing-up instances:
            
            - default/instance-a 
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]
            
            - default/instance-b 
            -> [ EXCLUDED ]
            
            - default/instance-c 
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]
            
            - default/instance-d 
            -> [ EXCLUDED ]
            
            Summary
            - processed instances: 4
            - created snapshots: 2
            "#,
            stdout
        );

        let instances = lxd.list(&LxdProjectName::default()).unwrap();

        assert_eq!(4, instances.len());
        assert_eq!(2, instances[0].snapshots.len());
        assert_eq!(1, instances[1].snapshots.len());
        assert_eq!(1, instances[2].snapshots.len());
        assert_eq!(0, instances[3].snapshots.len());
    }
}
