use crate::config::{Config, Policy};
use anyhow::Result;
use colored::Colorize;
use lib_lxd::*;
use prettytable::{cell, row, Table};
use std::io::Write;

pub fn query_instances(
    stdout: &mut dyn Write,
    config: &Config,
    lxd: &mut dyn LxdClient,
) -> Result<()> {
    let mut table = Table::new();

    table.set_titles(row!["Project", "Instance", "Policies"]);

    for project in lxd.list_projects()? {
        for instance in lxd.list(&project.name)? {
            let policies = format_policies(config.policies(&project, &instance));
            table.add_row(row![project.name, instance.name, policies]);
        }
    }

    write!(stdout, "{}", table)?;

    Ok(())
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
    use indoc::indoc;
    use lib_lxd::test_utils::*;
    use std::env::set_var;

    const POLICY: &str = indoc!(
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
        let config = Config::from_code(POLICY);

        let mut lxd = LxdFakeClient::new(vec![
            LxdInstance {
                name: instance_name("ruby"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            //
            LxdInstance {
                name: instance_name("rust"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            //
            LxdInstance {
                name: instance_name("mysql"),
                status: LxdInstanceStatus::Running,
                snapshots: Default::default(),
            },
            //
            LxdInstance {
                name: instance_name("redis"),
                status: LxdInstanceStatus::Stopped,
                snapshots: Default::default(),
            },
            //
            LxdInstance {
                name: instance_name("outlander"),
                status: LxdInstanceStatus::Stopped,
                snapshots: Default::default(),
            },
        ]);

        set_var("NO_COLOR", "1");
        query_instances(&mut stdout, &config, &mut lxd).unwrap();

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
