use crate::prelude::*;
use lib_lxd::{LxdInstanceName, LxdProjectName, LxdSnapshotName};
use serde::Deserialize;
use std::process::Command;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Hooks {
    pub on_backup_started: Option<String>,
    pub on_snapshot_created: Option<String>,
    pub on_backup_completed: Option<String>,
    pub on_prune_started: Option<String>,
    pub on_snapshot_deleted: Option<String>,
    pub on_prune_completed: Option<String>,
}

impl Hooks {
    pub fn on_backup_started(&self) -> Result<()> {
        Self::exec(self.on_backup_started.as_deref(), &[])
            .context("Couldn't execute the `on-backup-started` hook")
    }

    pub fn on_snapshot_created(
        &self,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        Self::exec(
            self.on_snapshot_created.as_deref(),
            &[
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
                ("snapshotName", snapshot_name.as_str()),
            ],
        )
        .context("Couldn't execute the `on-snapshot-created` hook")
    }

    pub fn on_backup_completed(&self) -> Result<()> {
        Self::exec(self.on_backup_completed.as_deref(), &[])
            .context("Couldn't execute the `on-backup-completed` hook")
    }

    pub fn on_prune_started(&self) -> Result<()> {
        Self::exec(self.on_prune_started.as_deref(), &[])
            .context("Couldn't execute the `on-prune-started` hook")
    }

    pub fn on_snapshot_deleted(
        &self,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        Self::exec(
            self.on_snapshot_deleted.as_deref(),
            &[
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
                ("snapshotName", snapshot_name.as_str()),
            ],
        )
        .context("Couldn't execute the `on-snapshot-deleted` hook")
    }

    pub fn on_prune_completed(&self) -> Result<()> {
        Self::exec(self.on_prune_completed.as_deref(), &[])
            .context("Couldn't execute the `on-prune-completed` hook")
    }

    fn exec(command: Option<&str>, variables: &[(&str, &str)]) -> Result<()> {
        let command = if let Some(command) = command {
            command
        } else {
            return Ok(());
        };

        let command = Self::render(command, variables);

        let result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .spawn()
            .context("Couldn't start hook")?
            .wait()
            .context("Couldn't await hook")?;

        if !result.success() {
            bail!("Hook returned a non-zero exit code");
        }

        Ok(())
    }

    fn render(command: &str, variables: &[(&str, &str)]) -> String {
        variables
            .iter()
            .fold(command.to_string(), |template, (var_name, var_value)| {
                template.replace(&format!("{{{{{}}}}}", var_name), var_value)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render {
        use super::*;

        #[test]
        fn test() {
            let actual = Hooks::render(
                "one {{one}} {{one}} two {{two}}",
                &[("one", "1"), ("two", "2")],
            );

            assert_eq!("one 1 1 two 2", actual);
        }
    }
}
