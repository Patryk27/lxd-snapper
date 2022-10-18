use crate::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Remote {
    Name(LxdRemoteName),

    NameAndConfig {
        name: LxdRemoteName,

        #[serde(default)]
        pull_snapshots: bool,
    },
}

impl Remote {
    pub fn local() -> Self {
        Self::Name(LxdRemoteName::new("local"))
    }

    pub fn name(&self) -> &LxdRemoteName {
        match self {
            Remote::Name(name) => name,
            Remote::NameAndConfig { name, .. } => name,
        }
    }
}
