use crate::prelude::*;

pub struct DebugNuke<'a, 'b> {
    env: &'a mut Environment<'b>,
}

impl<'a, 'b> DebugNuke<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self { env }
    }

    pub fn run(self) -> Result<()> {
        let has_remotes = self.env.config.remotes().has_any_non_local_remotes();

        writeln!(self.env.stdout, "Nuking instances:")?;

        for remote in self.env.config.remotes().iter() {
            for project in self.env.lxd.projects(remote.name())? {
                for instance in self.env.lxd.instances(remote.name(), &project.name)? {
                    if !self
                        .env
                        .config
                        .policies()
                        .matches(remote.name(), &project, &instance)
                    {
                        continue;
                    }

                    writeln!(self.env.stdout)?;

                    if has_remotes {
                        writeln!(
                            self.env.stdout,
                            "- {}:{}/{}",
                            remote.name(),
                            project.name,
                            instance.name
                        )?;
                    } else {
                        writeln!(self.env.stdout, "- {}/{}", project.name, instance.name)?;
                    }

                    for snapshot in instance.snapshots {
                        writeln!(self.env.stdout, "-> deleting snapshot: {}", snapshot.name)?;

                        self.env.lxd.delete_snapshot(
                            remote.name(),
                            &project.name,
                            &instance.name,
                            &snapshot.name,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::{utils::*, *};
    use crate::{assert_lxd, assert_out};

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

    fn lxd() -> LxdFakeClient {
        let mut lxd = LxdFakeClient::default();

        for instance in instances() {
            lxd.add(LxdFakeInstance {
                name: instance.name.as_str(),
                status: instance.status,
                snapshots: instance.snapshots,
                ..Default::default()
            });
        }

        lxd
    }

    #[test]
    fn given_empty_policy() {
        let mut stdout = Vec::new();
        let config = Config::default();
        let mut lxd = lxd();

        DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_out!(
            r#"
            Nuking instances:
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            local:default/instance-a (Running)
            -> snapshot-1

            local:default/instance-b (Running)
            -> snapshot-1
            -> snapshot-2

            local:default/instance-c (Stopping)

            local:default/instance-d (Stopped)
            -> snapshot-1
            "#,
            lxd
        );
    }

    #[test]
    fn given_policy_without_remotes() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                included-statuses: ['Running']
            "#
        ));

        let mut lxd = lxd();

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

        assert_lxd!(
            r#"
            local:default/instance-a (Running)

            local:default/instance-b (Running)

            local:default/instance-c (Stopping)

            local:default/instance-d (Stopped)
            -> snapshot-1
            "#,
            lxd
        );
    }

    #[test]
    fn given_policy_with_remotes() {
        // TODO
    }
}
