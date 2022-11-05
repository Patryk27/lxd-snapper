mod hooks;
mod policies;
mod policy;
mod remotes;

use crate::prelude::*;
use chrono::TimeZone;
use serde::Deserialize;
use std::{fmt::Display, fs, path::Path};

pub use self::{hooks::*, policies::*, policy::*, remotes::*};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default_snapshot_name_prefix")]
    snapshot_name_prefix: String,

    #[serde(default = "default_snapshot_name_format")]
    snapshot_name_format: String,

    #[serde(default)]
    hooks: Hooks,

    #[serde(default)]
    remotes: Remotes,

    #[serde(default)]
    policies: Policies,
}

impl Config {
    #[cfg(test)]
    pub fn parse(code: &str) -> Self {
        serde_yaml::from_str(code).unwrap()
    }

    pub fn load(file: impl AsRef<Path>) -> Result<Self> {
        let file = file.as_ref();

        let result: Result<_> = (|| {
            let code = fs::read_to_string(file).context("Couldn't read file")?;
            serde_yaml::from_str(&code).context("Couldn't parse file")
        })();

        result.with_context(|| format!("Couldn't load configuration from: {}", file.display()))
    }

    pub fn hooks(&self) -> &Hooks {
        &self.hooks
    }

    pub fn policies(&self) -> &Policies {
        &self.policies
    }

    pub fn remotes(&self) -> &Remotes {
        &self.remotes
    }

    pub fn snapshot_name<Tz>(&self, now: DateTime<Tz>) -> LxdSnapshotName
    where
        Tz: TimeZone,
        Tz::Offset: Display,
    {
        let format = format!("{}{}", self.snapshot_name_prefix, self.snapshot_name_format);

        LxdSnapshotName::new(now.format(&format).to_string())
    }

    pub fn matches_snapshot_name(&self, name: &LxdSnapshotName) -> bool {
        name.as_str().starts_with(&self.snapshot_name_prefix)
    }
}

fn default_snapshot_name_prefix() -> String {
    "auto-".into()
}

fn default_snapshot_name_format() -> String {
    "%Y%m%d-%H%M%S".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::utils::*;

    mod load {
        use super::*;

        #[test]
        fn examples() {
            let examples: Vec<_> = glob::glob("docs/example-configs/*.yaml")
                .unwrap()
                .into_iter()
                .map(|example| example.unwrap())
                .collect();

            if examples.is_empty() {
                panic!("Found no example configs");
            }

            for example in examples {
                Config::load(&example).unwrap();
            }
        }
    }

    mod snapshot_name {
        use super::*;
        use test_case::test_case;

        #[test_case("", "%Y%m%d-%H%M%S", "20120824-123456")]
        #[test_case("auto-", "%Y%m%d-%H%M%S", "auto-20120824-123456")]
        #[test_case("auto-", "%Y%m%d", "auto-20120824")]
        fn test(snapshot_name_prefix: &str, snapshot_name_format: &str, expected: &str) {
            let target = Config {
                snapshot_name_prefix: snapshot_name_prefix.into(),
                snapshot_name_format: snapshot_name_format.into(),
                ..Default::default()
            };

            let actual = target
                .snapshot_name(DateTime::parse_from_rfc3339("2012-08-24T12:34:56-00:00").unwrap());

            assert_eq!(expected, actual.as_str());
        }
    }

    mod matches_snapshot_name {
        use super::*;

        fn target(snapshot_name_prefix: &str) -> Config {
            Config {
                snapshot_name_prefix: snapshot_name_prefix.into(),
                ..Default::default()
            }
        }

        #[test]
        fn given_empty_prefix() {
            let target = target("");

            assert!(target.matches_snapshot_name(&snapshot_name("auto")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824-123456")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

            assert!(target.matches_snapshot_name(&snapshot_name("manual")));
            assert!(target.matches_snapshot_name(&snapshot_name("manual-20120824")));
            assert!(target.matches_snapshot_name(&snapshot_name("manual-20120824-123456")));
            assert!(target.matches_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
        }

        #[test]
        fn given_some_prefix() {
            let target = target("auto-");

            assert!(!target.matches_snapshot_name(&snapshot_name("auto")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824-123456")));
            assert!(target.matches_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

            assert!(!target.matches_snapshot_name(&snapshot_name("manual")));
            assert!(!target.matches_snapshot_name(&snapshot_name("manual-20120824")));
            assert!(!target.matches_snapshot_name(&snapshot_name("manual-20120824-123456")));
            assert!(!target.matches_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
        }
    }
}
