use anyhow::{bail, Result};
use std::io::Write;

#[derive(Default)]
pub struct BackupSummary {
    pub processed_containers: usize,
    pub created_snapshots: usize,
    pub errors: usize,
}

impl BackupSummary {
    pub fn print(self, stdout: &mut dyn Write) -> Result<()> {
        if self.errors != 0 {
            bail!("Some containers failed to be backed-up");
        }

        if self.processed_containers == 0 {
            bail!("Found no containers");
        }

        writeln!(stdout)?;
        writeln!(stdout, "Summary")?;
        writeln!(
            stdout,
            "- processed containers: {}",
            self.processed_containers
        )?;
        writeln!(stdout, "- created snapshots: {}", self.created_snapshots)?;

        Ok(())
    }
}
