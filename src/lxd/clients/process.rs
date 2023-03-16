use crate::lxd::*;
use anyhow::{anyhow, Context};
use pathsearch::find_executable_in_path;
use serde::de::DeserializeOwned;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub struct LxdProcessClient {
    lxc: PathBuf,
    timeout: Duration,
}

impl LxdProcessClient {
    pub fn new(lxc: impl AsRef<Path>, timeout: Duration) -> LxdResult<Self> {
        let lxc = lxc.as_ref();

        if !lxc.exists() {
            return Err(LxdError::Other(anyhow!(
                "Couldn't find the `lxc` executable: {}",
                lxc.display()
            )));
        }

        Ok(Self {
            lxc: lxc.into(),
            timeout,
        })
    }

    pub fn find(timeout: Duration) -> LxdResult<Self> {
        let lxc = find_executable_in_path("lxc")
            .ok_or_else(|| anyhow!("Couldn't find the `lxc` executable in your `PATH` - please try specifying exact location with `--lxc-path`"))?;

        Self::new(lxc, timeout)
    }

    fn execute(&mut self, callback: impl FnOnce(&mut Command)) -> LxdResult<String> {
        let mut command = Command::new(&self.lxc);

        callback(&mut command);

        let mut command = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Couldn't launch the `lxc` executable")?;

        let mut stdout_ex = command.stdout.take().unwrap();
        let mut stderr_ex = command.stderr.take().unwrap();

        let status = command
            .wait_timeout(self.timeout)
            .context("Couldn't await the `lxc` executable")?
            .context("Operation timed out - lxc took too long to answer")?;

        if status.success() {
            let mut stdout = String::default();

            stdout_ex
                .read_to_string(&mut stdout)
                .context("Couldn't read lxc's stdout")?;

            Ok(stdout)
        } else {
            let mut stderr = String::default();

            stderr_ex
                .read_to_string(&mut stderr)
                .context("Couldn't read lxc's stderr")?;

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
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(file!())
            .parent()
            .unwrap()
            .join("process")
            .join("tests")
            .join(name)
    }

    #[test]
    fn execute_timeout_ok() {
        let actual = LxdProcessClient::new(fixture("lxc-timeout.sh"), Duration::from_secs(10))
            .unwrap()
            .execute(|_| ())
            .unwrap();

        assert_eq!("done!", actual.trim());
    }

    #[test]
    fn execute_timeout_err() {
        let actual = LxdProcessClient::new(fixture("lxc-timeout.sh"), Duration::from_millis(500))
            .unwrap()
            .execute(|_| ())
            .unwrap_err()
            .to_string();

        assert_eq!(
            "Operation timed out - lxc took too long to answer",
            actual.trim()
        );
    }

    #[test]
    fn execute_non_zero_exit_code() {
        let actual =
            LxdProcessClient::new(fixture("lxc-non-zero-exit-code.sh"), Duration::from_secs(1))
                .unwrap()
                .execute(|_| ())
                .unwrap_err()
                .to_string();

        assert_eq!(
            "lxc returned a non-zero status code and said: oii stderr",
            actual.trim()
        );
    }
}
