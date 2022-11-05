use crate::lxd::LxdSnapshotName;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LxdSnapshot {
    pub name: LxdSnapshotName,
    pub created_at: DateTime<Utc>,
}
