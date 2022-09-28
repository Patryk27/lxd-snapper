use crate::prelude::*;
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

        table.set_titles(row!["Project", "Instance", "Policies"]);

        for project in self.env.lxd.list_projects()? {
            for instance in self.env.lxd.list(&project.name)? {
                let policies = self.env.config.policies.find(&project, &instance).collect();
                let policies = format_policies(policies);

                table.add_row(row![project.name, instance.name, policies]);
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
        let names: Vec<_> = policies.iter().map(|(name, _)| *name).collect();
        names.join(" + ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use lib_lxd::{test_utils::*, LxdFakeClient, LxdInstance, LxdInstanceStatus};
    use std::env::set_var;

    const CONFIG: &str = indoc!(
        r#"
        policies:
          _running:
            included-statuses: ['Running']
            
          databases:
            included-instances: ['mysql', 'redis']
        "#
    );

    #[test]
    fn test() {
        let mut stdout = Vec::new();
        let config = Config::from_code(CONFIG);

        let mut lxd = LxdFakeClient::new(vec![
            LxdInstance {
                name: instance_name("ruby"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("rust"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("mysql"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("redis"),
                status: LxdInstanceStatus::Stopped,
                snapshots: Default::default(),
            },
            LxdInstance {
                name: instance_name("outlander"),
                status: LxdInstanceStatus::Stopped,
                snapshots: Default::default(),
            },
        ]);

        set_var("NO_COLOR", "1");

        DebugListInstances::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
            .run()
            .unwrap();

        assert_out!(
            r#"
            +---------+-----------+----------------------+
            | Project | Instance  | Policies             |
            +=========+===========+======================+
            | default | mysql     | _running + databases |
            +---------+-----------+----------------------+
            | default | outlander | NONE                 |
            +---------+-----------+----------------------+
            | default | redis     | databases            |
            +---------+-----------+----------------------+
            | default | ruby      | _running             |
            +---------+-----------+----------------------+
            | default | rust      | _running             |
            +---------+-----------+----------------------+
            "#,
            stdout
        );
    }
}
