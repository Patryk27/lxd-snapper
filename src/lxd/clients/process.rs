use crate::lxd::*;
use anyhow::{anyhow, Context};
use pathsearch::find_executable_in_path;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct LxdProcessClient {
    lxc: PathBuf,
}

impl LxdProcessClient {
    pub fn new(lxc: impl AsRef<Path>) -> LxdResult<Self> {
        let lxc = lxc.as_ref();

        if !lxc.exists() {
            return Err(LxdError::Other(anyhow!(
                "Couldn't find the `lxc` executable: {}",
                lxc.display()
            )));
        }

        Ok(Self { lxc: lxc.into() })
    }

    pub fn find() -> LxdResult<Self> {
        let lxc = find_executable_in_path("lxc")
            .ok_or_else(|| anyhow!("Couldn't find the `lxc` executable in your `PATH` - please try specifying exact location with `--lxc-path`"))?;

        Self::new(lxc)
    }

    fn execute(&mut self, callback: impl FnOnce(&mut Command)) -> LxdResult<String> {
        let mut command = Command::new(&self.lxc);

        callback(&mut command);

        let output = command
            .output()
            .context("Couldn't launch the `lxc` executable")?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).context("Couldn't read lxc's stdout")?;

            Ok(stdout)
        } else {
            let stderr = String::from_utf8(output.stderr)
                .context("Couldn't read lxc's stderr")?
                .trim()
                .to_string();

            Err(LxdError::Other(anyhow!(
                "lxc returned a non-zero status code and said: {}",
                stderr,
            )))
        }
    }

    fn parse<T>(out: String) -> LxdResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&out)
            .context("Couldn't parse lxc's stdout")
            .map_err(LxdError::Other)
    }
}

impl LxdClient for LxdProcessClient {
    fn projects(&mut self, remote: &LxdRemoteName) -> LxdResult<Vec<LxdProject>> {
        let out = self.execute(|command| {
            command
                .arg("project")
                .arg("list")
                .arg(format!("{}:", remote))
                .arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn instances(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
    ) -> LxdResult<Vec<LxdInstance>> {
        let out = self.execute(|command| {
            command
                .arg("list")
                .arg(format!("{}:", remote))
                .arg(format!("--project={}", project))
                .arg("--format=json");
        })?;

        Self::parse(out)
    }

    fn create_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()> {
        self.execute(|command| {
            command
                .arg("snapshot")
                .arg(instance.on(remote))
                .arg(snapshot.as_str())
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()> {
        self.execute(|command| {
            command
                .arg("delete")
                .arg(format!("{}/{}", instance.on(remote), snapshot))
                .arg(format!("--project={}", project));
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Covered via Nix-based tests
}
