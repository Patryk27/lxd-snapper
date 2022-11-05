use super::serde::null_to_default;
use crate::lxd::{LxdInstanceName, LxdInstanceStatus, LxdSnapshot};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LxdInstance {
    pub name: LxdInstanceName,
    pub status: LxdInstanceStatus,

    // We need `null_to_default`, because LXC returns `null` for instances that
    // don't have any snapshots (instead of `[]`, as one could guess)
    #[serde(deserialize_with = "null_to_default")]
    pub snapshots: Vec<LxdSnapshot>,
}
