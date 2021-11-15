mod find_snapshots;
mod find_snapshots_to_keep;
mod summary;

use self::{find_snapshots::*, find_snapshots_to_keep::*, summary::*};
use crate::prelude::*;
use lib_lxd::{LxdInstance, LxdProject};

pub struct Prune<'a, 'b> {
    env: &'a mut Environment<'b>,
    summary: Summary,
}

impl<'a, 'b> Prune<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self {
            env,
            summary: Summary::default(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.env.config.hooks.on_prune_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks.on_prune_completed();

        cmd_result.and(hook_result)
    }

    fn try_run(&mut self) -> Result<()> {
        writeln!(self.env.stdout, "Pruning instances:")?;

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

        if let Some(policy) = self.env.config.policies.build(project, instance) {
            match self.try_process_intance(project, instance, &policy) {
                Ok((deleted_snapshots, kept_snapshots)) => {
                    self.summary.deleted_snapshots += deleted_snapshots;
                    self.summary.kept_snapshots += kept_snapshots;

                    writeln!(self.env.stdout, "-> [ OK ]")?;
                }

                Err(err) => {
                    self.summary.errors += 1;

                    writeln!(self.env.stdout)?;
                    writeln!(self.env.stdout, "error: {:?}", err)?;
                    writeln!(self.env.stdout)?;
                    writeln!(self.env.stdout, "-> [ FAILED ]")?;
                }
            }
        } else {
            writeln!(self.env.stdout, "-> [ EXCLUDED ]")?;
        }

        Ok(())
    }

    fn try_process_intance(
        &mut self,
        project: &LxdProject,
        instance: &LxdInstance,
        policy: &Policy,
    ) -> Result<(usize, usize)> {
        let mut deleted_snapshots = 0;
        let mut kept_snapshots = 0;

        let snapshots = find_snapshots(self.env.config, instance);
        let snapshots_to_keep = find_snapshots_to_keep(policy, &snapshots);

        for snapshot in &snapshots {
            if snapshots_to_keep.contains(&snapshot.name) {
                kept_snapshots += 1;

                writeln!(self.env.stdout, "-> keeping snapshot: {}", snapshot.name)?;
            } else {
                deleted_snapshots += 1;

                writeln!(self.env.stdout, "-> deleting snapshot: {}", snapshot.name)?;

                self.env
                    .lxd
                    .delete_snapshot(&project.name, &instance.name, &snapshot.name)?;

                self.env
                    .config
                    .hooks
                    .on_snapshot_deleted(&project.name, &instance.name, &snapshot.name)
                    .context("Couldn't execute the `on-snapshot-deleted` hook")?;
            }
        }

        Ok((deleted_snapshots, kept_snapshots))
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
            keep-last: 2
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
                snapshots: vec![
                    snapshot("manual-1", "2000-01-01 12:00:00"),
                    snapshot("auto-1", "2000-01-01 13:00:00"),
                    snapshot("auto-2", "2000-01-01 14:00:00"),
                    snapshot("auto-3", "2000-01-01 15:00:00"),
                    snapshot("auto-4", "2000-01-01 16:00:00"),
                    snapshot("manual-2", "2000-01-01 17:00:00"),
                ],
            },
            LxdInstance {
                name: instance_name("instance-b"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![
                    snapshot("manual-1", "2000-01-01 12:00:00"),
                    snapshot("auto-1", "2000-01-01 13:00:00"),
                    snapshot("auto-2", "2000-01-01 14:00:00"),
                    snapshot("auto-3", "2000-01-01 15:00:00"),
                    snapshot("manual-2", "2000-01-01 16:00:00"),
                ],
            },
            LxdInstance {
                name: instance_name("instance-c"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![
                    snapshot("manual-1", "2000-01-01 12:00:00"),
                    snapshot("auto-1", "2000-01-01 13:00:00"),
                    snapshot("auto-2", "2000-01-01 14:00:00"),
                    snapshot("manual-2", "2000-01-01 15:00:00"),
                ],
            },
        ]);

        Prune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_out!(
            r#"
            Pruning instances:
            
            - default/instance-a
            -> keeping snapshot: auto-4
            -> keeping snapshot: auto-3
            -> deleting snapshot: auto-2
            -> deleting snapshot: auto-1
            -> [ OK ]
            
            - default/instance-b
            -> [ EXCLUDED ]
            
            - default/instance-c
            -> keeping snapshot: auto-2
            -> keeping snapshot: auto-1
            -> [ OK ]
            
            Summary
            - processed instances: 3
            - deleted snapshots: 2
            - kept snapshots: 4
            "#,
            stdout
        );

        pa::assert_eq!(
            vec![
                LxdInstance {
                    name: instance_name("instance-a"),
                    status: LxdInstanceStatus::Running,
                    snapshots: vec![
                        snapshot("manual-1", "2000-01-01 12:00:00"),
                        snapshot("auto-3", "2000-01-01 15:00:00"),
                        snapshot("auto-4", "2000-01-01 16:00:00"),
                        snapshot("manual-2", "2000-01-01 17:00:00"),
                    ],
                },
                LxdInstance {
                    name: instance_name("instance-b"),
                    status: LxdInstanceStatus::Running,
                    snapshots: vec![
                        snapshot("manual-1", "2000-01-01 12:00:00"),
                        snapshot("auto-1", "2000-01-01 13:00:00"),
                        snapshot("auto-2", "2000-01-01 14:00:00"),
                        snapshot("auto-3", "2000-01-01 15:00:00"),
                        snapshot("manual-2", "2000-01-01 16:00:00"),
                    ],
                },
                LxdInstance {
                    name: instance_name("instance-c"),
                    status: LxdInstanceStatus::Running,
                    snapshots: vec![
                        snapshot("manual-1", "2000-01-01 12:00:00"),
                        snapshot("auto-1", "2000-01-01 13:00:00"),
                        snapshot("auto-2", "2000-01-01 14:00:00"),
                        snapshot("manual-2", "2000-01-01 15:00:00"),
                    ],
                },
            ],
            lxd.list(&Default::default()).unwrap()
        );
    }
}
