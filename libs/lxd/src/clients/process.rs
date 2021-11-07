use crate::*;
use anyhow::{anyhow, Context};
use pathsearch::find_executable_in_path;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::process::Command;

/// An LXD backend that communicates with the LXD daemon through the LXC CLI
/// application
pub struct LxdProcessClient {
    /// Full path to the `lxc` executable, e.g.: `/usr/bin/lxc`.
    lxc: PathBuf,
}

impl LxdProcessClient {
    pub fn new(lxc: impl AsRef<Path>) -> Result<Self> {
        let lxc = lxc.as_ref();

        if !lxc.exists() {
            return Err(Error::Other(anyhow!(
                "Couldn't find the `lxc` executable: {}",
                lxc.display()
            )));
        }

        Ok(Self { lxc: lxc.into() })
    }

    pub fn new_from_path() -> Result<Self> {
        let lxc = find_executable_in_path("lxc")
            .ok_or_else(|| anyhow!("Couldn't find the `lxc` executable in your `PATH` - please try specifying exact location with `--lxc-path`"))
            .map_err(Error::Other)?;

        Self::new(lxc)
    }

    fn execute(&mut self, callback: impl FnOnce(&mut Command)) -> Result<String> {
        let mut command = Command::new(&self.lxc);

        callback(&mut command);

        let output = command
            .output()
            .context("Couldn't launch the `lxc` executable")
            .map_err(Error::Other)?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)
                .context("Couldn't read lxc's stdout")
                .map_err(Error::Other)?;

            Ok(stdout)
        } else {
            let stderr = String::from_utf8(output.stderr)
                .context("Couldn't read lxc's stderr")
                .map_err(Error::Other)?
                .trim()
                .to_string();

            Err(Error::Other(anyhow!(
                "lxc returned a non-zero status code and said: {}",
                stderr,
            )))
        }
    }

    fn parse<T: DeserializeOwned>(out: String) -> Result<T> {
        serde_json::from_str(&out)
            .context("Couldn't parse lxc's stdout")
            .map_err(Error::Other)
    }
}

impl LxdClient for LxdProcessClient {
    fn list_projects(&mut self) -> Result<Vec<LxdProject>> {
        let out = self.execute(|command| {
            command.arg("project").arg("list").arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdInstance>> {
        let out = self.execute(|command| {
            command
                .arg("list")
                .arg(format!("--project={}", project))
                .arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn create_snapshot(
        &mut self,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> Result<()> {
        self.execute(|command| {
            command
                .arg("snapshot")
                .arg(instance.as_str())
                .arg(snapshot.as_str())
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> Result<()> {
        self.execute(|command| {
            command
                .arg("delete")
                .arg(format!("{}/{}", instance, snapshot))
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }
}
