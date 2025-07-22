use crate::lxd::*;
use anyhow::{anyhow, Context};
use pathsearch::find_executable_in_path;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct LxdProcessClient {
    path: PathBuf,
    flavor: LxcFlavor,
    timeout: Duration,
}

impl LxdProcessClient {
    pub fn new(path: impl AsRef<Path>, flavor: LxcFlavor, timeout: Duration) -> LxdResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(LxdError::Other(anyhow!(
                "Couldn't find the client executable: {}",
                path.display()
            )));
        }

        Ok(Self {
            path: path.into(),
            flavor,
            timeout,
        })
    }

    pub fn auto(timeout: Duration) -> LxdResult<Self> {
        if let Some(path) = find_executable_in_path("lxc") {
            return Self::new(path, LxcFlavor::Lxc, timeout);
        }

        if let Some(path) = find_executable_in_path("incus") {
            return Self::new(path, LxcFlavor::Incus, timeout);
        }

        Err(LxdError::Other(anyhow!(
            "Couldn't find the `lxc` or `incus` executable in `PATH` - try \
             providing an exact location via `--lxc-path` or `--incus-path`"
        )))
    }

    fn execute(&mut self, callback: impl FnOnce(&mut Command)) -> LxdResult<String> {
        let mut command = Command::new(&self.path);

        callback(&mut command);

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = (|| {
                let output = command
                    .output()
                    .context("Couldn't launch the client's executable")?;

                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)
                        .context("Couldn't read client's stdout")?;

                    Ok(stdout)
                } else {
                    let stderr = String::from_utf8(output.stderr)
                        .context("Couldn't read client's stderr")?
                        .trim()
                        .to_string();

                    Err(LxdError::Other(anyhow!(
                        "Client returned a non-zero status code and said: {}",
                        stderr,
                    )))
                }
            })();

            _ = tx.send(result);
        });

        rx.recv_timeout(self.timeout)
            .map_err(|_| anyhow!("Operation timed out - client took too long to answer"))?
    }

    fn parse<T>(out: String) -> LxdResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&out)
            .context("Couldn't parse client's stdout")
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
        let flavor = self.flavor;

        self.execute(|command| match flavor {
            LxcFlavor::Lxc => {
                command
                    .arg("snapshot")
                    .arg(instance.on(remote))
                    .arg(snapshot.as_str())
                    .arg(format!("--project={}", project));
            }

            LxcFlavor::Incus => {
                command
                    .arg("snapshot")
                    .arg("create")
                    .arg(instance.on(remote))
                    .arg(snapshot.as_str())
                    .arg(format!("--project={}", project));
            }
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
        let flavor = self.flavor;

        self.execute(|command| match flavor {
            LxcFlavor::Lxc => {
                command
                    .arg("delete")
                    .arg(format!("{}/{}", instance.on(remote), snapshot))
                    .arg(format!("--project={}", project));
            }

            LxcFlavor::Incus => {
                command
                    .arg("snapshot")
                    .arg("delete")
                    .arg(instance.on(remote))
                    .arg(snapshot.as_str())
                    .arg(format!("--project={}", project));
            }
        })?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LxcFlavor {
    Lxc,
    Incus,
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
        let actual = LxdProcessClient::new(
            fixture("lxc-timeout.sh"),
            LxcFlavor::Lxc,
            Duration::from_secs(10),
        )
        .unwrap()
        .execute(|_| ())
        .unwrap();

        assert_eq!("done!", actual.trim());
    }

    #[test]
    fn execute_timeout_err() {
        let actual = LxdProcessClient::new(
            fixture("lxc-timeout.sh"),
            LxcFlavor::Lxc,
            Duration::from_millis(500),
        )
        .unwrap()
        .execute(|_| ())
        .unwrap_err()
        .to_string();

        assert_eq!(
            "Operation timed out - client took too long to answer",
            actual.trim()
        );
    }

    #[test]
    fn execute_non_zero_exit_code() {
        let actual = LxdProcessClient::new(
            fixture("lxc-non-zero-exit-code.sh"),
            LxcFlavor::Lxc,
            Duration::from_secs(1),
        )
        .unwrap()
        .execute(|_| ())
        .unwrap_err()
        .to_string();

        assert_eq!(
            "Client returned a non-zero status code and said: oii stderr",
            actual.trim()
        );
    }
}
