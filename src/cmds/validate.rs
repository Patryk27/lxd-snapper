use crate::config::Config;
use crate::Args;
use anyhow::{bail, Result};
use lib_lxd::LxdClient;
use std::ops::DerefMut;

pub(crate) fn validate(args: Args) -> Result<()> {
    let config = load_config(&args)?;
    println!();

    let mut lxd = init_lxd(&args)?;
    println!();

    validate_config(&config, lxd.deref_mut())?;
    println!();

    println!("✓ Everything seems to be fine");

    Ok(())
}

fn load_config(args: &Args) -> Result<Config> {
    println!("Loading configuration file: {}", args.config.display());
    let config = crate::load_config(&args.config)?;
    println!(".. [ OK ]");

    Ok(config)
}

fn init_lxd(args: &Args) -> Result<Box<dyn LxdClient>> {
    println!("Connecting to LXD");
    let lxd = crate::init_lxd(false, args.lxc_path.clone())?;
    println!(".. [ OK ]");

    Ok(lxd)
}

fn validate_config(config: &Config, lxd: &mut dyn LxdClient) -> Result<()> {
    println!("Validating configuration file");

    let mut instances_with_policies = 0;

    for project in lxd.list_projects()? {
        for instance in lxd.list(&project.name)? {
            if config.policy(&project, &instance).is_some() {
                instances_with_policies += 1;
            }
        }
    }

    if instances_with_policies == 0 {
        bail!("No instance matches any of the policies");
    }

    println!(".. [ OK ]");

    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO tests for `validate_config`
}
