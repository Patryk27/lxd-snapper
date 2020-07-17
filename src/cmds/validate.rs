use crate::{config, lxd, Args};
use anyhow::Result;

crate fn validate(args: Args) -> Result<()> {
    println!("Checking:");
    println!();

    println!("- loading configuration file: {}", args.config.display());
    config(&args.config)?;
    println!("-> [ OK ]");
    println!();

    println!("- connecting to LXD");
    lxd(false, args.lxc_path)?;
    println!("-> [ OK ]");
    println!();

    println!("✓ Everything seems to be fine");

    Ok(())
}
