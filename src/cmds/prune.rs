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
        writeln!(self.stdout, "Pruning containers:")?;

        let mut summary = PruneSummary::default();

        for project in self.lxd.list_projects()? {
            for container in self.lxd.list(&project.name)? {
                self.try_prune_container(&mut summary, &project, &container)?;
            }
        }

        summary.print(self.stdout)
    }

    fn try_prune_container(
        &mut self,
        summary: &mut PruneSummary,
        project: &LxdProject,
        container: &LxdContainer,
    ) -> Result<()> {
        summary.processed_containers += 1;

        writeln!(self.stdout)?;
        writeln!(self.stdout, "- {}/{}", project.name, container.name)?;

        if let Some(policy) = self.config.policy(project, container) {
            match self.prune_container(project, container, &policy) {
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

    fn prune_container(
        &mut self,
        project: &LxdProject,
        container: &LxdContainer,
        policy: &Policy,
    ) -> Result<(usize, usize)> {
        let mut deleted_snapshots = 0;
        let mut kept_snapshots = 0;

        let snapshots = find_snapshots(&self.config, container);
        let snapshots_to_keep = find_snapshots_to_keep(policy, &snapshots);

        for snapshot in &snapshots {
            if snapshots_to_keep.contains(&snapshot.name) {
                kept_snapshots += 1;

                writeln!(self.stdout, "-> keeping snapshot: {}", snapshot.name)?;
            } else {
                deleted_snapshots += 1;

                writeln!(self.stdout, "-> deleting snapshot: {}", snapshot.name)?;

                self.lxd
                    .delete_snapshot(&project.name, &container.name, &snapshot.name)?;
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
            excluded-containers: ['container-b']
            keep-last: 2
        "#
    );

    #[test]
    fn test() {
        let mut stdout = Vec::new();

        let config = Config::from_code(POLICY);

        let mut lxd = LxdDummyClient::new(vec![
            LxdContainer {
                name: container_name("container-a"),
                status: LxdContainerStatus::Running,
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
            LxdContainer {
                name: container_name("container-b"),
                status: LxdContainerStatus::Running,
                snapshots: vec![
                    snapshot("manual-1", "2000-01-01 12:00:00"),
                    snapshot("auto-1", "2000-01-01 13:00:00"),
                    snapshot("auto-2", "2000-01-01 14:00:00"),
                    snapshot("auto-3", "2000-01-01 15:00:00"),
                    snapshot("manual-2", "2000-01-01 16:00:00"),
                ],
            },
            //
            LxdContainer {
                name: container_name("container-c"),
                status: LxdContainerStatus::Running,
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
            Pruning containers:
            
            - default/container-a
            -> keeping snapshot: auto-4
            -> keeping snapshot: auto-3
            -> deleting snapshot: auto-2
            -> deleting snapshot: auto-1
            -> [ OK ]
            
            - default/container-b
            -> [ EXCLUDED ]
            
            - default/container-c
            -> keeping snapshot: auto-2
            -> keeping snapshot: auto-1
            -> [ OK ]
            
            Summary
            - processed containers: 3
            - deleted snapshots: 2
            - kept snapshots: 4
            "#,
            stdout
        );

        pa::assert_eq!(
            vec![
                LxdContainer {
                    name: container_name("container-a"),
                    status: LxdContainerStatus::Running,
                    snapshots: vec![
                        snapshot("manual-1", "2000-01-01 12:00:00"),
                        snapshot("auto-3", "2000-01-01 15:00:00"),
                        snapshot("auto-4", "2000-01-01 16:00:00"),
                        snapshot("manual-2", "2000-01-01 17:00:00"),
                    ],
                },
                //
                LxdContainer {
                    name: container_name("container-b"),
                    status: LxdContainerStatus::Running,
                    snapshots: vec![
                        snapshot("manual-1", "2000-01-01 12:00:00"),
                        snapshot("auto-1", "2000-01-01 13:00:00"),
                        snapshot("auto-2", "2000-01-01 14:00:00"),
                        snapshot("auto-3", "2000-01-01 15:00:00"),
                        snapshot("manual-2", "2000-01-01 16:00:00"),
                    ],
                },
                //
                LxdContainer {
                    name: container_name("container-c"),
                    status: LxdContainerStatus::Running,
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
