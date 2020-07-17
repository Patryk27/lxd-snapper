use crate::config::Config;
use anyhow::Result;
use lib_lxd::*;
use std::io::Write;

crate fn nuke(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    writeln!(stdout, "Nuking containers:")?;

    for project in lxd.list_projects()? {
        for container in lxd.list(&project.name)? {
            if config.policy(&project, &container).is_none() {
                continue;
            }

            writeln!(stdout)?;
            writeln!(stdout, "- {}/{}", project.name, container.name)?;

            for snapshot in container.snapshots {
                writeln!(stdout, "-> deleting snapshot: {}", snapshot.name)?;
                lxd.delete_snapshot(&project.name, &container.name, &snapshot.name)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_out;
    use indoc::indoc;
    use lib_lxd::test_utils::*;
    use pretty_assertions as pa;

    fn containers() -> Vec<LxdContainer> {
        vec![
            LxdContainer {
                name: container_name("container-a"),
                status: LxdContainerStatus::Running,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
            //
            LxdContainer {
                name: container_name("container-b"),
                status: LxdContainerStatus::Running,
                snapshots: vec![
                    snapshot("snapshot-1", "2000-01-01 12:00:00"),
                    snapshot("snapshot-2", "2000-01-01 13:00:00"),
                ],
            },
            //
            LxdContainer {
                name: container_name("container-c"),
                status: LxdContainerStatus::Stopping,
                snapshots: Default::default(),
            },
            //
            LxdContainer {
                name: container_name("container-d"),
                status: LxdContainerStatus::Stopped,
                snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
            },
        ]
    }

    mod given_empty_policy {
        use super::*;

        #[test]
        fn deletes_no_snapshots() {
            let mut stdout = Vec::new();
            let config = Config::default();
            let mut lxd = LxdDummyClient::new(containers());

            nuke(&mut stdout, &config, &mut lxd).unwrap();

            assert_out!(
                r#"
                Nuking containers:
                "#,
                stdout
            );

            let actual_containers = lxd.list(&LxdProjectName::default()).unwrap();
            let expected_containers = containers();

            pa::assert_eq!(expected_containers, actual_containers);
        }
    }

    mod given_some_policy {
        use super::*;

        const POLICY: &str = indoc!(
            r#"
            policies:
              main:
                included-statuses: ['Running']
            "#
        );

        #[test]
        fn deletes_snapshots_only_for_containers_matching_that_policy() {
            let mut stdout = Vec::new();
            let config = Config::from_code(POLICY);
            let mut lxd = LxdDummyClient::new(containers());

            nuke(&mut stdout, &config, &mut lxd).unwrap();

            assert_out!(
                r#"
                Nuking containers:
                
                - default/container-a
                -> deleting snapshot: snapshot-1
                
                - default/container-b
                -> deleting snapshot: snapshot-1
                -> deleting snapshot: snapshot-2
                "#,
                stdout
            );

            pa::assert_eq!(
                vec![
                    LxdContainer {
                        name: container_name("container-a"),
                        status: LxdContainerStatus::Running,
                        snapshots: Default::default(),
                    },
                    //
                    LxdContainer {
                        name: container_name("container-b"),
                        status: LxdContainerStatus::Running,
                        snapshots: Default::default(),
                    },
                    //
                    LxdContainer {
                        name: container_name("container-c"),
                        status: LxdContainerStatus::Stopping,
                        snapshots: Default::default(),
                    },
                    //
                    LxdContainer {
                        name: container_name("container-d"),
                        status: LxdContainerStatus::Stopped,
                        snapshots: vec![snapshot("snapshot-1", "2000-01-01 12:00:00")],
                    },
                ],
                lxd.list(&LxdProjectName::default()).unwrap()
            );
        }
    }
}
