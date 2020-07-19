use crate::config::Policy;
use crate::Config;
use anyhow::Result;
use lib_lxd::*;
use std::io::Write;

use self::{find_snapshots::*, find_snapshots_to_keep::*, summary::*};

mod find_snapshots;
mod find_snapshots_to_keep;
mod summary;

crate fn prune(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    PruneCmd {
        stdout,
        config,
        lxd,
    }
    .run()
}

struct PruneCmd<'a> {
    stdout: &'a mut dyn Write,
    config: &'a Config,
    lxd: &'a mut dyn LxdClient,
}

impl<'a> PruneCmd<'a> {
    fn run(mut self) -> Result<()> {
        writeln!(self.stdout, "Pruning instances:")?;

        let mut summary = PruneSummary::default();

        for project in self.lxd.list_projects()? {
            for instance in self.lxd.list(&project.name)? {
                self.try_prune_instance(&mut summary, &project, &instance)?;
            }
        }

        summary.print(self.stdout)
    }

    fn try_prune_instance(
        &mut self,
        summary: &mut PruneSummary,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<()> {
        summary.processed_instances += 1;

        writeln!(self.stdout)?;
        writeln!(self.stdout, "- {}/{}", project.name, instance.name)?;

        if let Some(policy) = self.config.policy(project, instance) {
            match self.prune_instance(project, instance, &policy) {
                Ok((deleted_snapshots, kept_snapshots)) => {
                    summary.deleted_snapshots += deleted_snapshots;
                    summary.kept_snapshots += kept_snapshots;

                    writeln!(self.stdout, "-> [ OK ]")?;
                }

                Err(err) => {
                    summary.errors += 1;

                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "error: {:?}", err)?;
                    writeln!(self.stdout)?;
                    writeln!(self.stdout, "-> [ FAILED ]")?;
                }
            }
        } else {
            writeln!(self.stdout, "-> [ EXCLUDED ]")?;
        }

        Ok(())
    }

    fn prune_instance(
        &mut self,
        project: &LxdProject,
        instance: &LxdInstance,
        policy: &Policy,
    ) -> Result<(usize, usize)> {
        let mut deleted_snapshots = 0;
        let mut kept_snapshots = 0;

        let snapshots = find_snapshots(&self.config, instance);
        let snapshots_to_keep = find_snapshots_to_keep(policy, &snapshots);

        for snapshot in &snapshots {
            if snapshots_to_keep.contains(&snapshot.name) {
                kept_snapshots += 1;

                writeln!(self.stdout, "-> keeping snapshot: {}", snapshot.name)?;
            } else {
                deleted_snapshots += 1;

                writeln!(self.stdout, "-> deleting snapshot: {}", snapshot.name)?;

                self.lxd
                    .delete_snapshot(&project.name, &instance.name, &snapshot.name)?;
            }
        }

        Ok((deleted_snapshots, kept_snapshots))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use indoc::indoc;
    use lib_lxd::test_utils::*;
    use pretty_assertions as pa;

    const POLICY: &str = indoc!(
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

        let config = Config::from_code(POLICY);

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
            //
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
            //
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

        prune(&mut stdout, &config, &mut lxd).unwrap();

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
                //
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
                //
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
            lxd.list(&LxdProjectName::default()).unwrap()
        );
    }
}
