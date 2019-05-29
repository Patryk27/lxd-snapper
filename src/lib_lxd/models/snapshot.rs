use chrono::{DateTime, Local};
use serde::Deserialize;

use crate::LxdSnapshotName;

#[derive(Clone, Debug, Deserialize)]
pub struct LxdSnapshot {
    pub name: LxdSnapshotName,
    pub created_at: DateTime<Local>,
}