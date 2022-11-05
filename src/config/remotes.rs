use crate::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(transparent)]
pub struct Remotes {
    remotes: Vec<LxdRemoteName>,
}

impl Remotes {
    pub fn has_any_non_local_remotes(&self) -> bool {
        self.remotes.iter().any(|remote| remote.as_str() != "local")
    }

    pub fn iter(&self) -> impl Iterator<Item = &LxdRemoteName> {
        self.remotes.iter()
    }
}

impl Default for Remotes {
    fn default() -> Self {
        Self {
            remotes: vec![LxdRemoteName::local()],
        }
    }
}
