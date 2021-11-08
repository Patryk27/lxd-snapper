use crate::prelude::*;

pub struct DebugNuke<'a, 'b> {
    env: &'a mut Environment<'b>,
}

impl<'a, 'b> DebugNuke<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self { env }
    }

    pub fn run(self) -> Result<()> {
        writeln!(self.env.stdout, "Nuking instances:")?;

        for project in self.env.lxd.list_projects()? {
            for instance in self.env.lxd.list(&project.name)? {
                if !self.env.config.policies.matches(&project, &instance) {
                    continue;
                }

                writeln!(self.env.stdout)?;
                writeln!(self.env.stdout, "- {}/{}", project.name, instance.name)?;

                for snapshot in instance.snapshots {
                    writeln!(self.env.stdout, "-> deleting snapshot: {}", snapshot.name)?;

                    self.env
                        .lxd
                        .delete_snapshot(&project.name, &instance.name, &snapshot.name)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use lib_lxd::{test_utils::*, *};

    fn instances() -> Vec<LxdInstance> {
        vec![
            LxdInstance {
                name: instance_name("instance-a"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            LxdInstance {
                name: instance_name("instance-b"),
                status: LxdInstanceStatus::Running,
                snapshots: vec![
                    snapshot("snapshot-1", "2000-01-01 12:00:00"),
                    snapshot("snapshot-2", "2000-01-01 13:00:00"),
                ],
            },
            LxdInstance {
                name: instance_name("instance-c"),
                status: LxdInstanceStatus::Stopping,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("instance-d"),
                status: LxdInstanceStatus::Stopped,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
        ]
    }

    mod given_empty_policy {
        use super::*;

        #[test]
        fn deletes_no_snapshots() {
            let mut stdout = Vec::new();
            let config = Config::default();
            let mut lxd = LxdFakeClient::new(instances());

            DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                .run()
                .unwrap();

            assert_out!(
                r#"
                Nuking instances:
                "#,
                stdout
            );

            let actual_instances = lxd.list(&LxdProjectName::default()).unwrap();
            let expected_instances = instances();

            pa::assert_eq!(expected_instances, actual_instances);
        }
    }

    mod given_some_policy {
        use super::*;

        const CONFIG: &str = indoc!(
            r#"
            policies:
              main:
                included-statuses: ['Running']
            "#
        );

        #[test]
        fn deletes_snapshots_only_for_instances_matching_that_policy() {
            let mut stdout = Vec::new();
            let config = Config::from_code(CONFIG);
            let mut lxd = LxdFakeClient::new(instances());

            DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                .run()
                .unwrap();

            assert_out!(
                r#"
                Nuking instances:
                
                - default/instance-a
                -> deleting snapshot: snapshot-1
                
                - default/instance-b
                -> deleting snapshot: snapshot-1
                -> deleting snapshot: snapshot-2
                "#,
                stdout
            );

            pa::assert_eq!(
                vec![
                    LxdInstance {
                        name: instance_name("instance-a"),
                        status: LxdInstanceStatus::Running,
                        snapshots: Default::default(),
                    },
                    LxdInstance {
                        name: instance_name("instance-b"),
                        status: LxdInstanceStatus::Running,
                        snapshots: Default::default(),
                    },
                    LxdInstance {
                        name: instance_name("instance-c"),
                        status: LxdInstanceStatus::Stopping,
                        snapshots: Default::default(),
                    },
                    LxdInstance {
                        name: instance_name("instance-d"),
                        status: LxdInstanceStatus::Stopped,
                        snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
                    },
                ],
                lxd.list(&LxdProjectName::default()).unwrap()
            );
        }
    }
}
