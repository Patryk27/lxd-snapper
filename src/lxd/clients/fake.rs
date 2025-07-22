use crate::lxd::*;
use chrono::Utc;
use itertools::Itertools;
use std::collections::BTreeMap;

#[cfg(test)]
use std::fmt;

#[cfg(test)]
use std::collections::HashSet;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LxdFakeClient {
    instances: BTreeMap<LxdInstanceId, LxdInstance>,

    #[cfg(test)]
    errors: HashSet<LxdFakeError<'static>>,
}

impl LxdFakeClient {
    pub fn clone_from<'a>(
        other: &mut dyn LxdClient,
        remotes: impl IntoIterator<Item = &'a LxdRemoteName>,
    ) -> LxdResult<Self> {
        let mut this = Self::default();

        for remote in remotes {
            for project in other.projects(remote)? {
                for instance in other.instances(remote, &project.name)? {
                    this.instances.insert(
                        LxdInstanceId {
                            remote: remote.to_owned(),
                            project: project.name.clone(),
                            instance: instance.name.clone(),
                        },
                        instance,
                    );
                }
            }
        }

        Ok(this)
    }

    #[cfg(test)]
    pub fn add(&mut self, instance: LxdFakeInstance<'_>) {
        self.instances.insert(
            LxdInstanceId {
                remote: LxdRemoteName::new(instance.remote),
                project: LxdProjectName::new(instance.project),
                instance: LxdInstanceName::new(instance.name),
            },
            LxdInstance {
                name: LxdInstanceName::new(instance.name),
                status: instance.status,
                snapshots: instance.snapshots,
            },
        );
    }

    #[cfg(test)]
    pub fn inject_error(&mut self, error: LxdFakeError<'static>) {
        self.errors.insert(error);
    }

    fn get_mut(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
    ) -> LxdResult<&mut LxdInstance> {
        let id = LxdInstanceId {
            remote: remote.to_owned(),
            project: project.to_owned(),
            instance: instance.to_owned(),
        };

        self.instances
            .get_mut(&id)
            .ok_or_else(|| LxdError::NoSuchInstance {
                remote: remote.to_owned(),
                project: project.to_owned(),
                instance: instance.to_owned(),
            })
    }
}

impl LxdClient for LxdFakeClient {
    fn projects(&mut self, remote: &LxdRemoteName) -> LxdResult<Vec<LxdProject>> {
        let projects = self
            .instances
            .keys()
            .filter(|id| &id.remote == remote)
            .map(|id| &id.project)
            .unique()
            .cloned()
            .map(|name| LxdProject { name })
            .collect();

        Ok(projects)
    }

    fn instances(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
    ) -> LxdResult<Vec<LxdInstance>> {
        let instances = self
            .instances
            .iter()
            .filter(|(id, _)| &id.remote == remote && &id.project == project)
            .map(|(_, instance)| instance.clone())
            .collect();

        Ok(instances)
    }

    fn create_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()> {
        #[cfg(test)]
        if self.errors.contains(&LxdFakeError::OnCreateSnapshot {
            remote: remote.as_str(),
            project: project.as_str(),
            instance: instance.as_str(),
            snapshot: snapshot.as_str(),
        }) {
            return Err(LxdError::InjectedError);
        }

        let instance_obj = self.get_mut(remote, project, instance)?;

        if instance_obj
            .snapshots
            .iter()
            .any(|snapshot_obj| &snapshot_obj.name == snapshot)
        {
            return Err(LxdError::SnapshotAlreadyExists {
                remote: remote.to_owned(),
                project: project.to_owned(),
                instance: instance.to_owned(),
                snapshot: snapshot.to_owned(),
            });
        }

        instance_obj.snapshots.push(LxdSnapshot {
            name: snapshot.to_owned(),
            created_at: Utc::now(),
        });

        Ok(())
    }

    fn delete_snapshot(
        &mut self,
        remote: &LxdRemoteName,
        project: &LxdProjectName,
        instance: &LxdInstanceName,
        snapshot: &LxdSnapshotName,
    ) -> LxdResult<()> {
        #[cfg(test)]
        if self.errors.contains(&LxdFakeError::OnDeleteSnapshot {
            remote: remote.as_str(),
            project: project.as_str(),
            instance: instance.as_str(),
            snapshot: snapshot.as_str(),
        }) {
            return Err(LxdError::InjectedError);
        }

        let instance_obj = self.get_mut(remote, project, instance)?;

        let snapshot_idx = instance_obj
            .snapshots
            .iter()
            .position(|snapshot_obj| &snapshot_obj.name == snapshot)
            .ok_or_else(|| LxdError::NoSuchSnapshot {
                remote: remote.to_owned(),
                project: project.to_owned(),
                instance: instance.to_owned(),
                snapshot: snapshot.to_owned(),
            })?;

        instance_obj.snapshots.remove(snapshot_idx);

        Ok(())
    }
}

#[cfg(test)]
impl fmt::Display for LxdFakeClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (id, instance)) in self.instances.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }

            writeln!(
                f,
                "{}:{}/{} ({:?})",
                id.remote, id.project, id.instance, instance.status
            )?;

            for snapshot in &instance.snapshots {
                writeln!(f, "-> {}", snapshot.name)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LxdInstanceId {
    remote: LxdRemoteName,
    project: LxdProjectName,
    instance: LxdInstanceName,
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct LxdFakeInstance<'a> {
    pub remote: &'a str,
    pub project: &'a str,
    pub name: &'a str,
    pub status: LxdInstanceStatus,
    pub snapshots: Vec<LxdSnapshot>,
}

#[cfg(test)]
impl Default for LxdFakeInstance<'static> {
    fn default() -> Self {
        Self {
            remote: "local",
            project: "default",
            name: "",
            status: LxdInstanceStatus::Running,
            snapshots: Default::default(),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LxdFakeError<'a> {
    OnCreateSnapshot {
        remote: &'a str,
        project: &'a str,
        instance: &'a str,
        snapshot: &'a str,
    },

    OnDeleteSnapshot {
        remote: &'a str,
        project: &'a str,
        instance: &'a str,
        snapshot: &'a str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::utils::*;
    use pretty_assertions as pa;

    fn client() -> LxdFakeClient {
        let mut client = LxdFakeClient::default();

        client.add(LxdFakeInstance {
            project: "app",
            name: "checker",
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            project: "db",
            name: "elastic",
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            project: "db",
            name: "mysql",
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            remote: "remote-a",
            project: "db",
            name: "mysql",
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            remote: "remote-b",
            project: "db",
            name: "elastic",
            ..Default::default()
        });

        client.add(LxdFakeInstance {
            remote: "remote-b",
            project: "log",
            name: "grafana",
            ..Default::default()
        });

        client
    }

    #[test]
    fn clone_from() {
        let mut client1 = client();

        let client2 = LxdFakeClient::clone_from(
            &mut client1,
            &[
                remote_name("local"),
                remote_name("remote-a"),
                remote_name("remote-b"),
                remote_name("remote-c"),
            ],
        )
        .unwrap();

        pa::assert_eq!(client2, client1);
    }

    mod projects {
        use super::*;

        #[test]
        fn ok() {
            let mut client = client();

            pa::assert_eq!(
                Ok(vec![project("app"), project("db")]),
                client.projects(&remote_name("local")),
            );

            pa::assert_eq!(
                Ok(vec![project("db")]),
                client.projects(&remote_name("remote-a"))
            );

            pa::assert_eq!(
                Ok(vec![project("db"), project("log")]),
                client.projects(&remote_name("remote-b")),
            );

            pa::assert_eq!(
                Ok(vec![project("db"), project("log")]),
                client.projects(&remote_name("remote-b")),
            );
        }

        #[test]
        fn given_unknown_remote() {
            let mut client = client();

            pa::assert_eq!(Ok(vec![]), client.projects(&remote_name("unknown")));
        }
    }

    mod instances {
        use super::*;

        #[test]
        fn ok() {
            let mut client = client();

            pa::assert_eq!(
                Ok(vec![instance("checker")]),
                client.instances(&remote_name("local"), &project_name("app"))
            );

            pa::assert_eq!(
                Ok(vec![instance("elastic"), instance("mysql")]),
                client.instances(&remote_name("local"), &project_name("db"))
            );

            pa::assert_eq!(
                Ok(vec![instance("mysql")]),
                client.instances(&remote_name("remote-a"), &project_name("db"))
            );

            pa::assert_eq!(
                Ok(vec![instance("elastic")]),
                client.instances(&remote_name("remote-b"), &project_name("db"))
            );

            pa::assert_eq!(
                Ok(vec![instance("grafana")]),
                client.instances(&remote_name("remote-b"), &project_name("log"))
            );
        }

        #[test]
        fn given_unknown_remote() {
            let mut client = client();

            pa::assert_eq!(
                Ok(vec![]),
                client.instances(&remote_name("unknown"), &project_name("app"))
            );
        }

        #[test]
        fn given_unknown_project() {
            let mut client = client();

            pa::assert_eq!(
                Ok(vec![]),
                client.instances(&remote_name("local"), &project_name("unknown"))
            );
        }
    }

    mod create_snapshot {
        use super::*;

        #[test]
        fn ok() {
            let mut client = client();

            client
                .create_snapshot(
                    &remote_name("local"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap();

            for (instance_id, instance) in &client.instances {
                if instance_id.remote == remote_name("local")
                    && instance_id.project.as_str() == "db"
                    && instance_id.instance.as_str() == "elastic"
                {
                    assert_eq!(1, instance.snapshots.len());
                    assert_eq!("auto-1", instance.snapshots[0].name.as_str());
                } else {
                    assert_eq!(0, instance.snapshots.len());
                }
            }
        }

        #[test]
        fn given_existing_snapshot() {
            let mut client = client();

            client
                .create_snapshot(
                    &remote_name("local"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap();

            let actual = client
                .create_snapshot(
                    &remote_name("local"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::SnapshotAlreadyExists {
                remote: remote_name("local"),
                project: project_name("db"),
                instance: instance_name("elastic"),
                snapshot: snapshot_name("auto-1"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_remote() {
            let actual = client()
                .create_snapshot(
                    &remote_name("unknown"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("unknown"),
                project: project_name("db"),
                instance: instance_name("elastic"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_project() {
            let actual = client()
                .create_snapshot(
                    &remote_name("local"),
                    &project_name("unknown"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("local"),
                project: project_name("unknown"),
                instance: instance_name("elastic"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_instance() {
            let actual = client()
                .create_snapshot(
                    &remote_name("local"),
                    &project_name("app"),
                    &instance_name("unknown"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("local"),
                project: project_name("app"),
                instance: instance_name("unknown"),
            };

            pa::assert_eq!(expected, actual);
        }
    }

    mod delete_snapshot {
        use super::*;

        #[test]
        fn ok() {
            let mut client = client();

            for i in 1..=3 {
                client
                    .create_snapshot(
                        &remote_name("local"),
                        &project_name("db"),
                        &instance_name("elastic"),
                        &snapshot_name(format!("auto-{}", i)),
                    )
                    .unwrap();
            }

            client
                .delete_snapshot(
                    &remote_name("local"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-2"),
                )
                .unwrap();

            for (instance_id, instance) in &client.instances {
                if instance_id.remote == remote_name("local")
                    && instance_id.project.as_str() == "db"
                    && instance_id.instance.as_str() == "elastic"
                {
                    assert_eq!(2, instance.snapshots.len());
                    assert_eq!("auto-1", instance.snapshots[0].name.as_str());
                    assert_eq!("auto-3", instance.snapshots[1].name.as_str());
                } else {
                    assert_eq!(0, instance.snapshots.len());
                }
            }
        }

        #[test]
        fn given_unknown_remote() {
            let actual = client()
                .delete_snapshot(
                    &remote_name("unknown"),
                    &project_name("db"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("unknown"),
                project: project_name("db"),
                instance: instance_name("elastic"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_project() {
            let actual = client()
                .delete_snapshot(
                    &remote_name("local"),
                    &project_name("unknown"),
                    &instance_name("elastic"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("local"),
                project: project_name("unknown"),
                instance: instance_name("elastic"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_instance() {
            let actual = client()
                .delete_snapshot(
                    &remote_name("local"),
                    &project_name("app"),
                    &instance_name("unknown"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchInstance {
                remote: remote_name("local"),
                project: project_name("app"),
                instance: instance_name("unknown"),
            };

            pa::assert_eq!(expected, actual);
        }

        #[test]
        fn given_unknown_snapshot() {
            let actual = client()
                .delete_snapshot(
                    &remote_name("local"),
                    &project_name("app"),
                    &instance_name("checker"),
                    &snapshot_name("auto-1"),
                )
                .unwrap_err();

            let expected = LxdError::NoSuchSnapshot {
                remote: remote_name("local"),
                project: project_name("app"),
                instance: instance_name("checker"),
                snapshot: snapshot_name("auto-1"),
            };

            pa::assert_eq!(expected, actual);
        }
    }
}
