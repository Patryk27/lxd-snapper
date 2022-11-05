mod clients;
mod error;
mod models;

pub use self::{clients::*, error::*, models::*};

pub trait LxdClient {
    fn projects(&mut self, remote: &LxdRemoteName) -> LxdResult<Vec<LxdProject>>;

    fn instances(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
    ) -> LxdResult<Vec<LxdInstance>>;

    fn create_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()>;

    fn delete_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()>;
}

#[cfg(test)]
pub mod utils {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

    pub fn remote_name(name: impl AsRef<str>) -> LxdRemoteName {
        LxdRemoteName::new(name)
    }

    pub fn instance(name: impl AsRef<str>) -> LxdInstance {
        LxdInstance {
            name: instance_name(name),
            status: LxdInstanceStatus::Running,
            snapshots: Default::default(),
        }
    }

    pub fn instance_name(name: impl AsRef<str>) -> LxdInstanceName {
        LxdInstanceName::new(name)
    }

    pub fn project(name: impl AsRef<str>) -> LxdProject {
        LxdProject {
            name: project_name(name),
        }
    }

    pub fn project_name(name: impl AsRef<str>) -> LxdProjectName {
        LxdProjectName::new(name)
    }

    pub fn snapshot(name: impl AsRef<str>, created_at: impl AsRef<str>) -> LxdSnapshot {
        LxdSnapshot {
            name: snapshot_name(name),
            created_at: datetime(created_at),
        }
    }

    pub fn snapshot_name(name: impl AsRef<str>) -> LxdSnapshotName {
        LxdSnapshotName::new(name)
    }

    pub fn datetime(datetime: impl AsRef<str>) -> DateTime<Utc> {
        let datetime =
            NaiveDateTime::parse_from_str(datetime.as_ref(), "%Y-%m-%d %H:%M:%S").unwrap();

        Utc.from_utc_datetime(&datetime)
    }
}
