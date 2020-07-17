//! A minimal implementation of the LXD client for use in `lxd-snapper`.

#![feature(try_blocks)]

pub use self::{clients::*, error::*, models::*, result::*};

mod clients;
mod error;
mod models;
mod result;

pub mod test_utils;

pub trait LxdClient {
    /// Lists all the containers for given project.
    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdContainer>>;

    /// Lists all the projects.
    fn list_projects(&mut self) -> Result<Vec<LxdProject>>;

    /// Creates a new, stateless snapshot.
    ///
    /// # Errors
    ///
    /// - Fails when given container does not exist.
    /// - Fails when given snapshot already exists.
    fn create_snapshot(
        &mut self,
        project: &LxdProjectName,
        container: &LxdContainerName,
        snapshot: &LxdSnapshotName,
    ) -> Result;

    /// Deletes specified snapshot.
    ///
    /// # Errors
    ///
    /// - Fails when given container does not exist.
    /// - Fails when given snapshot does not exist.
    fn delete_snapshot(
        &mut self,
        project: &LxdProjectName,
        container: &LxdContainerName,
        snapshot: &LxdSnapshotName,
    ) -> Result;
}
