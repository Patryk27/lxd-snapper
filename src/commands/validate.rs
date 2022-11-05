use crate::prelude::*;
use crate::Args;
use std::ops::DerefMut;

pub fn validate(stdout: &mut dyn Write, args: Args) -> Result<()> {
    let config = load_config(stdout, &args)?;

    writeln!(stdout)?;
    let mut lxd = init_lxd(stdout, &args, &config)?;

    writeln!(stdout)?;
    validate_config(stdout, &config, lxd.deref_mut())?;

    writeln!(stdout)?;
    writeln!(stdout, "✓ Everything seems to be fine")?;

    Ok(())
}

fn load_config(stdout: &mut dyn Write, args: &Args) -> Result<Config> {
    writeln!(
        stdout,
        "Loading configuration file: {}",
        args.config.display()
    )?;

    let config = Config::load(&args.config)?;

    writeln!(stdout, ".. [ OK ]")?;

    Ok(config)
}

fn init_lxd(stdout: &mut dyn Write, args: &Args, config: &Config) -> Result<Box<dyn LxdClient>> {
    writeln!(stdout, "Connecting to LXD")?;

    let lxd = crate::init_lxd(args, config)?;

    writeln!(stdout, ".. [ OK ]")?;

    Ok(lxd)
}

fn validate_config(stdout: &mut dyn Write, config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    writeln!(stdout, "Validating configuration file")?;

    let mut matching_instances = 0;

    for remote in config.remotes().iter() {
        for project in lxd.projects(remote)? {
            for instance in lxd.instances(remote, &project.name)? {
                if config.policies().matches(remote, &project, &instance) {
                    matching_instances += 1;
                }
            }
        }
    }

    if matching_instances == 0 {
        writeln!(
            stdout,
            "{} No instance matches any of the policies",
            "warn:".yellow()
        )?;
    }

    writeln!(stdout, ".. [ OK ]")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_result, assert_stdout, Command};
    use std::path::PathBuf;

    fn config(test: &str) -> PathBuf {
        PathBuf::from(file!())
            .parent()
            .unwrap()
            .join("validate")
            .join("tests")
            .join(test)
            .join("config.yaml")
    }

    #[test]
    fn missing_config() {
        let mut stdout = Vec::new();

        let args = Args {
            dry_run: false,
            config: "/tmp/ayy-ayy".into(),
            lxc_path: None,
            cmd: Command::Validate,
        };

        let result = validate(&mut stdout, args);

        assert_stdout!(
            r#"
            Loading configuration file: /tmp/ayy-ayy
            "#,
            stdout
        );

        assert_result!(
            r#"
            Couldn't load configuration from: /tmp/ayy-ayy

            Caused by:
                0: Couldn't read file
                1: No such file or directory (os error 2)
            "#,
            result
        );
    }

    #[test]
    fn missing_lxc_path() {
        let mut stdout = Vec::new();

        let args = Args {
            dry_run: false,
            config: config("missing_lxc_path"),
            lxc_path: Some("/tmp/ayy-ayy".into()),
            cmd: Command::Validate,
        };

        let result = validate(&mut stdout, args);

        assert_stdout!(
            r#"
            Loading configuration file: src/commands/validate/tests/missing_lxc_path/config.yaml
            .. [ OK ]

            Connecting to LXD
            "#,
            stdout
        );

        assert_result!(
            r#"
            Couldn't initialize LXC client

            Caused by:
                Couldn't find the `lxc` executable: /tmp/ayy-ayy
            "#,
            result
        );
    }
}
