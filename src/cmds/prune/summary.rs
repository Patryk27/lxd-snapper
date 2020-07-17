use anyhow::{bail, Result};
use std::io::Write;

#[derive(Default)]
pub struct PruneSummary {
    pub processed_containers: usize,
    pub deleted_snapshots: usize,
    pub kept_snapshots: usize,
    pub errors: usize,
}

impl PruneSummary {
    pub fn print(self, stdout: &mut dyn Write) -> Result<()> {
        if self.errors != 0 {
            bail!("Some containers failed to be pruned");
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
        writeln!(stdout, "- deleted snapshots: {}", self.deleted_snapshots)?;
        writeln!(stdout, "- kept snapshots: {}", self.kept_snapshots)?;

        Ok(())
    }
}
