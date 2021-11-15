use crate::*;
use chrono::Utc;
use std::collections::BTreeMap;

/// An in-memory implementation of the LXD client
#[derive(Clone, Debug, Default)]
pub struct LxdFakeClient {
    projects: BTreeMap<LxdProjectName, Project>,
}

type Project = BTreeMap<LxdInstanceName, LxdInstance>;

impl LxdFakeClient {
    pub fn new(instances: Vec<LxdInstance>) -> Self {
        instances
            .into_iter()
            .fold(LxdFakeClient::default(), |mut lxd, instance| {
                lxd.create_instance(LxdProjectName::default(), instance);
                lxd
            })
    }

    pub fn from(other: &mut dyn LxdClient) -> Result<Self> {
        let mut this = LxdFakeClient::default();

        for project in other.list_projects()? {
            for instance in other.list(&project.name)? {
                this.create_instance(project.name.clone(), instance);
            }
        }

        Ok(this)
    }

    pub fn create_instance(&mut self, project: LxdProjectName, instance: LxdInstance) {
        self.projects
            .entry(project)
            .or_default()
            .insert(instance.name.clone(), instance);
    }

    fn project_mut(&mut self, project: &LxdProjectName) -> Option<&mut Project> {
        self.projects.get_mut(project)
    }

    fn instance_mut(
        &mut self,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
    ) -> Result<&mut LxdInstance> {
        self.project_mut(project)
            .and_then(|project| project.get_mut(instance))
            .ok_or_else(|| Error::NoSuchInstance {
                project: project.to_owned(),
                instance: instance.to_owned(),
            })
    }
}

impl LxdClient for LxdFakeClient {
    fn list_projects(&mut self) -> Result<Vec<LxdProject>> {
        Ok(self
            .projects
            .keys()
            .cloned()
            .map(|name| LxdProject { name })
            .collect())
    }

    fn list(&mut self, project: &LxdProjectName) -> Result<Vec<LxdInstance>> {
        if let Some(project) = self.project_mut(project) {
            Ok(project.values().cloned().collect())
        } else {
            // This is consistent with behavior of the `lxc` CLI, which also
            // returns just an empty set for non-existing projects
            Ok(Default::default())
        }
    }

    fn create_snapshot(
        &mut self,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        let instance = self.instance_mut(project_name, instance_name)?;

        if instance
            .snapshots
            .iter()
            .any(|snapshot| &snapshot.name == snapshot_name)
        {
            return Err(Error::SnapshotAlreadyExists {
                project: project_name.to_owned(),
                instance: instance_name.to_owned(),
                snapshot: snapshot_name.to_owned(),
            });
        }

        instance.snapshots.push(LxdSnapshot {
            name: snapshot_name.to_owned(),
            created_at: Utc::now(),
        });

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        project_name: &LxdProjectName,
        instance_name: &LxdInstanceName,
        snapshot_name: &LxdSnapshotName,
    ) -> Result<()> {
        let instance = self.instance_mut(project_name, instance_name)?;

        let snapshot_idx = instance
            .snapshots
            .iter()
            .position(|snapshot| &snapshot.name == snapshot_name)
            .ok_or_else(|| Error::NoSuchSnapshot {
                project: project_name.to_owned(),
                instance: instance_name.to_owned(),
                snapshot: snapshot_name.to_owned(),
            })?;

        instance.snapshots.remove(snapshot_idx);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::*;
    use pretty_assertions as pa;

    mod list_projects {
        use super::*;

        #[test]
        fn returns_names_of_existing_projects() {
            let mut lxd = LxdFakeClient::default();

            lxd.create_instance(project_name("first"), instance("foo"));
            lxd.create_instance(project_name("second"), instance("bar"));

            pa::assert_eq!(
                Ok(vec![project("first"), project("second")]),
                lxd.list_projects(),
            );
        }
    }

    mod list {
        use super::*;

        fn lxd() -> LxdFakeClient {
            let mut lxd = LxdFakeClient::default();

            lxd.create_instance(project_name("first"), instance("foo"));
            lxd.create_instance(project_name("first"), instance("bar"));
            lxd.create_instance(project_name("second"), instance("zar"));
            lxd.create_instance(project_name("third"), instance("dar"));
            lxd
        }

        mod given_an_existing_project {
            use super::*;

            #[test]
            fn returns_names_of_instances_inside_that_project() {
                let mut lxd = lxd();

                pa::assert_eq!(
                    Ok(vec![instance("bar"), instance("foo")]),
                    lxd.list(&LxdProjectName::new("first"))
                );

                pa::assert_eq!(
                    Ok(vec![instance("zar")]),
                    lxd.list(&LxdProjectName::new("second"))
                );

                pa::assert_eq!(
                    Ok(vec![instance("dar")]),
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

    mod create_snapshot {
        use super::*;

        mod given_an_existing_instance {
            use super::*;

            mod and_an_existing_snapshot {
                use super::*;

                #[test]
                fn returns_snapshot_already_exists() {
                    let mut lxd = LxdFakeClient::default();

                    lxd.create_instance(
                        LxdProjectName::default(),
                        LxdInstance {
                            name: instance_name("foo"),
                            status: LxdInstanceStatus::Running,
                            snapshots: vec![snapshot("bar", "2000-01-01 12:00:00")],
                        },
                    );

                    let actual = lxd.create_snapshot(
                        &LxdProjectName::default(),
                        &instance_name("foo"),
                        &snapshot_name("bar"),
                    );

                    let expected = Err(Error::SnapshotAlreadyExists {
                        project: LxdProjectName::default(),
                        instance: instance_name("foo"),
                        snapshot: snapshot_name("bar"),
                    });

                    pa::assert_eq!(expected, actual);
                }
            }

            mod and_a_missing_snapshot {
                use super::*;

                #[test]
                fn creates_snapshot() {
                    let mut lxd = LxdFakeClient::default();

                    lxd.create_instance(
                        LxdProjectName::default(),
                        LxdInstance {
                            name: instance_name("foo"),
                            status: LxdInstanceStatus::Running,
                            snapshots: Default::default(),
                        },
                    );

                    lxd.create_snapshot(
                        &LxdProjectName::default(),
                        &instance_name("foo"),
                        &snapshot_name("bar"),
                    )
                    .unwrap();

                    let instances = lxd.list(&LxdProjectName::default()).unwrap();

                    pa::assert_eq!(instances.len(), 1);
                    pa::assert_eq!(instances[0].snapshots.len(), 1);
                    pa::assert_eq!(instances[0].snapshots[0].name, snapshot_name("bar"));
                }
            }
        }

        mod given_a_missing_instance {
            use super::*;

            #[test]
            fn returns_no_such_instance() {
                let mut lxd = LxdFakeClient::default();

                let actual = lxd.create_snapshot(
                    &LxdProjectName::default(),
                    &instance_name("foo"),
                    &snapshot_name("bar"),
                );

                let expected = Err(Error::NoSuchInstance {
                    project: LxdProjectName::default(),
                    instance: instance_name("foo"),
                });

                pa::assert_eq!(expected, actual);
            }
        }
    }

    mod delete_snapshot {
        use super::*;

        mod given_an_existing_instance {
            use super::*;

            mod and_an_existing_snapshot {
                use super::*;

                #[test]
                fn deletes_snapshot() {
                    let mut lxd = LxdFakeClient::default();

                    lxd.create_instance(
                        LxdProjectName::default(),
                        LxdInstance {
                            name: instance_name("foo"),
                            status: LxdInstanceStatus::Running,
                            snapshots: vec![snapshot("bar", "2000-01-01 12:00:00")],
                        },
                    );

                    lxd.delete_snapshot(
                        &LxdProjectName::default(),
                        &instance_name("foo"),
                        &snapshot_name("bar"),
                    )
                    .unwrap();

                    let instances = lxd.list(&LxdProjectName::default()).unwrap();

                    pa::assert_eq!(instances.len(), 1);
                    pa::assert_eq!(instances[0].snapshots.len(), 0);
                }
            }

            mod and_a_missing_snapshot {
                use super::*;

                #[test]
                fn returns_no_such_snapshot() {
                    let mut lxd = LxdFakeClient::default();

                    lxd.create_instance(
                        LxdProjectName::default(),
                        LxdInstance {
                            name: instance_name("foo"),
                            status: LxdInstanceStatus::Running,
                            snapshots: Default::default(),
                        },
                    );

                    let actual = lxd.delete_snapshot(
                        &LxdProjectName::default(),
                        &instance_name("foo"),
                        &snapshot_name("bar"),
                    );

                    let expected = Err(Error::NoSuchSnapshot {
                        project: LxdProjectName::default(),
                        instance: instance_name("foo"),
                        snapshot: snapshot_name("bar"),
                    });

                    pa::assert_eq!(expected, actual);
                }
            }
        }

        mod given_a_missing_instance {
            use super::*;

            #[test]
            fn returns_no_such_instance() {
                let mut lxd = LxdFakeClient::default();

                let actual = lxd.delete_snapshot(
                    &LxdProjectName::default(),
                    &instance_name("foo"),
                    &snapshot_name("bar"),
                );

                let expected = Err(Error::NoSuchInstance {
                    project: LxdProjectName::default(),
                    instance: instance_name("foo"),
                });

                pa::assert_eq!(expected, actual);
            }
        }
    }
}
