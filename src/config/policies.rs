use crate::prelude::*;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(transparent)]
pub struct Policies {
    policies: IndexMap<String, Policy>,
}

impl Policies {
    pub fn find<'a>(
        &'a self,
        remote: &'a LxdRemoteName,
        project: &'a LxdProject,
        instance: &'a LxdInstance,
    ) -> impl Iterator<Item = (&'a str, &'a Policy)> + 'a {
        self.policies
            .iter()
            .filter(|(_, policy)| policy.applies_to(remote, project, instance))
            .map(|(name, policy)| (name.as_str(), policy))
    }

    pub fn matches(
        &self,
        remote: &LxdRemoteName,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> bool {
        self.find(remote, project, instance).next().is_some()
    }

    pub fn build(
        &self,
        remote: &LxdRemoteName,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> Option<Policy> {
        self.find(remote, project, instance)
            .map(|(_, policy)| policy)
            .fold(None, |result, current| {
                Some(result.unwrap_or_default().merge_with(current.clone()))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod build {
        use super::*;

        fn config(file: &str) -> Config {
            Config::load(format!("docs/example-configs/{}.yaml", file)).unwrap()
        }

        fn assert_policy(
            config: &Config,
            remote_name: &str,
            project_name: &str,
            instance_name: &str,
            instance_status: LxdInstanceStatus,
            expected_policy: Option<Policy>,
        ) {
            let remote_name = LxdRemoteName::new(remote_name);

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
                config.policies.build(&remote_name, &project, &instance),
                "remote_name={remote_name}, project_name={project_name}, instance_name={instance_name}, instance_status={instance_status}",
                remote_name = remote_name.as_str(),
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
                "local",
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
                "local",
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
                "local",
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
                "local",
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
                "local",
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
                "local",
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
                "local",
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
                "local",
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
