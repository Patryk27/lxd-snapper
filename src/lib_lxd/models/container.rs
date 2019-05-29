use serde::Deserialize;

use crate::{LxdContainerName, LxdContainerStatus, LxdSnapshot};

#[derive(Clone, Debug, Deserialize)]
pub struct LxdContainer {
    pub name: LxdContainerName,
    pub status: LxdContainerStatus,
    pub snapshots: Option<Vec<LxdSnapshot>>,
}
