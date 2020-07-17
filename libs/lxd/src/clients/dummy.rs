use crate::*;
use chrono::Utc;
use std::collections::BTreeMap;

/// An in-memory implementation of the LXD client
#[derive(Default)]
pub struct LxdDummyClient {
    projects: BTreeMap<LxdProjectName, Project>,
}

type Project = BTreeMap<LxdContainerName, LxdContainer>;

impl LxdDummyClient {
    pub fn new(containers: Vec<LxdContainer>) -> Self {
        containers
            .into_iter()
            .fold(LxdDummyClient::default(), |mut lxd, container| {
                lxd.create_container(LxdProjectName::default(), container);
                lxd
            })
    }

    pub fn from_other(other: &mut impl LxdClient) -> Result<Self> {
        let mut this = LxdDummyClient::default();

        for project in other.list_projects()? {
            for container in other.list(&project.name)? {
                this.create_container(project.name.clone(), container);
            }
        }

        Ok(this)
    }

    pub fn create_container(&mut self, project: LxdProjectName, container: LxdContainer) {
        self.projects
            .entry(project)
            .or_default()
            .insert(container.name.clone(), container);
    }

    fn project_mut(&mut self, project: &LxdProjectName) -> Option<&mut Project> {
        self.projects.get_mut(project)
    }

    fn container_mut(
        &mut self,
        project: &LxdProjectName,
        container: &LxdContainerName,
    ) -> Result<&mut LxdContainer> {
        let instance: Option<_> = try { self.project_mut(project)?.get_mut(container)? };

        instance.ok_or_else(|| Error::NoSuchContainer {
            project: project.to_owned(),
            container: container.to_owned(),
        })
    }
}

impl LxdClient for LxdDummyClient {
    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdContainer>> {
        if let Some(project) = self.project_mut(project) {
            Ok(project.values().cloned().collect())
        } else {
            // This is consistent with behavior of the `lxc` CLI, which also returns just an
            // empty set for non-existing projects
            Ok(Default::default())
        }
    }

    fn list_projects(&mut self) -> Result<Vec<LxdProject>> {
        Ok(self
            .projects
            .keys()
            .cloned()
            .map(|name| LxdProject { name })
            .collect())
    }

    fn create_snapshot(
        &mut self,
        project_name: &LxdProjectName,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        let container = self.container_mut(project_name, container_name)?;

        if container
            .snapshots
            .iter()
            .any(|snapshot| &snapshot.name == snapshot_name)
        {
            return Err(Error::SnapshotAlreadyExists {
                project: project_name.to_owned(),
                container: container_name.to_owned(),
                snapshot: snapshot_name.to_owned(),
            });
        }

        container.snapshots.push(LxdSnapshot {
            name: snapshot_name.to_owned(),
            created_at: Utc::now(),
        });

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        project_name: &LxdProjectName,
        container_name: &LxdContainerName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result {
        let container = self.container_mut(project_name, container_name)?;

        let snapshot_idx = container
            .snapshots
            .iter()
            .position(|snapshot| &snapshot.name == snapshot_name)
            .ok_or_else(|| Error::NoSuchSnapshot {
                project: project_name.to_owned(),
                container: container_name.to_owned(),
                snapshot: snapshot_name.to_owned(),
            })?;

        container.snapshots.remove(snapshot_idx);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::*;
    use pretty_assertions as pa;

    mod list {
        use super::*;

        fn lxd() -> LxdDummyClient {
            let mut lxd = LxdDummyClient::default();

            lxd.create_container(project_name("first"), container("foo"));
            lxd.create_container(project_name("first"), container("bar"));
            lxd.create_container(project_name("second"), container("zar"));
            lxd.create_container(project_name("third"), container("dar"));

            lxd
        }

        mod given_an_existing_project {
            use super::*;

            #[test]
            fn returns_names_of_containers_inside_that_project() {
                let mut lxd = lxd();

                pa::assert_eq!(
                    Ok(vec![container("bar"), container("foo")]),
                    lxd.list(&LxdProjectName::new("first"))
                );

                pa::assert_eq!(
                    Ok(vec![container("zar")]),
                    lxd.list(&LxdProjectName::new("second"))
                );

                pa::assert_eq!(
                    Ok(vec![container("dar")]),
                    lxd.list(&LxdProjectName::new("third"))
                );
            }
        }

        mod given_a_missing_project {
            use super::*;

            #[test]
            fn returns_nothing() {
                let mut lxd = lxd();

                let actual = lxd.list(&LxdProjectName::default());
                let expected = Ok(Vec::new());

                pa::assert_eq!(expected, actual);
            }
        }
    }

    mod list_projects {
        use super::*;

        #[test]
        fn returns_names_of_existing_projects() {
            let mut lxd = LxdDummyClient::default();

            lxd.create_container(project_name("first"), container("foo"));
            lxd.create_container(project_name("second"), container("bar"));

            pa::assert_eq!(
                Ok(vec![project("first"), project("second")]),
                lxd.list_projects(),
            );
        }
    }

    mod create_snapshot {
        use super::*;

        mod given_an_existing_container {
            use super::*;

            mod and_an_existing_snapshot {
                use super::*;

                #[test]
                fn returns_snapshot_already_exists() {
                    let mut lxd = LxdDummyClient::default();

                    lxd.create_container(
                        LxdProjectName::default(),
                        LxdContainer {
                            name: container_name("foo"),
                            status: LxdContainerStatus::Running,
                            snapshots: vec![snapshot("bar", "2000-01-01 12:00:00")],
                        },
                    );

                    let actual = lxd.create_snapshot(
                        &LxdProjectName::default(),
                        &container_name("foo"),
                        &snapshot_name("bar"),
                    );

                    let expected = Err(Error::SnapshotAlreadyExists {
                        project: LxdProjectName::default(),
                        container: container_name("foo"),
                        snapshot: snapshot_name("bar"),
                    });

                    pa::assert_eq!(expected, actual);
                }
            }

            mod and_a_missing_snapshot {
                use super::*;

                #[test]
                fn creates_snapshot() {
                    let mut lxd = LxdDummyClient::default();

                    lxd.create_container(
                        LxdProjectName::default(),
                        LxdContainer {
                            name: container_name("foo"),
                            status: LxdContainerStatus::Running,
                            snapshots: Default::default(),
                        },
                    );

                    lxd.create_snapshot(
                        &LxdProjectName::default(),
                        &container_name("foo"),
                        &snapshot_name("bar"),
                    )
                    .unwrap();

                    let containers = lxd.list(&LxdProjectName::default()).unwrap();

                    pa::assert_eq!(containers.len(), 1);
                    pa::assert_eq!(containers[0].snapshots.len(), 1);
                    pa::assert_eq!(containers[0].snapshots[0].name, snapshot_name("bar"));
                }
            }
        }

        mod given_a_missing_container {
            use super::*;

            #[test]
            fn returns_no_such_container() {
                let mut lxd = LxdDummyClient::default();

                let actual = lxd.create_snapshot(
                    &LxdProjectName::default(),
                    &container_name("foo"),
                    &snapshot_name("bar"),
                );

                let expected = Err(Error::NoSuchContainer {
                    project: LxdProjectName::default(),
                    container: container_name("foo"),
                });

                pa::assert_eq!(expected, actual);
            }
        }
    }

    mod delete_snapshot {
        use super::*;

        mod given_an_existing_container {
            use super::*;

            mod and_an_existing_snapshot {
                use super::*;

                #[test]
                fn deletes_snapshot() {
                    let mut lxd = LxdDummyClient::default();

                    lxd.create_container(
                        LxdProjectName::default(),
                        LxdContainer {
                            name: container_name("foo"),
                            status: LxdContainerStatus::Running,
                            snapshots: vec![snapshot("bar", "2000-01-01 12:00:00")],
                        },
                    );

                    lxd.delete_snapshot(
                        &LxdProjectName::default(),
                        &container_name("foo"),
                        &snapshot_name("bar"),
                    )
                    .unwrap();

                    let containers = lxd.list(&LxdProjectName::default()).unwrap();

                    pa::assert_eq!(containers.len(), 1);
                    pa::assert_eq!(containers[0].snapshots.len(), 0);
                }
            }

            mod and_a_missing_snapshot {
                use super::*;

                #[test]
                fn returns_no_such_snapshot() {
                    let mut lxd = LxdDummyClient::default();

                    lxd.create_container(
                        LxdProjectName::default(),
                        LxdContainer {
                            name: container_name("foo"),
                            status: LxdContainerStatus::Running,
                            snapshots: Default::default(),
                        },
                    );

                    let actual = lxd.delete_snapshot(
                        &LxdProjectName::default(),
                        &container_name("foo"),
                        &snapshot_name("bar"),
                    );

                    let expected = Err(Error::NoSuchSnapshot {
                        project: LxdProjectName::default(),
                        container: container_name("foo"),
                        snapshot: snapshot_name("bar"),
                    });

                    pa::assert_eq!(expected, actual);
                }
            }
        }

        mod given_a_missing_container {
            use super::*;

            #[test]
            fn returns_no_such_container() {
                let mut lxd = LxdDummyClient::default();

                let actual = lxd.delete_snapshot(
                    &LxdProjectName::default(),
                    &container_name("foo"),
                    &snapshot_name("bar"),
                );

                let expected = Err(Error::NoSuchContainer {
                    project: LxdProjectName::default(),
                    container: container_name("foo"),
                });

                pa::assert_eq!(expected, actual);
            }
        }
    }
}
