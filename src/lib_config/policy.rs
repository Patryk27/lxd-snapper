use std::collections::HashSet;

use serde::Deserialize;

use lib_lxd::{LxdContainer, LxdContainerName, LxdContainerStatus};

#[derive(Clone, Debug, Deserialize, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(default, rename = "allowed-containers")]
    pub allowed_containers: Option<HashSet<LxdContainerName>>,

    #[serde(default, rename = "allowed-statuses")]
    pub allowed_statuses: Option<HashSet<LxdContainerStatus>>,

    #[serde(default, rename = "keep-hourly")]
    pub keep_hourly: Option<usize>,

    #[serde(default, rename = "keep-daily")]
    pub keep_daily: Option<usize>,

    #[serde(default, rename = "keep-weekly")]
    pub keep_weekly: Option<usize>,

    #[serde(default, rename = "keep-monthly")]
    pub keep_monthly: Option<usize>,

    #[serde(default, rename = "keep-yearly")]
    pub keep_yearly: Option<usize>,

    #[serde(default, rename = "keep-last")]
    pub keep_last: Option<usize>,

    #[serde(default, rename = "keep-limit")]
    pub keep_limit: Option<usize>,
}

impl Policy {
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Returns whether this policy applies to given container.
    ///
    /// For instance: if this policy is restricted to match only containers with specified names,
    /// it will return `true` only for container matching those names.
    pub fn applies_to(&self, container: &LxdContainer) -> bool {
        let name_allowed = self.allowed_containers.as_ref()
            .map(|names| names.contains(&container.name))
            .unwrap_or(true);

        let status_matches = self.allowed_statuses.as_ref()
            .map(|statuses| statuses.contains(&container.status))
            .unwrap_or(true);

        name_allowed && status_matches
    }

    /// Merges this policy with another one, overwriting fields in a cascading fashion.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Policy A:
    ///   keep-daily: 20
    ///   keep-weekly: 5
    ///
    /// Policy B:
    ///   keep-weekly: 10
    ///   keep-monthly: 8
    ///
    /// Policy A + B:
    ///   keep-daily: 20  # Taken from A
    ///   keep-weekly: 10 # Overwritten from B
    ///   keep-monthly: 8 # Taken from B
    ///
    /// Policy B + A:
    ///   keep-daily: 20  # Taken from A
    ///   keep-weekly: 5  # Overwritten from A
    ///   keep-monthly: 8 # Taken from B
    /// ```
    ///
    /// # Guarantees
    ///
    /// Merging is *not commutative* - merging A with B may result in a different policy than the
    /// one you'd get when merging B with A.
    pub fn merge_with(self, other: &Self) -> Self {
        Self {
            keep_hourly: other.keep_hourly.or(self.keep_hourly),
            keep_daily: other.keep_daily.or(self.keep_daily),
            keep_weekly: other.keep_weekly.or(self.keep_weekly),
            keep_monthly: other.keep_monthly.or(self.keep_monthly),
            keep_yearly: other.keep_yearly.or(self.keep_yearly),
            keep_last: other.keep_last.or(self.keep_last),
            keep_limit: other.keep_limit.or(self.keep_limit),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filtering_on_empty_policy() {
        let policy = Policy::new();

        assert_eq!(policy.applies_to(&container_a()), true);
        assert_eq!(policy.applies_to(&container_b()), true);
        assert_eq!(policy.applies_to(&container_c()), true);
    }

    #[test]
    fn test_filtering_by_containers() {
        let policy = Policy {
            allowed_containers: Some(containers(&["a", "d"])),
            ..Policy::new()
        };

        assert_eq!(policy.applies_to(&container_a()), true);
        assert_eq!(policy.applies_to(&container_b()), false);
        assert_eq!(policy.applies_to(&container_c()), false);
    }

    #[test]
    fn test_filtering_by_statuses() {
        let policy = Policy {
            allowed_statuses: Some(statuses(&[LxdContainerStatus::Aborting, LxdContainerStatus::Stopped])),
            ..Policy::new()
        };

        assert_eq!(policy.applies_to(&container_a()), false);
        assert_eq!(policy.applies_to(&container_b()), true);
        assert_eq!(policy.applies_to(&container_c()), true);
    }

    #[test]
    fn test_filtering_by_containers_and_statuses() {
        let policy = Policy {
            allowed_containers: Some(containers(&["c"])),
            allowed_statuses: Some(statuses(&[LxdContainerStatus::Aborting, LxdContainerStatus::Stopped])),
            ..Policy::new()
        };

        assert_eq!(policy.applies_to(&container_a()), false);
        assert_eq!(policy.applies_to(&container_b()), false);
        assert_eq!(policy.applies_to(&container_c()), true);
    }

    #[test]
    fn test_merging() {
        // Policy A: has all values set, serves as a base
        let policy_a = Policy {
            keep_daily: Some(10),
            keep_weekly: Some(5),
            keep_monthly: Some(2),
            keep_yearly: Some(1),
            keep_last: Some(8),
            ..Policy::new()
        };

        // Policy B: overwrites only the `keep weekly` and `keep monthly` options
        let policy_b = Policy {
            keep_weekly: Some(100),
            keep_monthly: Some(200),
            ..Policy::new()
        };

        // Policy C: overwrites only the `keep yearly` option
        let policy_c = Policy {
            keep_yearly: Some(100),
            ..Policy::new()
        };

        // Policy A + B
        let policy_ab = Policy {
            keep_weekly: Some(100), // Overwritten from policy B
            keep_monthly: Some(200), // Overwritten from policy B
            ..policy_a.clone()
        };

        // Policy A + C
        let policy_ac = Policy {
            keep_yearly: Some(100), // Overwritten from policy C
            ..policy_a.clone()
        };

        assert_eq!(policy_a.clone().merge_with(&policy_b), policy_ab);
        assert_eq!(policy_a.clone().merge_with(&policy_c), policy_ac);
    }

    /// Creates a fake container for testing purposes.
    fn container_a() -> LxdContainer {
        LxdContainer {
            name: LxdContainerName::new("a"),
            status: LxdContainerStatus::Running,
            snapshots: None,
        }
    }

    /// Creates a fake container for testing purposes.
    fn container_b() -> LxdContainer {
        LxdContainer {
            name: LxdContainerName::new("b"),
            status: LxdContainerStatus::Aborting,
            snapshots: None,
        }
    }

    /// Creates a fake container for testing purposes.
    fn container_c() -> LxdContainer {
        LxdContainer {
            name: LxdContainerName::new("c"),
            status: LxdContainerStatus::Stopped,
            snapshots: None,
        }
    }

    /// Creates a fake set of container names for testing purposes.
    fn containers(names: &[&str]) -> HashSet<LxdContainerName> {
        names
            .into_iter()
            .map(|name| LxdContainerName::new(name))
            .collect()
    }

    /// Creates a fake set of container statuses for testing purposes.
    fn statuses(statuses: &[LxdContainerStatus]) -> HashSet<LxdContainerStatus> {
        statuses
            .into_iter()
            .map(|&status| status)
            .collect()
    }
}