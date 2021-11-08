use crate::prelude::*;
use lib_lxd::*;

pub fn find_snapshots(config: &Config, instances: &LxdInstance) -> Vec<LxdSnapshot> {
    let mut snapshots: Vec<_> = instances
        .snapshots
        .iter()
        .filter(|snapshot| config.matches_snapshot_name(&snapshot.name))
        .cloned()
        .collect();

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    snapshots
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_lxd::test_utils::*;

    const CONFIG: &str = indoc!(
        r#"
        snapshot-name-prefix: auto-
      
        policies:
          main:
            keep-last: 15
        "#
    );

    #[test]
    fn returns_only_snapshots_matching_format() {
        let config = Config::from_code(CONFIG);

        let instance = LxdInstance {
            name: instance_name("test"),
            status: LxdInstanceStatus::Running,
            snapshots: vec![
                snapshot("snap-0", "2000-01-01 12:00:00"),
                snapshot("auto", "2000-01-01 13:00:00"),
                snapshot("auto-", "2000-01-01 14:00:00"),
                snapshot("auto-20000101", "2000-01-01 15:00:00"),
                snapshot("auto-20000101-160000", "2000-01-01 16:00:00"),
                snapshot("snap-1", "2000-01-01 17:00:00"),
            ],
        };

        let actual = find_snapshots(&config, &instance);

        let expected = vec![
            snapshot("auto-20000101-160000", "2000-01-01 16:00:00"),
            snapshot("auto-20000101", "2000-01-01 15:00:00"),
            snapshot("auto-", "2000-01-01 14:00:00"),
        ];

        pa::assert_eq!(expected, actual);
    }

    #[test]
    fn returns_snapshots_sorted_by_creation_date_descending() {
        let config = Config::from_code(CONFIG);

        let instance = LxdInstance {
            name: instance_name("test"),
            status: LxdInstanceStatus::Running,
            snapshots: vec![
                snapshot("auto-1", "2012-08-24 12:34:56"),
                snapshot("auto-2", "2012-08-24 12:36:56"),
                snapshot("auto-4", "2010-08-24 12:34:56"),
                snapshot("auto-0", "2012-08-24 12:35:56"),
            ],
        };

        let actual = find_snapshots(&config, &instance);

        let expected = vec![
            snapshot("auto-2", "2012-08-24 12:36:56"),
            snapshot("auto-0", "2012-08-24 12:35:56"),
            snapshot("auto-1", "2012-08-24 12:34:56"),
            snapshot("auto-4", "2010-08-24 12:34:56"),
        ];

        pa::assert_eq!(expected, actual);
    }
}
