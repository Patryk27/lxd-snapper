use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Policy {
    pub included_remotes: Option<HashSet<LxdRemoteName>>,
    pub excluded_remotes: Option<HashSet<LxdRemoteName>>,
    pub included_projects: Option<HashSet<LxdProjectName>>,
    pub excluded_projects: Option<HashSet<LxdProjectName>>,
    pub included_instances: Option<HashSet<LxdInstanceName>>,
    pub excluded_instances: Option<HashSet<LxdInstanceName>>,
    pub included_statuses: Option<HashSet<LxdInstanceStatus>>,
    pub excluded_statuses: Option<HashSet<LxdInstanceStatus>>,
    pub keep_hourly: Option<usize>,
    pub keep_daily: Option<usize>,
    pub keep_weekly: Option<usize>,
    pub keep_monthly: Option<usize>,
    pub keep_yearly: Option<usize>,
    pub keep_last: Option<usize>,
    pub keep_limit: Option<usize>,
}

impl Policy {
    pub fn keep_hourly(&self) -> usize {
        self.keep_hourly.unwrap_or(0)
    }

    pub fn keep_daily(&self) -> usize {
        self.keep_daily.unwrap_or(0)
    }

    pub fn keep_weekly(&self) -> usize {
        self.keep_weekly.unwrap_or(0)
    }

    pub fn keep_monthly(&self) -> usize {
        self.keep_monthly.unwrap_or(0)
    }

    pub fn keep_yearly(&self) -> usize {
        self.keep_yearly.unwrap_or(0)
    }

    pub fn keep_last(&self) -> usize {
        self.keep_last.unwrap_or(0)
    }

    pub fn keep_limit(&self) -> Option<usize> {
        self.keep_limit
    }

    pub fn applies_to(
        &self,
        remote: &LxdRemoteName,
        project: &LxdProject,
        instance: &LxdInstance,
    ) -> bool {
        fn set_contains<T>(items: &Option<HashSet<T>>, item: &T, default: bool) -> bool
        where
            T: Hash + Eq,
        {
            items
                .as_ref()
                .map(|items| items.contains(item))
                .unwrap_or(default)
        }

        let remote_included = set_contains(&self.included_remotes, remote, true);
        let remote_excluded = set_contains(&self.excluded_remotes, remote, false);

        let project_included = set_contains(&self.included_projects, &project.name, true);
        let project_excluded = set_contains(&self.excluded_projects, &project.name, false);

        let instance_included = set_contains(&self.included_instances, &instance.name, true);
        let instance_excluded = set_contains(&self.excluded_instances, &instance.name, false);

        let status_included = set_contains(&self.included_statuses, &instance.status, true);
        let status_excluded = set_contains(&self.excluded_statuses, &instance.status, false);

        remote_included
            && !remote_excluded
            && project_included
            && !project_excluded
            && instance_included
            && !instance_excluded
            && status_included
            && !status_excluded
    }

    pub fn merge_with(self, other: Self) -> Self {
        Self {
            included_remotes: None,
            excluded_remotes: None,
            included_projects: None,
            excluded_projects: None,
            included_instances: None,
            excluded_instances: None,
            included_statuses: None,
            excluded_statuses: None,
            keep_hourly: other.keep_hourly.or(self.keep_hourly),
            keep_daily: other.keep_daily.or(self.keep_daily),
            keep_weekly: other.keep_weekly.or(self.keep_weekly),
            keep_monthly: other.keep_monthly.or(self.keep_monthly),
            keep_yearly: other.keep_yearly.or(self.keep_yearly),
            keep_last: other.keep_last.or(self.keep_last),
            keep_limit: other.keep_limit.or(self.keep_limit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod applies_to {
        use super::*;

        fn remotes(names: &[&str]) -> HashSet<LxdRemoteName> {
            names.iter().map(LxdRemoteName::new).collect()
        }

        fn projects(names: &[&str]) -> HashSet<LxdProjectName> {
            names.iter().map(LxdProjectName::new).collect()
        }

        fn instances(names: &[&str]) -> HashSet<LxdInstanceName> {
            names.iter().map(LxdInstanceName::new).collect()
        }

        fn statuses(statuses: &[LxdInstanceStatus]) -> HashSet<LxdInstanceStatus> {
            statuses.iter().cloned().collect()
        }

        fn check(
            policy: &Policy,
            expected: impl Fn(&LxdRemoteName, &LxdProject, &LxdInstance) -> bool,
        ) {
            let mut matching_instances = 0;

            let instances = [
                ("instance-a", LxdInstanceStatus::Running),
                ("instance-b", LxdInstanceStatus::Aborting),
                ("instance-c", LxdInstanceStatus::Stopped),
            ];

            for remote in ["remote-a", "remote-b", "remote-c"] {
                let remote = LxdRemoteName::new(remote);

                for project in ["project-a", "project-b", "project-c"] {
                    let project = LxdProject {
                        name: LxdProjectName::new(project),
                    };

                    for (instance_name, instance_status) in instances {
                        let instance = LxdInstance {
                            name: LxdInstanceName::new(instance_name),
                            status: instance_status,
                            snapshots: Default::default(),
                        };

                        let actual = policy.applies_to(&remote, &project, &instance);
                        let expected = expected(&remote, &project, &instance);

                        assert_eq!(
                            expected, actual,
                            "\nAssertion failed for:\n- remote = {:?}\n- project = {:?}\n- instance = {:?}",
                            remote, project, instance
                        );

                        if actual {
                            matching_instances += 1;
                        }
                    }
                }
            }

            assert!(
                matching_instances > 0,
                "Tested policy doesn't match any instance"
            );
        }

        #[test]
        fn given_policy_with_no_restrictions() {
            check(&Policy::default(), |_, _, _| true);
        }

        #[test]
        fn given_policy_with_included_remotes() {
            let policy = Policy {
                included_remotes: Some(remotes(&["remote-a", "remote-c"])),
                ..Default::default()
            };

            check(&policy, |remote, _, _| {
                ["remote-a", "remote-c"].contains(&remote.as_str())
            });
        }

        #[test]
        fn given_policy_with_excluded_remotes() {
            let policy = Policy {
                excluded_remotes: Some(remotes(&["remote-a", "remote-c"])),
                ..Default::default()
            };

            check(&policy, |remote, _, _| {
                !["remote-a", "remote-c"].contains(&remote.as_str())
            });
        }

        #[test]
        fn given_policy_with_included_projects() {
            let policy = Policy {
                included_projects: Some(projects(&["project-a", "project-c"])),
                ..Default::default()
            };

            check(&policy, |_, project, _| {
                ["project-a", "project-c"].contains(&project.name.as_str())
            });
        }

        #[test]
        fn given_policy_with_excluded_projects() {
            let policy = Policy {
                excluded_projects: Some(projects(&["project-a", "project-c"])),
                ..Default::default()
            };

            check(&policy, |_, project, _| {
                !["project-a", "project-c"].contains(&project.name.as_str())
            });
        }

        #[test]
        fn given_policy_with_included_instances() {
            let policy = Policy {
                included_instances: Some(instances(&["instance-a", "instance-c"])),
                ..Default::default()
            };

            check(&policy, |_, _, instance| {
                ["instance-a", "instance-c"].contains(&instance.name.as_str())
            });
        }

        #[test]
        fn given_policy_with_excluded_instances() {
            let policy = Policy {
                excluded_instances: Some(instances(&["instance-a", "instance-c"])),
                ..Default::default()
            };

            check(&policy, |_, _, instance| {
                !["instance-a", "instance-c"].contains(&instance.name.as_str())
            });
        }

        #[test]
        fn given_policy_with_included_statuses() {
            let policy = Policy {
                included_statuses: Some(statuses(&[
                    LxdInstanceStatus::Aborting,
                    LxdInstanceStatus::Stopped,
                ])),
                ..Default::default()
            };

            check(&policy, |_, _, instance| {
                [LxdInstanceStatus::Aborting, LxdInstanceStatus::Stopped].contains(&instance.status)
            });
        }

        #[test]
        fn given_policy_with_excluded_statuses() {
            let policy = Policy {
                excluded_statuses: Some(statuses(&[
                    LxdInstanceStatus::Aborting,
                    LxdInstanceStatus::Stopped,
                ])),
                ..Default::default()
            };

            check(&policy, |_, _, instance| {
                ![LxdInstanceStatus::Aborting, LxdInstanceStatus::Stopped]
                    .contains(&instance.status)
            });
        }

        #[test]
        fn given_policy_with_mixed_rules() {
            let policy = Policy {
                included_remotes: Some(remotes(&["remote-a", "remote-b"])),
                included_projects: Some(projects(&["project-b", "project-c"])),
                included_instances: Some(instances(&["instance-a", "instance-c"])),
                ..Default::default()
            };

            check(&policy, |remote, project, instance| {
                let remote_matches = ["remote-a", "remote-b"].contains(&remote.as_str());
                let project_matches = ["project-b", "project-c"].contains(&project.name.as_str());
                let instance_matches =
                    ["instance-a", "instance-c"].contains(&instance.name.as_str());

                remote_matches && project_matches && instance_matches
            });
        }
    }

    #[test]
    fn merge_with() {
        // Policy A: has all values set, serves as a base
        let policy_a = Policy {
            keep_daily: Some(10),
            keep_weekly: Some(5),
            keep_monthly: Some(2),
            keep_yearly: Some(1),
            keep_last: Some(8),
            ..Default::default()
        };

        // Policy B: overwrites only the `keep weekly` and `keep monthly` options
        let policy_b = Policy {
            keep_weekly: Some(100),
            keep_monthly: Some(200),
            ..Default::default()
        };

        // Policy C: overwrites only the `keep yearly` option
        let policy_c = Policy {
            keep_yearly: Some(100),
            ..Default::default()
        };

        // Policy A + B
        let policy_ab = Policy {
            keep_weekly: Some(100),  // Overwritten from policy B
            keep_monthly: Some(200), // Overwritten from policy B
            ..policy_a.clone()
        };

        // Policy A + C
        let policy_ac = Policy {
            keep_yearly: Some(100), // Overwritten from policy C
            ..policy_a.clone()
        };

        pa::assert_eq!(policy_a.clone().merge_with(policy_b), policy_ab);
        pa::assert_eq!(policy_a.merge_with(policy_c), policy_ac);
    }
}
