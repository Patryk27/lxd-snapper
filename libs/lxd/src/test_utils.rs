use crate::*;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

pub fn instance(name: &str) -> LxdInstance {
    LxdInstance {
        name: instance_name(name),
        status: LxdInstanceStatus::Running,
        snapshots: Default::default(),
    }
}

pub fn instance_name(name: &str) -> LxdInstanceName {
    LxdInstanceName::new(name)
}

pub fn project(name: &str) -> LxdProject {
    LxdProject {
        name: project_name(name),
    }
}

pub fn project_name(name: &str) -> LxdProjectName {
    LxdProjectName::new(name)
}

pub fn snapshot(name: &str, created_at: &str) -> LxdSnapshot {
    LxdSnapshot {
        name: snapshot_name(name),
        created_at: datetime(created_at),
    }
}

pub fn snapshot_name(name: &str) -> LxdSnapshotName {
    LxdSnapshotName::new(name)
}

pub fn datetime(datetime: &str) -> DateTime<Utc> {
    let datetime = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S").unwrap();
    Utc.from_utc_datetime(&datetime)
}
