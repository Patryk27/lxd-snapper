use crate::LxdSnapshotName;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct LxdSnapshot {
    pub name: LxdSnapshotName,
    pub created_at: DateTime<Utc>,
}
