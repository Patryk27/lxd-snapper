use anyhow::*;
use chrono::{DateTime, TimeZone};
use indexmap::IndexMap;
use lib_lxd::{LxdInstance, LxdProject, LxdSnapshotName};
use serde::export::fmt::Display;
use serde::Deserialize;
use std::{fs, path::Path};

pub use self::policy::*;

mod policy;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default_snapshot_name_prefix")]
    snapshot_name_prefix: String,
    policies: IndexMap<String, Policy>,
}

impl Config {
    /// Loads settings from given file.
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self> {
        let file = file.as_ref();

        let result: Result<Self> = (|| {
            let code = fs::read_to_string(file).context("Couldn't read file")?;
            serde_yaml::from_str(&code).context("Couldn't parse file")
        })();

        result.with_context(|| format!("Couldn't load configuration from: {}", file.display()))
    }

    /// Load settings from given YAML code.
    #[cfg(test)]
    pub fn from_code(code: &str) -> Self {
        serde_yaml::from_str(&code).unwrap()
    }

    /// Builds snapshot name for given date & time.
    pub fn snapshot_name<Tz>(&self, now: DateTime<Tz>) -> LxdSnapshotName
    where
        Tz: TimeZone,
        Tz::Offset: Display,
    {
        let format = format!("{}%Y%m%d-%H%M%S", self.snapshot_name_prefix);
        LxdSnapshotName::new(now.format(&format).to_string())
    }

    /// Returns whether given snapshot name matches the one specified in the
    /// configuration.
    pub fn is_snapshot_name(&self, name: &LxdSnapshotName) -> bool {
        name.as_str().starts_with(&self.snapshot_name_prefix)
    }

    /// Returns all policies applying for specified project & instance.
    /// If no policy matches given criteria, returns an empty list.
    pub fn policies(&self, project: &LxdProject, instance: &LxdInstance) -> Vec<(&str, &Policy)> {
        self.policies
            .iter()
            .filter(|(_, policy)| policy.applies_to(project, instance))
            .map(|(name, policy)| (name.as_str(), policy))
            .collect()
    }

    /// Returns policy for specified project & instance.
    /// If no policy matches given criteria, returns `None`.
    pub fn policy(&self, project: &LxdProject, instance: &LxdInstance) -> Option<Policy> {
        self.policies(project, instance)
            .into_iter()
            .map(|(_, policy)| policy)
            .fold(None, |result, current| {
                Some(result.unwrap_or_default().merge_with(current.clone()))
            })
    }
}

fn default_snapshot_name_prefix() -> String {
    "auto-".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_lxd::test_utils::*;
    use lib_lxd::*;
    use pretty_assertions as pa;

    mod from_file {
        use super::*;

        #[test]
        fn load_examples() {
            let examples: Vec<_> = glob::glob("docs/example-configs/*.yaml")
                .unwrap()
                .into_iter()
                .map(|example| example.unwrap())
                .collect();

            if examples.is_empty() {
                panic!("Found no example configs");
            }

            for example in examples {
                Config::from_file(&example).unwrap();
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

    mod is_snapshot_name {
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

                assert!(config.is_snapshot_name(&snapshot_name("auto")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824-123456")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

                assert!(config.is_snapshot_name(&snapshot_name("manual")));
                assert!(config.is_snapshot_name(&snapshot_name("manual-20120824")));
                assert!(config.is_snapshot_name(&snapshot_name("manual-20120824-123456")));
                assert!(config.is_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
            }
        }

        mod given_some_prefix {
            use super::*;

            #[test]
            fn returns_true_when_snapshot_name_begins_with_this_prefix() {
                let config = config("auto-");

                assert!(!config.is_snapshot_name(&snapshot_name("auto")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824-123456")));
                assert!(config.is_snapshot_name(&snapshot_name("auto-20120824-123456-bus")));

                assert!(!config.is_snapshot_name(&snapshot_name("manual")));
                assert!(!config.is_snapshot_name(&snapshot_name("manual-20120824")));
                assert!(!config.is_snapshot_name(&snapshot_name("manual-20120824-123456")));
                assert!(!config.is_snapshot_name(&snapshot_name("manual-20120824-123456-bus")));
            }
        }
    }

    mod policy {
        use super::*;

        fn config(file: &str) -> Config {
            Config::from_file(format!("docs/example-configs/{}.yaml", file)).unwrap()
        }

        fn assert_policy(
            config: &Config,
            project_name: &str,
            instance_name: &str,
            instance_status: LxdInstanceStatus,
            expected_policy: Option<Policy>,
        ) {
            let project = LxdProject {
                name: LxdProjectName::new(project_name),
            };

            let instance = LxdInstance {
                name: LxdInstanceName::new(instance_name),
                status: instance_status,
                snapshots: Default::default(),
            };

            pa::assert_eq!(
                expected_policy,
                config.policy(&project, &instance),
                "project_name={project_name}, instance_name={instance_name}, instance_status={instance_status}",
                project_name = project_name,
                instance_name = instance_name,
                instance_status = format!("{:?}", instance_status),
            );
        }

        #[test]
        fn cascading() {
            let config = config("cascading");

            // -------- //
            // Client A //

            // `everyone` + `important-clients`
            assert_policy(
                &config,
                "client-a",
                "php",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(15),
                    ..Default::default()
                }),
            );

            // `everyone` + `important-clients` + `databases`
            assert_policy(
                &config,
                "client-a",
                "mysql",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(25),
                    ..Default::default()
                }),
            );

            // -------- //
            // Client B //

            // `everyone` + `important-clients`
            assert_policy(
                &config,
                "client-b",
                "php",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(15),
                    ..Default::default()
                }),
            );

            // `everyone` + `important-clients` + `databases`
            assert_policy(
                &config,
                "client-b",
                "mysql",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(25),
                    ..Default::default()
                }),
            );

            // -------- //
            // Client C //

            // `everyone` + `unimportant-clients`
            assert_policy(
                &config,
                "client-c",
                "php",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(5),
                    ..Default::default()
                }),
            );

            // `everyone` + `unimportant-clients` + `databases`
            assert_policy(
                &config,
                "client-c",
                "mysql",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(25),
                    ..Default::default()
                }),
            );

            // -------- //
            // Client D //

            // `everyone`
            assert_policy(
                &config,
                "client-d",
                "php",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(2),
                    ..Default::default()
                }),
            );

            // `everyone` + `databases`
            assert_policy(
                &config,
                "client-d",
                "mysql",
                LxdInstanceStatus::Running,
                Some(Policy {
                    keep_last: Some(25),
                    ..Default::default()
                }),
            );
        }
    }
}
