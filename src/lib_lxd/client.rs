use crate::{LxdContainer, LxdContainerName, LxdSnapshotName, Result};

pub trait LxdClient {
    /// Checks whether we can connect to LXC.
    /// Returns an error when e.g. user passes invalid process name into the `LxdProcessClient`.
    ///
    /// # Example
    ///
    /// ```
    /// use lib_lxd::*;
    ///
    /// let mut lxd = LxdInMemoryClient::new(vec![
    ///     LxdContainer {
    ///         name: LxdContainerName::new("sacred-penguin"),
    ///         status: LxdContainerStatus::Running,
    ///         snapshots: None,
    ///     },
    /// ]);
    ///
    /// if let Err(err) = lxd.check_connection() {
    ///     panic!("Failed to connect to LXD: {}", err);
    /// }
    /// ```
    fn check_connection(&mut self) -> Result;

    /// Creates a new LXC stateless snapshot.
    ///
    /// # Example
    ///
    /// ```
    /// use lib_lxd::*;
    ///
    /// let mut lxd = LxdInMemoryClient::new(vec![
    ///     LxdContainer {
    ///         name: LxdContainerName::new("sacred-penguin"),
    ///         status: LxdContainerStatus::Running,
    ///         snapshots: None,
    ///     },
    /// ]);
    ///
    /// lxd.create_snapshot(
    ///   &LxdContainerName::new("sacred-penguin"),
    ///   &LxdSnapshotName::new("my-snapshot")
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// - Fails when given container does not exist.
    /// - Fails when given snapshot already exists.
    fn create_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result;

    /// Deletes given LXC snapshot.
    ///
    /// # Example
    ///
    /// ```
    /// use lib_lxd::*;
    ///
    /// let mut lxd = LxdInMemoryClient::new(vec![
    ///     LxdContainer {
    ///         name: LxdContainerName::new("sacred-penguin"),
    ///         status: LxdContainerStatus::Running,
    ///         snapshots: None,
    ///     },
    /// ]);
    ///
    /// lxd.create_snapshot(
    ///   &LxdContainerName::new("sacred-penguin"),
    ///   &LxdSnapshotName::new("my-snapshot")
    /// ).unwrap();
    ///
    /// lxd.delete_snapshot(
    ///   &LxdContainerName::new("sacred-penguin"),
    ///   &LxdSnapshotName::new("my-snapshot")
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// - Fails when given container does not exist.
    /// - Fails when given snapshot does not exist.
    fn delete_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result;

    /// Lists all the LXC containers there are.
    fn list(&mut self) -> Result<Vec<LxdContainer>>;
}