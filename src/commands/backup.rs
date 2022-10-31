use crate::prelude::*;

pub struct Backup<'a, 'b> {
    env: &'a mut Environment<'b>,
    summary: Summary,
}

impl<'a, 'b> Backup<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self {
            env,
            summary: Summary::default().with_created_snapshots(),
        }
    }

    pub fn with_summary_title(mut self, title: &'static str) -> Self {
        self.summary.set_title(title);
        self
    }

    pub fn run(mut self) -> Result<()> {
        self.env.config.hooks().on_backup_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks().on_backup_completed();

        cmd_result.and(hook_result)
    }

    fn try_run(&mut self) -> Result<()> {
        if self.env.config.remotes().has_any_non_local_remotes() {
            for remote in self.env.config.remotes().iter() {
                self.process_remote(true, remote)
                    .with_context(|| format!("Couldn't process remote: {}", remote.name()))?;
            }
        } else {
            self.process_remote(false, &Remote::local())?;
        }

        write!(self.env.stdout, "{}", self.summary)?;

        if self.summary.has_errors() {
            bail!("Failed to backup some of the instances");
        }

        self.summary.as_result()
    }

    fn process_remote(&mut self, print_remote: bool, remote: &Remote) -> Result<()> {
        let projects = self
            .env
            .lxd
            .projects(remote.name())
            .context("Couldn't list projects")?;

        let print_project = projects.iter().any(|project| !project.name.is_default());

        for project in projects {
            self.process_project(print_remote, remote, print_project, &project)
                .with_context(|| format!("Couldn't process project: {}", project.name))?;
        }

        Ok(())
    }

    fn process_project(
        &mut self,
        print_remote: bool,
        remote: &Remote,
        print_project: bool,
        project: &LxdProject,
    ) -> Result<()> {
        let instances = self
            .env
            .lxd
            .instances(remote.name(), &project.name)
            .context("Couldn't list instances")?;

        for instance in instances {
            self.process_instance(print_remote, remote, print_project, project, &instance)
                .with_context(|| format!("Couldn't process instance: {}", instance.name))?;
        }

        Ok(())
    }

    fn process_instance(
        &mut self,
        print_remote: bool,
        remote: &Remote,
        print_project: bool,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<()> {
        writeln!(
            self.env.stdout,
            "{}",
            PrettyLxdInstanceName::new(
                print_remote,
                remote.name(),
                print_project,
                &project.name,
                &instance.name
            )
            .to_string()
            .bold()
        )?;

        if self
            .env
            .config
            .policies()
            .matches(remote.name(), project, instance)
        {
            match self.try_process_instance(remote, project, instance) {
                Ok(_) => {
                    self.summary.add_created_snapshot();

                    writeln!(self.env.stdout, " {}", "[ OK ]".green())?;
                }

                Err(err) => {
                    self.summary.add_error();

                    writeln!(self.env.stdout, " {}", "[ FAILED ]".red())?;
                    writeln!(self.env.stdout)?;

                    let err = format!("{:?}", err);

                    for line in err.lines() {
                        writeln!(self.env.stdout, "  {}", line)?;
                    }
                }
            }
        } else {
            writeln!(self.env.stdout, "  - {}", "[ EXCLUDED ]".yellow())?;
        }

        writeln!(self.env.stdout)?;

        Ok(())
    }

    fn try_process_instance(
        &mut self,
        remote: &Remote,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<LxdSnapshotName> {
        self.summary.add_processed_instance();

        let snapshot_name = self.env.config.snapshot_name(self.env.time());

        write!(
            self.env.stdout,
            "  - creating snapshot: {}",
            snapshot_name.as_str().italic()
        )?;

        self.env
            .lxd
            .create_snapshot(remote.name(), &project.name, &instance.name, &snapshot_name)
            .context("Couldn't create snapshot")?;

        self.env.config.hooks().on_snapshot_created(
            remote.name(),
            &project.name,
            &instance.name,
            &snapshot_name,
        )?;

        Ok(snapshot_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::{utils::*, LxdFakeClient, LxdInstanceStatus};
    use crate::{assert_lxd, assert_result, assert_stdout};

    #[test]
    fn smoke() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                excluded-instances: ['mariadb']
                included-statuses: ['Running']
            "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "elastic",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mariadb",
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mongodb",
            status: LxdInstanceStatus::Stopped,
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "postgresql",
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>elastic</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>mariadb</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>mongodb</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>postgresql</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>Summary</b>
            -------
              processed instances: 2
              created snapshots: 2
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            local:default/elastic (Running)
            -> auto-19700101-000000

            local:default/mariadb (Running)
            -> snapshot-1

            local:default/mongodb (Stopped)

            local:default/postgresql (Running)
            -> snapshot-1
            -> auto-19700101-000000
            "#,
            lxd
        );
    }

    #[test]
    fn smoke_with_remotes() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                excluded-remotes: ['db-3']
                included-statuses: ['Running']

            remotes:
              - local
              - db-1
              - db-2
              - db-3
              - db-4
            "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            remote: "db-1",
            name: "postgresql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-2",
            name: "postgresql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-3",
            name: "postgresql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-4",
            name: "postgresql",
            status: LxdInstanceStatus::Stopping,
            ..Default::default()
        });

        Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>db-1:postgresql</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>db-2:postgresql</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>db-3:postgresql</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>db-4:postgresql</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>Summary</b>
            -------
              processed instances: 2
              created snapshots: 2
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            db-1:default/postgresql (Running)
            -> auto-19700101-000000

            db-2:default/postgresql (Running)
            -> auto-19700101-000000

            db-3:default/postgresql (Running)

            db-4:default/postgresql (Stopping)
            "#,
            lxd
        );
    }

    #[test]
    fn failed_snapshot() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
            "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "elastic",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mariadb",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "postgresql",
            ..Default::default()
        });

        lxd.inject_error(LxdFakeError::OnCreateSnapshot {
            remote: "local",
            project: "default",
            instance: "mariadb",
            snapshot: "auto-19700101-000000",
        });

        let result = Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd)).run();

        assert_stdout!(
            r#"
            <b>elastic</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>mariadb</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=31>[ FAILED ]</fg>

              Couldn't create snapshot

              Caused by:
                  InjectedError

            <b>postgresql</b>
              - creating snapshot: <i>auto-19700101-000000</i> <fg=32>[ OK ]</fg>

            <b>Summary</b>
            -------
              processed instances: 3
              created snapshots: 2
            "#,
            stdout
        );

        assert_result!("Failed to backup some of the instances", result);

        assert_lxd!(
            r#"
            local:default/elastic (Running)
            -> auto-19700101-000000

            local:default/mariadb (Running)

            local:default/postgresql (Running)
            -> auto-19700101-000000
            "#,
            lxd
        );
    }
}
