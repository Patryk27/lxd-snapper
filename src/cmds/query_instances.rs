use crate::config::{Config, Policy};
use anyhow::Result;
use colored::Colorize;
use lib_lxd::LxdClient;
use std::io::Write;

crate fn query_instances(
    stdout: &mut dyn Write,
    config: &Config,
    lxd: &mut dyn LxdClient,
) -> Result<()> {
    writeln!(stdout, "Found instances:")?;

    for project in lxd.list_projects()? {
        for instance in lxd.list(&project.name)? {
            let policies = format_policies(config.policies(&project, &instance));

            writeln!(
                stdout,
                "- {} with policy: {}",
                format!("{}/{}", project.name, instance.name).green(),
                policies,
            )?;
        }
    }

    Ok(())
}

fn format_policies(policies: Vec<(&str, &Policy)>) -> String {
    if policies.is_empty() {
        "NONE".yellow().to_string()
    } else {
        let names: Vec<_> = policies
            .iter()
            .map(|(name, _)| name.green().to_string())
            .collect();

        names.join(" + ")
    }
}
