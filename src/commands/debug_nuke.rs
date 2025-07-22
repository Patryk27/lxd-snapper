use crate::prelude::*;

pub struct DebugNuke<'a, 'b> {
    env: &'a mut Environment<'b>,
}

impl<'a, 'b> DebugNuke<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self { env }
    }

    pub fn run(self) -> Result<()> {
        let mut summary = Summary::default().with_deleted_snapshots();

        for remote in self.env.config.remotes().iter() {
            for project in self.env.client.projects(remote)? {
                for instance in self.env.client.instances(remote, &project.name)? {
                    writeln!(
                        self.env.stdout,
                        "{}",
                        format!("- {}:{}/{}", remote, project.name, instance.name).bold(),
                    )?;

                    if !self
                        .env
                        .config
                        .policies()
                        .matches(remote, &project, &instance)
                    {
                        writeln!(self.env.stdout, "  - {}", "[ EXCLUDED ]".yellow())?;
                        writeln!(self.env.stdout)?;

                        continue;
                    }

                    summary.add_processed_instance();

                    for snapshot in instance.snapshots {
                        write!(
                            self.env.stdout,
                            "  - deleting snapshot: {}",
                            snapshot.name.as_str().italic()
                        )?;

                        let result = self.env.client.delete_snapshot(
                            remote,
                            &project.name,
                            &instance.name,
                            &snapshot.name,
                        );

                        match result {
                            Ok(_) => {
                                summary.add_deleted_snapshot();

                                writeln!(self.env.stdout, " {}", "[ OK ]".green())?;
                            }

                            Err(err) => {
                                writeln!(self.env.stdout, " {}", "[ FAILED ]".red())?;

                                return Err(err.into());
                            }
                        }
                    }

                    writeln!(self.env.stdout)?;
                }
            }
        }

        write!(self.env.stdout, "{}", summary)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::{utils::*, *};
    use crate::{assert_lxd, assert_stdout};

    fn client() -> LxdFakeClient {
        let mut client = LxdFakeClient::default();

        client.add(LxdFakeInstance {
            name: "instance-a",
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            name: "instance-b",
            snapshots: vec![
                snapshot("snapshot-1", "2000-01-01 12:00:00"),
                snapshot("snapshot-2", "2000-01-01 13:00:00"),
            ],
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            name: "instance-c",
            status: LxdInstanceStatus::Stopping,
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            name: "instance-d",
            status: LxdInstanceStatus::Stopped,
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            name: "instance-d",
            remote: "remote",
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        client
    }

    #[test]
    fn smoke() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                included-statuses: ['Running']
            "#
        ));

        let mut client = client();

        DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut client))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>- local:default/instance-a</b>
              - deleting snapshot: <i>snapshot-1</i> <fg=32>[ OK ]</fg>

            <b>- local:default/instance-b</b>
              - deleting snapshot: <i>snapshot-1</i> <fg=32>[ OK ]</fg>
              - deleting snapshot: <i>snapshot-2</i> <fg=32>[ OK ]</fg>

            <b>- local:default/instance-c</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- local:default/instance-d</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>Summary</b>
            -------
              processed instances: 2
              deleted snapshots: 3
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

            remote:default/instance-d (Running)
            -> snapshot-1
            "#,
            client
        );
    }

    #[test]
    fn smoke_with_remotes() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                included-statuses: ['Running']

            remotes:
              - local
              - remote
            "#
        ));

        let mut client = client();

        DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut client))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>- local:default/instance-a</b>
              - deleting snapshot: <i>snapshot-1</i> <fg=32>[ OK ]</fg>

            <b>- local:default/instance-b</b>
              - deleting snapshot: <i>snapshot-1</i> <fg=32>[ OK ]</fg>
              - deleting snapshot: <i>snapshot-2</i> <fg=32>[ OK ]</fg>

            <b>- local:default/instance-c</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- local:default/instance-d</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- remote:default/instance-d</b>
              - deleting snapshot: <i>snapshot-1</i> <fg=32>[ OK ]</fg>

            <b>Summary</b>
            -------
              processed instances: 3
              deleted snapshots: 4
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

            remote:default/instance-d (Running)
            "#,
            client
        );
    }

    #[test]
    fn empty_policy() {
        let mut stdout = Vec::new();
        let config = Config::default();
        let mut client = client();

        DebugNuke::new(&mut Environment::test(&mut stdout, &config, &mut client))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>- local:default/instance-a</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- local:default/instance-b</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- local:default/instance-c</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>- local:default/instance-d</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>Summary</b>
            -------
              processed instances: 0
              deleted snapshots: 0
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

            remote:default/instance-d (Running)
            -> snapshot-1
            "#,
            client
        );
    }
}
