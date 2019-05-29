use std::{fs, path::Path};

use indexmap::IndexMap;
use serde::Deserialize;

use lib_lxd::LxdContainer;

use crate::{Error, Policy, Result};

/// This structure defines the `config.yml` configuration file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Path to the LXC client, if one wants to overwrite the default one.
    #[serde(default, rename = "lxc-path")]
    pub lxc_path: Option<String>,

    /// Map of container snapshotting policies, which determine which containers should be
    /// snapshotted, prunned and so on.
    pub policies: IndexMap<String, Policy>,
}

impl Config {
    /// Loads configuration from specified Yaml file.
    ///
    /// # Errors
    ///
    /// Returns an error when specified file does not exist, is an invalid Yaml file or does not
    /// match configuration file's schema.
    pub fn from_yaml_file(file: &Path) -> Result<Self> {
        let config = fs::read_to_string(file).map_err(|err| {
            Error::IoError(file.into(), err)
        })?;

        serde_yaml::from_str(&config).map_err(|err| {
            Error::SerdeError(file.into(), err)
        })
    }

    /// Determines policy for specified container, merging all the policies that match specified
    /// container's name, status and so on.
    ///
    /// # Errors
    ///
    /// This function does not fail, although it returns `None` if no policy matches specified
    /// container.
    pub fn determine_policy_for(&self, container: &LxdContainer) -> Option<Policy> {
        self.policies
            .values()
            .filter(|policy| policy.applies_to(container))
            .fold(None, |result_policy, current_policy| {
                Some(result_policy
                    .unwrap_or_default()
                    .merge_with(current_policy))
            })
    }
}
