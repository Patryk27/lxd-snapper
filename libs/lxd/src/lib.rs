//! A minimal implementation of the LXD client for use in `lxd-snapper`.

pub use self::{clients::*, error::*, models::*, result::*};

mod clients;
mod error;
mod models;
mod result;

pub mod test_utils;

pub trait LxdClient {
    /// Returns all the projects.
    fn list_projects(&mut self) -> Result<Vec<LxdProject>>;

    /// Returns all the instances for given project.
    /// When given project does not exist, returns an empty list.
    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdInstance>>;

    /// Creates a new, stateless snapshot.
    ///
    /// # Errors
    ///
    /// - Fails when given instance does not exist.
    /// - Fails when given snapshot already exists.
    fn create_snapshot(
        &mut self,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> Result<()>;

    /// Deletes specified snapshot.
    ///
    /// # Errors
    ///
    /// - Fails when given instance does not exist.
    /// - Fails when given snapshot does not exist.
    fn delete_snapshot(
        &mut self,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> Result<()>;
}
