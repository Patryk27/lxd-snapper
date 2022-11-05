use crate::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Hooks {
    on_backup_started: Option<String>,
    on_snapshot_created: Option<String>,
    on_instance_backed_up: Option<String>,
    on_backup_completed: Option<String>,

    on_prune_started: Option<String>,
    on_snapshot_deleted: Option<String>,
    on_instance_pruned: Option<String>,
    on_prune_completed: Option<String>,
}

impl Hooks {
    pub fn on_backup_started(&self) -> Option<String> {
        Self::render(self.on_backup_started.as_deref(), &[])
    }

    pub fn on_snapshot_created(
        &self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Option<String> {
        Self::render(
            self.on_snapshot_created.as_deref(),
            &[
                ("remoteName", remote_name.as_str()),
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
                ("snapshotName", snapshot_name.as_str()),
            ],
        )
    }

    pub fn on_instance_backed_up(
        &self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
    ) -> Option<String> {
        Self::render(
            self.on_instance_backed_up.as_deref(),
            &[
                ("remoteName", remote_name.as_str()),
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
            ],
        )
    }

    pub fn on_backup_completed(&self) -> Option<String> {
        Self::render(self.on_backup_completed.as_deref(), &[])
    }

    pub fn on_prune_started(&self) -> Option<String> {
        Self::render(self.on_prune_started.as_deref(), &[])
    }

    pub fn on_snapshot_deleted(
        &self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Option<String> {
        Self::render(
            self.on_snapshot_deleted.as_deref(),
            &[
                ("remoteName", remote_name.as_str()),
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
                ("snapshotName", snapshot_name.as_str()),
            ],
        )
    }

    pub fn on_instance_pruned(
        &self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
    ) -> Option<String> {
        Self::render(
            self.on_instance_pruned.as_deref(),
            &[
                ("remoteName", remote_name.as_str()),
                ("projectName", project_name.as_str()),
                ("instanceName", instance_name.as_str()),
            ],
        )
    }

    pub fn on_prune_completed(&self) -> Option<String> {
        Self::render(self.on_prune_completed.as_deref(), &[])
    }

    fn render(cmd: Option<&str>, variables: &[(&str, &str)]) -> Option<String> {
        cmd.map(|cmd| {
            variables
                .iter()
                .fold(cmd.to_string(), |template, (var_name, var_value)| {
                    template
                        .replace(&format!("{{{{{}}}}}", var_name), var_value)
                        .replace(&format!("{{{{ {} }}}}", var_name), var_value)
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let hooks = Hooks {
            on_backup_started: Some("on-backup-started".into()),
            on_snapshot_created: Some(
                "on-snapshot-created(\
                   {{ remoteName }}, \
                   {{ projectName }}, \
                   {{ instanceName }}, \
                   {{ snapshotName }}\
                 )"
                .into(),
            ),
            on_instance_backed_up: Some(
                "on-instance-backed-up(\
                   {{ remoteName }}, \
                   {{ projectName }}, \
                   {{ instanceName }}\
                 )"
                .into(),
            ),
            on_backup_completed: Some("on-backup-completed".into()),
            on_prune_started: Some("on-prune-started".into()),
            on_snapshot_deleted: Some(
                "on-snapshot-deleted(\
                   {{ remoteName }}, \
                   {{ projectName }}, \
                   {{ instanceName }}, \
                   {{ snapshotName }}\
                 )"
                .into(),
            ),
            on_instance_pruned: Some(
                "on-instance-pruned(\
                   {{ remoteName }}, \
                   {{ projectName }}, \
                   {{ instanceName }}\
                 )"
                .into(),
            ),
            on_prune_completed: Some("on-prune-completed".into()),
        };

        let remote_name = LxdRemoteName::new("remote");
        let project_name = LxdProjectName::new("project");
        let instance_name = LxdInstanceName::new("instance");
        let snapshot_name = LxdSnapshotName::new("snapshot");

        assert_eq!(
            Some("on-backup-started"),
            hooks.on_backup_started().as_deref()
        );

        assert_eq!(
            Some("on-snapshot-created(remote, project, instance, snapshot)"),
            hooks
                .on_snapshot_created(&remote_name, &project_name, &instance_name, &snapshot_name)
                .as_deref()
        );

        assert_eq!(
            Some("on-instance-backed-up(remote, project, instance)"),
            hooks
                .on_instance_backed_up(&remote_name, &project_name, &instance_name)
                .as_deref()
        );

        assert_eq!(
            Some("on-backup-completed"),
            hooks.on_backup_completed().as_deref()
        );

        assert_eq!(
            Some("on-prune-started"),
            hooks.on_prune_started().as_deref()
        );

        assert_eq!(
            Some("on-snapshot-deleted(remote, project, instance, snapshot)"),
            hooks
                .on_snapshot_deleted(&remote_name, &project_name, &instance_name, &snapshot_name)
                .as_deref()
        );

        assert_eq!(
            Some("on-instance-pruned(remote, project, instance)"),
            hooks
                .on_instance_pruned(&remote_name, &project_name, &instance_name)
                .as_deref()
        );

        assert_eq!(
            Some("on-prune-completed"),
            hooks.on_prune_completed().as_deref()
        );
    }

    #[test]
    fn render() {
        assert_eq!(None, Hooks::render(None, &[]));

        // ---

        let actual = Hooks::render(
            Some("one {{one}} {{ one }} two {{two}}"),
            &[("one", "1"), ("two", "2")],
        );

        assert_eq!(Some("one 1 1 two 2"), actual.as_deref());
    }
}
