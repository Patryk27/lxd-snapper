use crate::{LxdInstanceName, LxdProjectName, LxdSnapshotName};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No such instance: {0} (for project `{project}`)")]
    NoSuchInstance {
        project: LxdProjectName,
        instance: LxdInstanceName,
    },

    #[error("No such snapshot: {snapshot} (for instance `{instance}` in project `{project}`)")]
    NoSuchSnapshot {
        project: LxdProjectName,
        instance: LxdInstanceName,
        snapshot: LxdSnapshotName,
    },

    #[error(
        "Snapshot already exists: {snapshot} (for instance `{instance}` in project `{project}`)"
    )]
    SnapshotAlreadyExists {
        project: LxdProjectName,
        instance: LxdInstanceName,
        snapshot: LxdSnapshotName,
    },

    #[error(transparent)]
    Other(anyhow::Error),
}

#[cfg(test)]
impl PartialEq<Error> for Error {
    fn eq(&self, other: &Error) -> bool {
        self.to_string() == other.to_string()
    }
}
