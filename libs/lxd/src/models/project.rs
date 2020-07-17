use crate::LxdProjectName;
use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct LxdProject {
    pub name: LxdProjectName,
}
