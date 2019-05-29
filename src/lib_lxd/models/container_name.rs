use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct LxdContainerName(String);

impl LxdContainerName {
    pub fn new(name: &str) -> Self {
        Self(name.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LxdContainerName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}