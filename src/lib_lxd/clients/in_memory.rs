use std::collections::HashMap;

use chrono::Local;

use crate::*;

pub struct LxdInMemoryClient {
    containers: HashMap<LxdContainerName, LxdContainer>,
}

impl LxdInMemoryClient {
    pub fn new(containers: Vec<LxdContainer>) -> Self {
        let containers = containers
            .into_iter()
            .map(|container| {
                (container.name.clone(), container)
            })
            .collect();

        Self { containers }
    }

    fn get_container_mut(&mut self, name: &LxdContainerName) -> Result<&mut LxdContainer> {
        self.containers
            .get_mut(name)
            .ok_or_else(|| Error::ClientError("No such container exists".into()))
    }
}

impl LxdClient for LxdInMemoryClient {
    fn check_connection(&mut self) -> Result {
        Ok(())
    }

    fn create_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        // Load the container
        let container = self.get_container_mut(container_name)?;
        let snapshots = container.snapshots.get_or_insert_with(Vec::new);

        // Create the snapshot
        let snapshot = LxdSnapshot {
            name: snapshot_name.to_owned(),
            created_at: Local::now(),
        };

        // Make sure that no such snapshot already exists
        if snapshots.iter().any(|snapshot| &snapshot.name == snapshot_name) {
            return Err(
                Error::ClientError("Snapshot already exists".into())
            );
        }

        snapshots.push(snapshot);

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        // Load container
        let container = self.get_container_mut(container_name)?;

        // Make sure specified snapshot exists
        let snapshots = container.snapshots
            .as_mut()
            .ok_or_else(|| {
                Error::ClientError("No such snapshot exists".into())
            })?;

        if !snapshots.iter().any(|snapshot| &snapshot.name == snapshot_name) {
            return Err(
                Error::ClientError("No such snapshot exists".into())
            );
        }

        // If so, delete it
        let snapshots = container.snapshots
            .take()
            .unwrap()
            .into_iter()
            .filter(|snapshot| &snapshot.name != snapshot_name)
            .collect();

        container.snapshots = Some(snapshots);

        Ok(())
    }

    fn list(&mut self) -> Result<Vec<LxdContainer>> {
        let containers = self.containers
            .values()
            .map(Clone::clone)
            .collect();

        Ok(containers)
    }
}

#[cfg(test)]
mod tests {
    use crate::LxdContainerStatus;

    use super::*;

    #[test]
    fn test_creating_snapshots() {
        let mut lxd = LxdInMemoryClient::new(vec![
            LxdContainer {
                name: LxdContainerName::new("hello-world"),
                status: LxdContainerStatus::Running,
                snapshots: None,
            },
        ]);

        // Error: No such container exists
        assert!(lxd.create_snapshot(
            &LxdContainerName::new("hello-world--invalid"),
            &LxdSnapshotName::new("snap0"),
        ).is_err());

        // Ok: Snapshot created
        assert!(lxd.create_snapshot(
            &LxdContainerName::new("hello-world"),
            &LxdSnapshotName::new("snap0"),
        ).is_ok());

        // Err: Snapshot already exists
        assert!(lxd.create_snapshot(
            &LxdContainerName::new("hello-world"),
            &LxdSnapshotName::new("snap0"),
        ).is_err());

        // Make sure snapshot was actually created
        let containers = lxd.list().unwrap();
        let container = &containers[0];
        let container_snapshots = container.snapshots.as_ref().unwrap();

        assert_eq!(container_snapshots.len(), 1);
    }

    #[test]
    fn test_deleting_snapshots() {
        let mut lxd = LxdInMemoryClient::new(vec![
            LxdContainer {
                name: LxdContainerName::new("hello-world"),
                status: LxdContainerStatus::Running,

                snapshots: Some(vec![
                    LxdSnapshot {
                        name: LxdSnapshotName::new("snap0"),
                        created_at: Local::now(),
                    },
                ]),
            },
        ]);

        // Error: No such container exists
        assert!(lxd.delete_snapshot(
            &LxdContainerName::new("hello-world--invalid"),
            &LxdSnapshotName::new("snap100"),
        ).is_err());

        // Error: No such snapshot exists
        assert!(lxd.delete_snapshot(
            &LxdContainerName::new("hello-world"),
            &LxdSnapshotName::new("snap100"),
        ).is_err());

        // Ok: Snapshot deleted
        assert!(lxd.delete_snapshot(
            &LxdContainerName::new("hello-world"),
            &LxdSnapshotName::new("snap0"),
        ).is_ok());

        // Err: Snapshot deleted
        assert!(lxd.delete_snapshot(
            &LxdContainerName::new("hello-world"),
            &LxdSnapshotName::new("snap0"),
        ).is_err());

        // Make sure snapshot was actually deleted
        let containers = lxd.list().unwrap();
        let container = &containers[0];
        let container_snapshots = container.snapshots.as_ref().unwrap();

        assert_eq!(container_snapshots.len(), 0);
    }
}