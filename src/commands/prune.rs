mod find_snapshots;
mod find_snapshots_to_keep;
mod summary;

use self::{find_snapshots::*, find_snapshots_to_keep::*, summary::*};
use crate::prelude::*;

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
        self.env.config.hooks().on_prune_started()?;

        let cmd_result = self.try_run();
        let hook_result = self.env.config.hooks().on_prune_completed();

        cmd_result.and(hook_result)
    }

    fn try_run(&mut self) -> Result<()> {
        writeln!(self.env.stdout, "Pruning instances:")?;

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

        if let Some(policy) = self
            .env
            .config
            .policies()
            .build(remote.name(), project, instance)
        {
            match self.try_process_intance(remote, project, instance, &policy) {
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
        remote: &Remote,
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

                self.env.lxd.delete_snapshot(
                    remote.name(),
                    &project.name,
                    &instance.name,
                    &snapshot.name,
                )?;

                self.env
                    .config
                    .hooks()
                    .on_snapshot_deleted(
                        remote.name(),
                        &project.name,
                        &instance.name,
                        &snapshot.name,
                    )
                    .context("Couldn't execute the `on-snapshot-deleted` hook")?;
            }
        }

        Ok((deleted_snapshots, kept_snapshots))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::{utils::*, LxdFakeClient};
    use crate::{assert_lxd, assert_out};

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

        assert_out!(
            r#"
            Pruning instances:
            
            - default/elastic
            -> keeping snapshot: auto-4
            -> keeping snapshot: auto-3
            -> deleting snapshot: auto-2
            -> deleting snapshot: auto-1
            -> [ OK ]
            
            - default/mariadb
            -> [ EXCLUDED ]
            
            - default/postgresql
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
        // TODO
    }
}
