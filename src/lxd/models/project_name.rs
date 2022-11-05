use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct LxdProjectName(String);

impl LxdProjectName {
    #[cfg(test)]
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(name.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_default(&self) -> bool {
        self.as_str() == "default"
    }
}

#[cfg(test)]
impl Default for LxdProjectName {
    fn default() -> Self {
        Self::new("default")
    }
}

impl fmt::Display for LxdProjectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
