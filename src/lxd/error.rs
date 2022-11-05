use crate::lxd::{LxdInstanceName, LxdProjectName, LxdRemoteName, LxdSnapshotName};
use std::result;
use thiserror::Error;

pub type LxdResult<T> = result::Result<T, LxdError>;

#[derive(Debug, Error)]
pub enum LxdError {
    #[error("No such instance: {} (in project `{project}`)", .instance.on(.remote))]
    NoSuchInstance {
        remote: LxdRemoteName,
        project: LxdProjectName,
        instance: LxdInstanceName,
    },

    #[error("No such snapshot: {snapshot} (on instance `{}` in project `{project}`)", .instance.on(.remote))]
    NoSuchSnapshot {
        remote: LxdRemoteName,
        project: LxdProjectName,
        instance: LxdInstanceName,
        snapshot: LxdSnapshotName,
    },

    #[error(
        "Snapshot already exists: {snapshot} (on instance `{}` in project `{project}`)", .instance.on(.remote)
    )]
    SnapshotAlreadyExists {
        remote: LxdRemoteName,
        project: LxdProjectName,
        instance: LxdInstanceName,
        snapshot: LxdSnapshotName,
    },

    #[cfg(test)]
    #[error("InjectedError")]
    InjectedError,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(test)]
impl PartialEq<LxdError> for LxdError {
    fn eq(&self, other: &LxdError) -> bool {
        self.to_string() == other.to_string()
    }
}
