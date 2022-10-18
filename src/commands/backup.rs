mod summary;

use self::summary::*;
use crate::prelude::*;

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
        self.env.config.hooks().on_backup_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks().on_backup_completed();

        cmd_result.and(hook_result)
    }

    fn try_run(&mut self) -> Result<()> {
        writeln!(self.env.stdout, "Backing-up instances:")?;

        if self.env.config.remotes().has_any_non_local_remotes() {
            for remote in self.env.config.remotes().iter() {
                self.process_remote(true, remote)
                    .with_context(|| format!("Couldn't process remote: {}", remote.name()))?;
            }
        } else {
            self.process_remote(false, &Remote::local())?;
        }

        self.summary.print(self.env.stdout)?;

        Ok(())
    }

    fn process_remote(&mut self, print_remote_name: bool, remote: &Remote) -> Result<()> {
        let projects = self
            .env
            .lxd
            .projects(remote.name())
            .context("Couldn't list projects")?;

        for project in projects {
            self.process_project(print_remote_name, remote, &project)
                .with_context(|| format!("Couldn't process project: {}", project.name))?;
        }

        Ok(())
    }

    fn process_project(
        &mut self,
        print_remote_name: bool,
        remote: &Remote,
        project: &LxdProject,
    ) -> Result<()> {
        let instances = self
            .env
            .lxd
            .instances(remote.name(), &project.name)
            .context("Couldn't list instances")?;

        for instance in instances {
            self.process_instance(print_remote_name, remote, project, &instance)
                .with_context(|| format!("Couldn't process instance: {}", instance.name))?;
        }

        Ok(())
    }

    fn process_instance(
        &mut self,
        print_remote_name: bool,
        remote: &Remote,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<()> {
        self.summary.processed_instances += 1;

        writeln!(self.env.stdout)?;

        if print_remote_name {
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

        if self
            .env
            .config
            .policies()
            .matches(remote.name(), project, instance)
        {
            match self.try_process_instance(remote, project, instance) {
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
        remote: &Remote,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Result<LxdSnapshotName> {
        let snapshot_name = self.env.config.snapshot_name(self.env.time());

        writeln!(self.env.stdout, "-> creating snapshot: {}", snapshot_name)?;

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
    use crate::{assert_lxd, assert_out};

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
            name: "mysql",
            snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            ..Default::default()
        });

        Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_out!(
            r#"
            Backing-up instances:

            - default/elastic
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]

            - default/mariadb
            -> [ EXCLUDED ]

            - default/mongodb
            -> [ EXCLUDED ]

            - default/mysql
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]

            Summary
            - processed instances: 4
            - created snapshots: 2
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

            local:default/mysql (Running)
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
            name: "mysql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-2",
            name: "mysql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-3",
            name: "mysql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            remote: "db-4",
            name: "mysql",
            status: LxdInstanceStatus::Stopping,
            ..Default::default()
        });

        Backup::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_out!(
            r#"
            Backing-up instances:

            - db-1:default/mysql
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]

            - db-2:default/mysql
            -> creating snapshot: auto-19700101-000000
            -> [ OK ]

            - db-3:default/mysql
            -> [ EXCLUDED ]

            - db-4:default/mysql
            -> [ EXCLUDED ]

            Summary
            - processed instances: 4
            - created snapshots: 2
            "#,
            stdout
        );

        assert_lxd!(
            r#"
            db-1:default/mysql (Running)
            -> auto-19700101-000000

            db-2:default/mysql (Running)
            -> auto-19700101-000000

            db-3:default/mysql (Running)

            db-4:default/mysql (Stopping)
            "#,
            lxd
        );
    }
}
