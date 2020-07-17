use crate::{LxdContainerName, LxdProjectName, LxdSnapshotName};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Couldn't find the `lxc` executable in your PATH - please provide appropriate path via \
         the `--lxc-path` parameter"
    )]
    CouldntFindLxc,

    #[error("Couldn't launch the `lxc` executable")]
    CouldntLaunchLxc(#[source] anyhow::Error),

    #[error("No such container: {0} (for project `{project}`)")]
    NoSuchContainer {
        project: LxdProjectName,
        container: LxdContainerName,
    },

    #[error("No such snapshot: {snapshot} (for container `{container}` in project `{project}`)")]
    NoSuchSnapshot {
        project: LxdProjectName,
        container: LxdContainerName,
        snapshot: LxdSnapshotName,
    },

    #[error(
        "Snapshot already exists: {snapshot} (for container `{container}` in project `{project}`)"
    )]
    SnapshotAlreadyExists {
        project: LxdProjectName,
        container: LxdContainerName,
        snapshot: LxdSnapshotName,
    },
}

#[cfg(test)]
impl PartialEq<Error> for Error {
    fn eq(&self, other: &Error) -> bool {
        self.to_string() == other.to_string()
    }
}
