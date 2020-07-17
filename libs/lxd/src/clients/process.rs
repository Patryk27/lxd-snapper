// TODO proper error handling

use crate::*;
use anyhow::{anyhow, Context};
use pathsearch::find_executable_in_path;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Communicates with LXD daemon through the LXC CLI application
pub struct LxdProcessClient {
    /// Full path to the `lxc` executable
    lxc_path: PathBuf,
}

impl LxdProcessClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            lxc_path: find_executable_in_path("lxc").ok_or(Error::CouldntFindLxc)?,
        })
    }

    pub fn new_ex(lxc_path: impl AsRef<Path>) -> Self {
        Self {
            lxc_path: lxc_path.as_ref().into(),
        }
    }

    fn execute(&mut self, callback: impl FnOnce(&mut Command)) -> Result<String> {
        let mut command = Command::new(&self.lxc_path);

        callback(&mut command);

        let output = command
            .output()
            .map_err(|err| Error::CouldntLaunchLxc(err.into()))?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)
                .context("Couldn't read lxc's stdout")
                .map_err(Error::CouldntLaunchLxc)?;

            Ok(stdout)
        } else {
            let stderr = String::from_utf8(output.stderr)
                .context("Couldn't read lxc's stderr")
                .map_err(Error::CouldntLaunchLxc)?
                .trim()
                .to_string();

            Err(Error::CouldntLaunchLxc(anyhow!(
                "lxc returned a non-zero status code and said: {}",
                stderr
            )))
        }
    }

    fn parse<T: DeserializeOwned>(out: String) -> Result<T> {
        serde_json::from_str(&out)
            .context("Couldn't parse lxc's stdout")
            .map_err(Error::CouldntLaunchLxc)
    }
}

impl LxdClient for LxdProcessClient {
    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdContainer>> {
        let out = self.execute(|command| {
            command
                .arg("list")
                .arg(format!("--project={}", project))
                .arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn list_projects(&mut self) -> Result<Vec<LxdProject>> {
        let out = self.execute(|command| {
            command.arg("project").arg("list").arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn create_snapshot(
        &mut self,
        project: &LxdProjectName,
        container: &LxdContainerName,
        snapshot: &LxdSnapshotName,
    ) -> Result {
        self.execute(|command| {
            command
                .arg("snapshot")
                .arg(container.as_str())
                .arg(snapshot.as_str())
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        project: &LxdProjectName,
        container: &LxdContainerName,
        snapshot: &LxdSnapshotName,
    ) -> Result {
        self.execute(|command| {
            command
                .arg("delete")
                .arg(format!("{}/{}", container, snapshot))
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }
}
