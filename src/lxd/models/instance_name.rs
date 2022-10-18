use crate::lxd::LxdRemoteName;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct LxdInstanceName(String);

impl LxdInstanceName {
    #[cfg(test)]
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(name.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn on(&self, remote: &LxdRemoteName) -> String {
        format!("{}:{}", remote, self)
    }
}

impl fmt::Display for LxdInstanceName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on() {
        let instance = LxdInstanceName::new("some-instance");
        let remote = LxdRemoteName::new("some-remote");

        assert_eq!("some-remote:some-instance", instance.on(&remote));
    }
}
