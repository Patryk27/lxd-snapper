mod summary;

use self::summary::*;
use crate::prelude::*;
use lib_lxd::{LxdInstance, LxdProject, LxdSnapshotName};

pub struct Backup<'a, 'b> {
    env: &'a mut Environment<'b>,
    summary: Summary,
}

impl<'a, 'b> Backup<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self {
            env,
            summary: Default::default(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.env.config.hooks.on_backup_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks.on_backup_completed();

        cmd_result.and(hook_result)
    }

    fn try_run(&mut self) -> Result<()> {
        writeln!(self.env.stdout, "Backing-up instances:")?;

        let projects = self
            .env
            .lxd
            .list_projects()
            .context("Couldn't list projects")?;

        for project in projects {
            self.process_project(&project)
                .with_context(|| format!("Couldn't process project: {}", project.name))?;
        }

        self.summary.print(self.env.stdout)?;

        Ok(())
    }

    fn process_project(&mut self, project: &LxdProject) -> Result<()> {
        let instances = self
            .env
            .lxd
            .list(&project.name)
            .context("Couldn't list instances")?;

        for instance in instances {
            self.process_instance(project, &instance)
                .with_context(|| format!("Couldn't process instance: {}", instance.name))?;
        }

        Ok(())
    }

    fn process_instance(&mut self, project: &LxdProject, instance: &LxdInstance) -> Result<()> {
        self.summary.processed_instances += 1;

        writeln!(self.env.stdout)?;
        writeln!(self.env.stdout, "- {}/{}", project.name, instance.name)?;

        if self.env.config.policies.matches(project, instance) {
            match self.try_process_instance(project, instance) {
                Ok(_) => {
                    self.summary.created_snapshots += 1;

                    writeln!(self.env.stdout, "-> [ OK ]")?;
                }

                Err(err) => {
                    self.summary.errors += 1;

                    writeln!(self.env.stdout)?;
                    writeln!(self.env.stdout, "{} {:?}", "error:".red(), err)?;
                    writeln!(self.env.stdout)?;
                    writeln!(self.env.stdout, "-> [ FAILED ]")?;
                }
            }
        } else {
            writeln!(self.env.stdout, "-> [ EXCLUDED ]")?;
        }

        Ok(())
    }

    fn try_process_instance(
        &mut self,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<LxdSnapshotName> {
        let snapshot_name = self.env.config.snapshot_name(self.env.time());

        writeln!(self.env.stdout, "-> creating snapshot: {}", snapshot_name)?;

        self.env
            .lxd
            .create_snapshot(&project.name, &instance.name, &snapshot_name)
            .context("Couldn't create snapshot")?;

        self.env
            .config
            .hooks
            .on_snapshot_created(&project.name, &instance.name, &snapshot_name)?;

        Ok(snapshot_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use lib_lxd::{test_utils::*, LxdClient, LxdFakeClient, LxdInstanceStatus};

    const CONFIG: &str = indoc!(
        r#"
        policies:
          main:
            excluded-instances: ['instance-b']
            included-statuses: ['Running']
        "#
    );

    #[test]
    fn test() {
        let mut stdout = Vec::new();
        let config = Config::from_code(CONFIG);

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

        Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

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

        let instances = lxd.list(&Default::default()).unwrap();

        assert_eq!(4, instances.len());
        assert_eq!(2, instances[0].snapshots.len());
        assert_eq!(1, instances[1].snapshots.len());
        assert_eq!(1, instances[2].snapshots.len());
        assert_eq!(0, instances[3].snapshots.len());
    }
}
