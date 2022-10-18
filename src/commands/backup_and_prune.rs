use super::{Backup, Prune};
use crate::prelude::*;
use anyhow::Error;

pub struct BackupAndPrune<'a, 'b> {
    env: &'a mut Environment<'b>,
}

impl<'a, 'b> BackupAndPrune<'a, 'b> {
    pub fn new(env: &'a mut Environment<'b>) -> Self {
        Self { env }
    }

    pub fn run(self) -> Result<()> {
        let backup_result = Backup::new(self.env).run();
        writeln!(self.env.stdout)?;
        let prune_result = Prune::new(self.env).run();

        match (backup_result, prune_result) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(err)) | (Err(err), Ok(_)) => Err(err),

            (Err(backup_err), Err(prune_err)) => {
                bail!(
                    "Couldn't backup and prune\n\nBackup error:\n{}\n\nPrune error:\n{}",
                    Self::format_error(backup_err),
                    Self::format_error(prune_err)
                )
            }
        }
    }

    fn format_error(err: Error) -> String {
        format!("{:?}", err)
            .lines()
            .map(|line| {
                if line.is_empty() {
                    Default::default()
                } else {
                    format!("    {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lxd::LxdFakeClient;
    use crate::{assert_err, assert_out};

    fn lxd() -> LxdFakeClient {
        let mut lxd = LxdFakeClient::default();

        lxd.add(LxdFakeInstance {
            name: "instance-a",
            ..Default::default()
        });

        lxd
    }

    mod when_backup_succeeds {
        use super::*;

        mod and_prune_succeeds {
            use super::*;

            const CONFIG: &str = indoc!(
                r#"
                policies:
                  all:
                    keep-last: 0
                "#
            );

            #[test]
            fn returns_ok() {
                let mut stdout = Vec::new();
                let config = Config::parse(CONFIG);
                let mut lxd = lxd();

                BackupAndPrune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                    .run()
                    .unwrap();

                assert_out!(
                    r#"
                    Backing-up instances:

                    - default/instance-a
                    -> creating snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - created snapshots: 1

                    Pruning instances:

                    - default/instance-a
                    -> deleting snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - deleted snapshots: 1
                    - kept snapshots: 0
                    "#,
                    stdout
                );
            }
        }

        mod and_prune_fails {
            use super::*;

            const CONFIG: &str = indoc!(
                r#"
                hooks:
                  on-prune-completed: "exit 1"

                policies:
                  all:
                    keep-last: 0
                "#
            );

            #[test]
            fn returns_prune_error() {
                let mut stdout = Vec::new();
                let config = Config::parse(CONFIG);
                let mut lxd = lxd();

                let result =
                    BackupAndPrune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                        .run();

                assert_out!(
                    r#"
                    Backing-up instances:

                    - default/instance-a
                    -> creating snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - created snapshots: 1

                    Pruning instances:

                    - default/instance-a
                    -> deleting snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - deleted snapshots: 1
                    - kept snapshots: 0
                    "#,
                    stdout
                );

                assert_err!(
                    r#"
                    Couldn't execute the `on-prune-completed` hook

                    Caused by:
                        Hook returned a non-zero exit code
                    "#,
                    result
                );
            }
        }
    }

    mod when_backup_fails {
        use super::*;

        mod and_prune_succeeds {
            use super::*;

            const CONFIG: &str = indoc!(
                r#"
                hooks:
                  on-backup-completed: "exit 1"

                policies:
                  all:
                    keep-last: 0
                "#
            );

            #[test]
            fn returns_backup_error() {
                let mut stdout = Vec::new();
                let config = Config::parse(CONFIG);
                let mut lxd = lxd();

                let result =
                    BackupAndPrune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                        .run();

                assert_out!(
                    r#"
                    Backing-up instances:

                    - default/instance-a
                    -> creating snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - created snapshots: 1

                    Pruning instances:

                    - default/instance-a
                    -> deleting snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - deleted snapshots: 1
                    - kept snapshots: 0
                    "#,
                    stdout
                );

                assert_err!(
                    r#"
                    Couldn't execute the `on-backup-completed` hook

                    Caused by:
                        Hook returned a non-zero exit code
                    "#,
                    result
                );
            }
        }

        mod and_prune_fails {
            use super::*;

            const CONFIG: &str = indoc!(
                r#"
                hooks:
                  on-backup-completed: "exit 1"
                  on-prune-completed: "exit 1"

                policies:
                  all:
                    keep-last: 0
                "#
            );

            #[test]
            fn returns_both_errors() {
                let mut stdout = Vec::new();
                let config = Config::parse(CONFIG);
                let mut lxd = lxd();

                let result =
                    BackupAndPrune::new(&mut Environment::test(&mut stdout, &config, &mut lxd))
                        .run();

                assert_out!(
                    r#"
                    Backing-up instances:

                    - default/instance-a
                    -> creating snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - created snapshots: 1

                    Pruning instances:

                    - default/instance-a
                    -> deleting snapshot: auto-19700101-000000
                    -> [ OK ]

                    Summary
                    - processed instances: 1
                    - deleted snapshots: 1
                    - kept snapshots: 0
                    "#,
                    stdout
                );

                assert_err!(
                    r#"
                    Couldn't backup and prune

                    Backup error:
                        Couldn't execute the `on-backup-completed` hook

                        Caused by:
                            Hook returned a non-zero exit code

                    Prune error:
                        Couldn't execute the `on-prune-completed` hook

                        Caused by:
                            Hook returned a non-zero exit code
                    "#,
                    result
                );
            }
        }
    }
}
