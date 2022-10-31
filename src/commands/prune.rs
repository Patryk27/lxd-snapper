mod find_snapshots;
mod find_snapshots_to_keep;

use self::{find_snapshots::*, find_snapshots_to_keep::*};
use crate::prelude::*;

pub struct Prune<'a, 'b> {
    env: &'a mut Environment<'b>,
    summary: Summary,
}

impl<'a, 'b> Prune<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self {
            env,
            summary: Summary::default()
                .with_deleted_snapshots()
                .with_kept_snapshots(),
        }
    }

    pub fn with_summary_title(mut self, title: &'static str) -> Self {
        self.summary.set_title(title);
        self
    }

    pub fn run(mut self) -> Result<()> {
        self.env.config.hooks().on_prune_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks().on_prune_completed();

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
            bail!("Failed to prune some of the instances");
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

        if let Some(policy) = self
            .env
            .config
            .policies()
            .build(remote.name(), project, instance)
        {
            self.try_process_instance(remote, project, instance, &policy)?;
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
        policy: &Policy,
    ) -> Result<()> {
        self.summary.add_processed_instance();

        let snapshots = find_snapshots(self.env.config, instance);
        let snapshots_to_keep = find_snapshots_to_keep(policy, &snapshots);

        for snapshot in &snapshots {
            if snapshots_to_keep.contains(&snapshot.name) {
                self.summary.add_kept_snapshot();

                writeln!(
                    self.env.stdout,
                    "  - keeping snapshot: {}",
                    snapshot.name.as_str().italic()
                )?;
            } else {
                write!(
                    self.env.stdout,
                    "  - deleting snapshot: {}",
                    snapshot.name.as_str().italic()
                )?;

                let result = self
                    .env
                    .lxd
                    .delete_snapshot(remote.name(), &project.name, &instance.name, &snapshot.name)
                    .context("Couldn't delete snapshot");

                let result = result.and_then(|_| {
                    self.env.config.hooks().on_snapshot_deleted(
                        remote.name(),
                        &project.name,
                        &instance.name,
                        &snapshot.name,
                    )
                });

                match result {
                    Ok(()) => {
                        self.summary.add_deleted_snapshot();

                        writeln!(self.env.stdout, " {}", "[ OK ]".green())?;
                    }

                    Err(err) => {
                        self.summary.add_error();

                        writeln!(self.env.stdout, " {}", "[ FAILED ]".red())?;

                        let err = format!("{:?}", err);

                        for line in err.lines() {
                            writeln!(self.env.stdout, "      {}", line)?;
                        }
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
    use crate::lxd::{utils::*, LxdFakeClient};
    use crate::{assert_lxd, assert_result, assert_stdout};

    #[test]
    fn smoke() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              all:
                excluded-instances: ['mariadb']
                keep-last: 2
        "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "elastic",
            snapshots: vec![
                snapshot("manual-1", "2000-01-01 12:00:00"),
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 14:00:00"),
                snapshot("auto-3", "2000-01-01 15:00:00"),
                snapshot("auto-4", "2000-01-01 16:00:00"),
                snapshot("manual-2", "2000-01-01 17:00:00"),
            ],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mariadb",
            snapshots: vec![
                snapshot("manual-1", "2000-01-01 12:00:00"),
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 14:00:00"),
                snapshot("auto-3", "2000-01-01 15:00:00"),
                snapshot("manual-2", "2000-01-01 16:00:00"),
            ],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "postgresql",
            snapshots: vec![
                snapshot("manual-1", "2000-01-01 12:00:00"),
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 14:00:00"),
                snapshot("manual-2", "2000-01-01 15:00:00"),
            ],
            ..Default::default()
        });

        Prune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>elastic</b>
              - keeping snapshot: <i>auto-4</i>
              - keeping snapshot: <i>auto-3</i>
              - deleting snapshot: <i>auto-2</i> <fg=32>[ OK ]</fg>
              - deleting snapshot: <i>auto-1</i> <fg=32>[ OK ]</fg>

            <b>mariadb</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>postgresql</b>
              - keeping snapshot: <i>auto-2</i>
              - keeping snapshot: <i>auto-1</i>

            <b>Summary</b>
            -------
              processed instances: 2
              deleted snapshots: 2
              kept snapshots: 4
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            local:default/elastic (Running)
            -> manual-1
            -> auto-3
            -> auto-4
            -> manual-2

            local:default/mariadb (Running)
            -> manual-1
            -> auto-1
            -> auto-2
            -> auto-3
            -> manual-2

            local:default/postgresql (Running)
            -> manual-1
            -> auto-1
            -> auto-2
            -> manual-2
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
            snapshots: vec![snapshot("auto-1", "2000-01-01 13:00:00")],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-2",
            name: "postgresql",
            snapshots: vec![snapshot("auto-1", "2000-01-01 13:00:00")],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-3",
            name: "postgresql",
            snapshots: vec![snapshot("auto-1", "2000-01-01 13:00:00")],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-4",
            name: "postgresql",
            snapshots: vec![snapshot("auto-1", "2000-01-01 13:00:00")],
            status: LxdInstanceStatus::Stopping,
            ..Default::default()
        });

        Prune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            <b>db-1:postgresql</b>
              - deleting snapshot: <i>auto-1</i> <fg=32>[ OK ]</fg>

            <b>db-2:postgresql</b>
              - deleting snapshot: <i>auto-1</i> <fg=32>[ OK ]</fg>

            <b>db-3:postgresql</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>db-4:postgresql</b>
              - <fg=33>[ EXCLUDED ]</fg>

            <b>Summary</b>
            -------
              processed instances: 2
              deleted snapshots: 2
              kept snapshots: 0
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            db-1:default/postgresql (Running)

            db-2:default/postgresql (Running)

            db-3:default/postgresql (Running)
            -> auto-1

            db-4:default/postgresql (Stopping)
            -> auto-1
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
                keep-last: 0
            "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "elastic",
            snapshots: vec![
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 13:00:00"),
            ],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mariadb",
            snapshots: vec![
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 13:00:00"),
            ],
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "postgresql",
            snapshots: vec![
                snapshot("auto-1", "2000-01-01 13:00:00"),
                snapshot("auto-2", "2000-01-01 13:00:00"),
            ],
            ..Default::default()
        });

        lxd.inject_error(LxdFakeError::OnDeleteSnapshot {
            remote: "local",
            project: "default",
            instance: "mariadb",
            snapshot: "auto-1",
        });

        let result = Prune::new(&mut Environment::test(&mut stdout, &config, &mut lxd)).run();

        assert_stdout!(
            r#"
            <b>elastic</b>
              - deleting snapshot: <i>auto-1</i> <fg=32>[ OK ]</fg>
              - deleting snapshot: <i>auto-2</i> <fg=32>[ OK ]</fg>

            <b>mariadb</b>
              - deleting snapshot: <i>auto-1</i> <fg=31>[ FAILED ]</fg>
                  Couldn't delete snapshot

                  Caused by:
                      InjectedError
              - deleting snapshot: <i>auto-2</i> <fg=32>[ OK ]</fg>

            <b>postgresql</b>
              - deleting snapshot: <i>auto-1</i> <fg=32>[ OK ]</fg>
              - deleting snapshot: <i>auto-2</i> <fg=32>[ OK ]</fg>

            <b>Summary</b>
            -------
              processed instances: 3
              deleted snapshots: 5
              kept snapshots: 0
            "#,
            stdout
        );

        assert_result!("Failed to prune some of the instances", result);

        assert_lxd!(
            r#"
            local:default/elastic (Running)

            local:default/mariadb (Running)
            -> auto-1

            local:default/postgresql (Running)
            "#,
            lxd
        );
    }
}
