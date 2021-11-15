mod hooks;
mod policies;
mod policy;

use crate::prelude::*;
use chrono::TimeZone;
use lib_lxd::LxdSnapshotName;
use serde::Deserialize;
use std::{fmt::Display, fs, path::Path};

pub use self::{hooks::*, policies::*, policy::*};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default_snapshot_name_prefix")]
    pub snapshot_name_prefix: String,
    #[serde(default)]
    pub hooks: Hooks,
    #[serde(default)]
    pub policies: Policies,
}

impl Config {
    #[cfg(test)]
    pub fn from_code(code: &str) -> Self {
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

    pub fn snapshot_name<Tz>(&self, now: DateTime<Tz>) -> LxdSnapshotName
    where
        Tz: TimeZone,
        Tz::Offset: Display,
    {
        let format = format!("{}%Y%m%d-%H%M%S", self.snapshot_name_prefix);
        LxdSnapshotName::new(now.format(&format).to_string())
    }

    pub fn matches_snapshot_name(&self, name: &LxdSnapshotName) -> bool {
        name.as_str().starts_with(&self.snapshot_name_prefix)
    }
}

fn default_snapshot_name_prefix() -> String {
    "auto-".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_lxd::test_utils::*;

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
        use chrono::FixedOffset;

        fn config(snapshot_name_prefix: &str) -> Config {
            Config {
                snapshot_name_prefix: snapshot_name_prefix.into(),
                ..Default::default()
            }
        }

        fn now() -> DateTime<FixedOffset> {
            DateTime::parse_from_rfc3339("2012-08-24T12:34:56-00:00").unwrap()
        }

        mod given_empty_prefix {
            use super::*;

            #[test]
            fn returns_snapshot_name() {
                let actual = config("").snapshot_name(now());
                let expected = "20120824-123456";

                pa::assert_eq!(expected, actual.as_str());
            }
        }

        mod given_some_prefix {
            use super::*;

            #[test]
            fn returns_snapshot_name() {
                let actual = config("auto-").snapshot_name(now());
                let expected = "auto-20120824-123456";

                pa::assert_eq!(expected, actual.as_str());
            }
        }
    }

    mod matches_snapshot_name {
        use super::*;

        fn config(snapshot_name_prefix: &str) -> Config {
            Config {
                snapshot_name_prefix: snapshot_name_prefix.into(),
                ..Default::default()
            }
        }

        mod given_empty_prefix {
            use super::*;

            #[test]
            fn returns_always_true() {
                let config = config("");

                assert!(config.matches_snapshot_name(&snapshot_name("auto")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824-123456")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

                assert!(config.matches_snapshot_name(&snapshot_name("manual")));
                assert!(config.matches_snapshot_name(&snapshot_name("manual-20120824")));
                assert!(config.matches_snapshot_name(&snapshot_name("manual-20120824-123456")));
                assert!(config.matches_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
            }
        }

        mod given_some_prefix {
            use super::*;

            #[test]
            fn returns_true_when_snapshot_name_begins_with_this_prefix() {
                let config = config("auto-");

                assert!(!config.matches_snapshot_name(&snapshot_name("auto")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824-123456")));
                assert!(config.matches_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

                assert!(!config.matches_snapshot_name(&snapshot_name("manual")));
                assert!(!config.matches_snapshot_name(&snapshot_name("manual-20120824")));
                assert!(!config.matches_snapshot_name(&snapshot_name("manual-20120824-123456")));
                assert!(!config.matches_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
            }
        }
    }
}
