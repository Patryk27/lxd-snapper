use std::{
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};

use serde::de::DeserializeOwned;

use crate::{Error, LxdClient, LxdContainer, LxdContainerName, LxdSnapshotName, Result};

/// A client that communicates with the LXD daemon through the LXC CLI application.
///
/// It was just way easier to create an LXC wrapper rather than fully-fledged
/// communicating-through-socket application.
pub struct LxdProcessClient {
    lxc_path: PathBuf,
}

impl LxdProcessClient {
    /// Creates a new client that communicates with the LXD daemon through the LXC CLI application.
    ///
    /// Contrary to the `LxdProcessClient::new_autodetect_path()` method, this one requires
    /// specifying full path to the LXC client.
    ///
    /// # Example
    ///
    /// ```
    /// # use lib_lxd::LxdProcessClient;
    /// # use std::path::Path;
    ///
    /// let lxd = LxdProcessClient::new(
    ///   Path::new("/snap/bin/lxc")
    /// );
    /// ```
    pub fn new(lxc_path: &Path) -> Self {
        Self { lxc_path: lxc_path.into() }
    }

    /// Creates a new client that communicates with the LXD daemon through the LXC CLI application.
    ///
    /// Contrary to the `LxdProcessClient::new()` method, this one tries to automatically detect
    /// where the LXC client was installed.
    pub fn new_detect() -> Result<Self> {
        let paths = [
            // LXD installed from Snap:
            "/snap/bin/lxc",

            // LXD installed from apt (Ubuntu):
            "/usr/bin/lxc",

            // Other possible paths, but not encountered by me in the wild:
            "/usr/local/bin/lxc",
            "/usr/local/sbin/lxc",
            "/bin/lxc",
            "/sbin/lxc",
        ];

        for path in &paths {
            let path = Path::new(path);

            if std::fs::metadata(path).is_ok() {
                return Ok(
                    LxdProcessClient::new(path)
                );
            }
        }

        Err(Error::FailedToAutodetect)
    }

    fn execute(&mut self, args: &[&str]) -> Result<String> {
        let out = Command::new(&self.lxc_path)
            .args(args)
            .output();

        let out = out.map_err(|err| {
            Error::FailedToExecute(self.lxc_path.clone(), err)
        })?;

        if out.status.success() {
            let stdout = String::from_utf8(out.stdout)
                .unwrap();

            Ok(stdout)
        } else {
            let stderr = String::from_utf8(out.stderr)
                .unwrap()
                .trim()
                .to_string();

            Err(Error::ClientError(stderr))
        }
    }

    fn parse<T: DeserializeOwned>(out: String) -> Result<T> {
        serde_json::from_reader(
            Cursor::new(out)
        ).map_err(Error::FailedToParse)
    }
}

impl LxdClient for LxdProcessClient {
    fn check_connection(&mut self) -> Result {
        self.list().map(|_| ())
    }

    fn create_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        self.execute(&[
            "snapshot",
            container_name.inner(),
            snapshot_name.inner(),
        ])?;

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        self.execute(&[
            "delete",
            &format!("{}/{}", container_name.inner(), snapshot_name.inner()),
        ])?;

        Ok(())
    }

    fn list(&mut self) -> Result<Vec<LxdContainer>> {
        let result = self.execute(&[
            "list",
            "--format=json",
        ])?;

        Self::parse(result)
    }
}