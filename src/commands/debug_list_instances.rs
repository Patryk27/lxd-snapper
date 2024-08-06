use crate::prelude::*;
use itertools::Itertools;
use prettytable::{row, Table};

pub struct DebugListInstances<'a, 'b> {
    env: &'a mut Environment<'b>,
}

impl<'a, 'b> DebugListInstances<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self { env }
    }

    pub fn run(self) -> Result<()> {
        let mut table = Table::new();
        let has_remotes = self.env.config.remotes().has_any_non_local_remotes();

        if has_remotes {
            table.set_titles(row!["Remote", "Project", "Instance", "Policies"]);
        } else {
            table.set_titles(row!["Project", "Instance", "Policies"]);
        }

        for remote in self.env.config.remotes().iter() {
            for project in self.env.lxd.projects(remote)? {
                for instance in self.env.lxd.instances(remote, &project.name)? {
                    let policies = self
                        .env
                        .config
                        .policies()
                        .find(remote, &project, &instance)
                        .collect();

                    let policies = format_policies(policies);

                    if has_remotes {
                        table.add_row(row![remote, project.name, instance.name, policies]);
                    } else {
                        table.add_row(row![project.name, instance.name, policies]);
                    }
                }
            }
        }

        write!(self.env.stdout, "{}", table)?;

        Ok(())
    }
}

fn format_policies(policies: Vec<(&str, &Policy)>) -> String {
    if policies.is_empty() {
        "NONE".yellow().to_string()
    } else {
        policies.iter().map(|(name, _)| *name).join(" + ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_stdout;
    use crate::lxd::{LxdFakeClient, LxdInstanceStatus};

    #[test]
    fn smoke() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
                policies:
                  running:
                    included-statuses: ['Running']

                  databases:
                    included-instances: ['mysql', 'redis']
                "#
        ));

        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "ruby",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "rust",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "mysql",
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "redis",
            status: LxdInstanceStatus::Stopped,
            ..Default::default()
        });

        lxd.add(LxdFakeInstance {
            name: "outlander",
            status: LxdInstanceStatus::Stopped,
            ..Default::default()
        });

        DebugListInstances::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            +---------+-----------+---------------------+
            | Project | Instance  | Policies            |
            +=========+===========+=====================+
            | default | mysql     | running + databases |
            +---------+-----------+---------------------+
            | default | outlander | <fg=33>NONE</fg>              |
            +---------+-----------+---------------------+
            | default | redis     | databases           |
            +---------+-----------+---------------------+
            | default | ruby      | running             |
            +---------+-----------+---------------------+
            | default | rust      | running             |
            +---------+-----------+---------------------+
            "#,
            stdout
        );
    }

    #[test]
    fn smoke_with_remotes() {
        let mut stdout = Vec::new();

        let config = Config::parse(indoc!(
            r#"
            policies:
              important-servers:
                included-remotes: ['server-a', 'server-b']

            remotes:
              - server-a
              - server-b
              - server-c
            "#
        ));

        let mut lxd = LxdFakeClient::default();

        for remote in ["local", "server-a", "server-b", "server-c"] {
            lxd.add(LxdFakeInstance {
                remote,
                name: "php",
                ..Default::default()
            });
        }

        DebugListInstances::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_stdout!(
            r#"
            +----------+---------+----------+-------------------+
            | Remote   | Project | Instance | Policies          |
            +==========+=========+==========+===================+
            | server-a | default | php      | important-servers |
            +----------+---------+----------+-------------------+
            | server-b | default | php      | important-servers |
            +----------+---------+----------+-------------------+
            | server-c | default | php      | <fg=33>NONE</fg>            |
            +----------+---------+----------+-------------------+
            "#,
            stdout
        );
    }
}
