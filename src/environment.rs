use crate::prelude::*;
use std::fmt::Write as _;
use std::io::Write;
use std::process::Command;

pub struct Environment<'a> {
    pub time: fn() -> DateTime<Utc>,
    pub stdout: &'a mut dyn Write,
    pub config: &'a Config,
    pub lxd: &'a mut dyn LxdClient,
    pub dry_run: bool,
}

impl<'a> Environment<'a> {
    #[cfg(test)]
    pub fn test(stdout: &'a mut dyn Write, config: &'a Config, lxd: &'a mut dyn LxdClient) -> Self {
        use chrono::TimeZone;

        Self {
            time: || Utc.timestamp_opt(0, 0).unwrap(),
            stdout,
            config,
            lxd,
            dry_run: false,
        }
    }

    pub fn time(&self) -> DateTime<Utc> {
        (self.time)()
    }

    pub fn hooks<'b>(&'b mut self) -> EnvironmentHooks<'b, 'a> {
        EnvironmentHooks { env: self }
    }
}

pub struct EnvironmentHooks<'b, 'a> {
    env: &'b mut Environment<'a>,
}

impl EnvironmentHooks<'_, '_> {
    pub fn on_backup_started(&mut self) -> Result<()> {
        let cmd = self.env.config.hooks().on_backup_started();

        self.run("on-backup-started", cmd)
    }

    pub fn on_snapshot_created(
        &mut self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        let cmd = self.env.config.hooks().on_snapshot_created(
            remote_name,
            project_name,
            instance_name,
            snapshot_name,
        );

        self.run("on-snapshot-created", cmd)
    }

    pub fn on_instance_backed_up(
        &mut self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
    ) -> Result<()> {
        let cmd =
            self.env
                .config
                .hooks()
                .on_instance_backed_up(remote_name, project_name, instance_name);

        self.run("on-instance-backed-up", cmd)
    }

    pub fn on_backup_completed(&mut self) -> Result<()> {
        let cmd = self.env.config.hooks().on_backup_completed();

        self.run("on-backup-completed", cmd)
    }

    pub fn on_prune_started(&mut self) -> Result<()> {
        let cmd = self.env.config.hooks().on_prune_started();

        self.run("on-prune-started", cmd)
    }

    pub fn on_instance_pruned(
        &mut self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
    ) -> Result<()> {
        let cmd =
            self.env
                .config
                .hooks()
                .on_instance_pruned(remote_name, project_name, instance_name);

        self.run("on-instance-pruned", cmd)
    }

    pub fn on_snapshot_deleted(
        &mut self,
        remote_name: &LxdRemoteName,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        let cmd = self.env.config.hooks().on_snapshot_deleted(
            remote_name,
            project_name,
            instance_name,
            snapshot_name,
        );

        self.run("on-snapshot-deleted", cmd)
    }

    pub fn on_prune_completed(&mut self) -> Result<()> {
        let cmd = self.env.config.hooks().on_prune_completed();

        self.run("on-prune-completed", cmd)
    }

    fn run(&mut self, hook: &str, cmd: Option<String>) -> Result<()> {
        self.try_run(cmd)
            .with_context(|| format!("Couldn't execute the `{}` hook", hook))
    }

    fn try_run(&mut self, cmd: Option<String>) -> Result<()> {
        let cmd = if let Some(cmd) = cmd {
            cmd
        } else {
            return Ok(());
        };

        if self.env.dry_run {
            return Ok(());
        }

        let result = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .context("Couldn't launch hook")?;

        if !result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stdout = stdout.trim();

            let stderr = String::from_utf8_lossy(&result.stderr);
            let stderr = stderr.trim();

            let mut msg = String::from("Hook returned a non-zero exit code.");

            if !stdout.is_empty() {
                msg.push_str("\n\nHook's stdout:");

                for line in stdout.lines() {
                    _ = write!(&mut msg, "\n    {}", line);
                }
            }

            if !stderr.is_empty() {
                msg.push_str("\n\nHook's stderr:");

                for line in stderr.lines() {
                    _ = write!(&mut msg, "\n    {}", line);
                }
            }

            bail!(msg);
        }

        Ok(())
    }
}
