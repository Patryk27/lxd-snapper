use anyhow::{bail, Result};
use std::io::Write;

#[derive(Default)]
pub struct Summary {
    pub processed_instances: usize,
    pub deleted_snapshots: usize,
    pub kept_snapshots: usize,
    pub errors: usize,
}

impl Summary {
    pub fn print(self, stdout: &mut dyn Write) -> Result<()> {
        if self.errors != 0 {
            bail!("Some instances couldn't be pruned");
        }

        if self.processed_instances == 0 {
            bail!("Found no instances matching any of the policies");
        }

        writeln!(stdout)?;
        writeln!(stdout, "Summary")?;
        writeln!(
            stdout,
            "- processed instances: {}",
            self.processed_instances
        )?;
        writeln!(stdout, "- deleted snapshots: {}", self.deleted_snapshots)?;
        writeln!(stdout, "- kept snapshots: {}", self.kept_snapshots)?;

        Ok(())
    }
}
