use crate::lxd::LxdProjectName;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LxdProject {
    pub name: LxdProjectName,
}
