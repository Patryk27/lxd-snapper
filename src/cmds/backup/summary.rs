use anyhow::{bail, Result};
use std::io::Write;

#[derive(Default)]
pub struct BackupSummary {
    pub processed_instances: usize,
    pub created_snapshots: usize,
    pub errors: usize,
}

impl BackupSummary {
    pub fn print(self, stdout: &mut dyn Write) -> Result<()> {
        if self.errors != 0 {
            bail!("Some instances couldn't be backed-up");
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
        writeln!(stdout, "- created snapshots: {}", self.created_snapshots)?;

        Ok(())
    }
}
