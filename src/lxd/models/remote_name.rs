use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct LxdRemoteName(String);

impl LxdRemoteName {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(name.as_ref().into())
    }

    pub fn local() -> Self {
        Self::new("local")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Default for LxdRemoteName {
    fn default() -> Self {
        Self::local()
    }
}

impl fmt::Display for LxdRemoteName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
