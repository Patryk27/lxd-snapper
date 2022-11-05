use crate::prelude::*;
use indexmap::IndexSet;

pub fn find_snapshots_to_keep<'a>(
    policy: &Policy,
    snapshots: &'a [LxdSnapshot],
) -> IndexSet<&'a LxdSnapshotName> {
    const PATTERNS: &[(&str, &str)] = &[
        ("hourly", "%Y-%m-%d %H"),
        ("daily", "%Y-%m-%d"),
        ("weekly", "%Y-%m-%U"),
        ("monthly", "%Y-%m"),
        ("yearly", "%Y-%m"),
        ("last", "%s"),
    ];

    let mut keep_hourly = policy.keep_hourly();
    let mut keep_daily = policy.keep_daily();
    let mut keep_weekly = policy.keep_weekly();
    let mut keep_monthly = policy.keep_monthly();
    let mut keep_yearly = policy.keep_yearly();
    let mut keep_last = policy.keep_last();

    let mut alive_names = IndexSet::new();
    let mut alive_dates = IndexSet::new();

    for snapshot in snapshots {
        if let Some(limit) = policy.keep_limit() {
            if alive_names.len() >= limit {
                break;
            }
        }

        for (pattern_name, pattern_format) in PATTERNS {
            let snapshot_date = format!("{}", snapshot.created_at.format(pattern_format));

            if !alive_dates.contains(&snapshot_date) {
                let keep = match *pattern_name {
                    "hourly" => &mut keep_hourly,
                    "daily" => &mut keep_daily,
                    "weekly" => &mut keep_weekly,
                    "monthly" => &mut keep_monthly,
                    "yearly" => &mut keep_yearly,
                    "last" => &mut keep_last,
                    _ => unreachable!(),
                };

                if *keep > 0 {
                    alive_names.insert(&snapshot.name);
                    alive_dates.insert(snapshot_date);
                    *keep -= 1;
                    break;
                }
            }
        }
    }

    alive_names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::utils::*;

    fn test(policy: Policy, snapshots: Vec<LxdSnapshot>, expected: Vec<&str>) {
        let actual: Vec<_> = find_snapshots_to_keep(&policy, &snapshots)
            .into_iter()
            .cloned()
            .collect();

        let expected: Vec<_> = expected.into_iter().map(LxdSnapshotName::new).collect();

        pa::assert_eq!(expected, actual);
    }

    #[test]
    fn keep_hourly() {
        let policy = Policy {
            keep_hourly: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-10 10:30:00"),
            snapshot("snap-4", "2000-05-10 10:25:00"),
            snapshot("snap-3", "2000-05-10 08:00:00"),
            snapshot("snap-2", "2000-05-10 07:30:00"),
            snapshot("snap-1", "2000-05-10 06:25:00"),
        ];

        let expected = vec!["snap-6", "snap-5", "snap-3", "snap-2"];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_daily() {
        let policy = Policy {
            keep_daily: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-10 12:00:00"),
            snapshot("snap-4", "2000-05-09 12:00:00"),
            snapshot("snap-3", "2000-05-09 12:00:00"),
            snapshot("snap-2", "2000-05-08 12:00:00"),
            snapshot("snap-1", "2000-05-07 12:00:00"),
        ];

        let expected = vec!["snap-6", "snap-4", "snap-2", "snap-1"];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_hourly_and_daily() {
        let policy = Policy {
            keep_hourly: Some(4),
            keep_daily: Some(2),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-9", "2000-05-10 12:00:00"),
            snapshot("snap-8", "2000-05-10 10:00:00"),
            snapshot("snap-7", "2000-05-10 08:00:00"),
            snapshot("snap-6", "2000-05-09 12:00:00"),
            snapshot("snap-5", "2000-05-09 10:00:00"),
            snapshot("snap-4", "2000-05-09 08:00:00"),
            snapshot("snap-3", "2000-05-08 12:00:00"),
            snapshot("snap-2", "2000-05-08 10:00:00"),
            snapshot("snap-1", "2000-05-08 08:00:00"),
        ];

        let expected = vec![
            "snap-9", // via keep-hourly
            "snap-8", // via keep-hourly
            "snap-7", // via keep-hourly
            "snap-6", // via keep-hourly
            "snap-5", // via keep-daily
            "snap-3", // via keep-daily
        ];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_weekly() {
        let policy = Policy {
            keep_weekly: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 00:00:00"),
            snapshot("snap-5", "2000-05-09 00:00:00"),
            snapshot("snap-4", "2000-05-02 00:00:00"),
            snapshot("snap-3", "2000-05-01 00:00:00"),
            snapshot("snap-2", "2000-04-25 00:00:00"),
            snapshot("snap-1", "2000-04-10 00:00:00"),
        ];

        let expected = vec!["snap-6", "snap-4", "snap-2", "snap-1"];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_daily_and_weekly() {
        let policy = Policy {
            keep_daily: Some(4),
            keep_weekly: Some(2),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-9", "2000-05-10 00:00:00"),
            snapshot("snap-8", "2000-05-09 00:00:00"),
            snapshot("snap-7", "2000-05-08 00:00:00"),
            snapshot("snap-6", "2000-04-07 00:00:00"),
            snapshot("snap-5", "2000-04-06 00:00:00"),
            snapshot("snap-4", "2000-04-05 00:00:00"),
            snapshot("snap-3", "2000-03-05 00:00:00"),
            snapshot("snap-2", "2000-03-05 00:00:00"),
            snapshot("snap-1", "2000-03-05 00:00:00"),
        ];

        let expected = vec![
            "snap-9", // via keep-daily
            "snap-8", // via keep-daily
            "snap-7", // via keep-daily
            "snap-6", // via keep-daily
            "snap-5", // via keep-weekly
            "snap-3", // via keep-weekly
        ];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_monthly() {
        let policy = Policy {
            keep_monthly: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 00:00:00"),
            snapshot("snap-5", "2000-05-02 00:00:00"),
            snapshot("snap-4", "2000-04-15 00:00:00"),
            snapshot("snap-3", "2000-04-15 00:00:00"),
            snapshot("snap-2", "2000-02-25 00:00:00"),
            snapshot("snap-1", "2000-01-15 00:00:00"),
        ];

        let expected = vec!["snap-6", "snap-4", "snap-2", "snap-1"];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_yearly() {
        let policy = Policy {
            keep_yearly: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-06-10 00:00:00"),
            snapshot("snap-5", "2000-06-10 00:00:00"),
            snapshot("snap-4", "1999-06-10 00:00:00"),
            snapshot("snap-3", "1999-06-10 00:00:00"),
            snapshot("snap-2", "1998-06-10 00:00:00"),
            snapshot("snap-1", "1995-06-10 00:00:00"),
        ];

        let expected = vec!["snap-6", "snap-4", "snap-2", "snap-1"];

        test(policy, snapshots, expected);
    }

    #[test]
    fn keep_last() {
        let policy = Policy {
            keep_last: Some(4),
            ..Default::default()
        };

        let snapshots = vec![
            snapshot("snap-6", "2000-05-10 12:00:00"),
            snapshot("snap-5", "2000-05-09 12:00:00"),
            snapshot("snap-4", "2000-05-08 12:00:00"),
            snapshot("snap-3", "2000-05-07 12:00:00"),
            snapshot("snap-2", "2000-05-06 12:00:00"),
            snapshot("snap-1", "2000-05-05 12:00:00"),
        ];

        let expected = vec!["snap-6", "snap-5", "snap-4", "snap-3"];

        test(policy, snapshots, expected);
    }
}
