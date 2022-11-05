use anyhow::{bail, Result};
use colored::Colorize;
use std::fmt;

pub struct Summary {
    title: &'static str,
    processed_instances: usize,
    created_snapshots: Option<usize>,
    deleted_snapshots: Option<usize>,
    kept_snapshots: Option<usize>,
    errors: usize,
}

impl Summary {
    pub fn with_created_snapshots(mut self) -> Self {
        self.created_snapshots = Some(0);
        self
    }

    pub fn with_deleted_snapshots(mut self) -> Self {
        self.deleted_snapshots = Some(0);
        self
    }

    pub fn with_kept_snapshots(mut self) -> Self {
        self.kept_snapshots = Some(0);
        self
    }

    pub fn set_title(&mut self, title: &'static str) {
        self.title = title;
    }

    pub fn add_processed_instance(&mut self) {
        self.processed_instances += 1;
    }

    pub fn add_created_snapshot(&mut self) {
        *self.created_snapshots.as_mut().unwrap() += 1;
    }

    pub fn add_deleted_snapshot(&mut self) {
        *self.deleted_snapshots.as_mut().unwrap() += 1;
    }

    pub fn add_kept_snapshot(&mut self) {
        *self.kept_snapshots.as_mut().unwrap() += 1;
    }

    pub fn add_error(&mut self) {
        self.errors += 1;
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    pub fn as_result(&self) -> Result<()> {
        if self.processed_instances == 0 {
            bail!("Found no instance(s) that would match the configured policies");
        }

        Ok(())
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            title: "Summary",
            processed_instances: Default::default(),
            created_snapshots: Default::default(),
            deleted_snapshots: Default::default(),
            kept_snapshots: Default::default(),
            errors: Default::default(),
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.title.bold())?;
        writeln!(f, "{}", "-".repeat(self.title.chars().count()))?;
        writeln!(f, "  processed instances: {}", self.processed_instances)?;

        if let Some(n) = self.created_snapshots {
            writeln!(f, "  created snapshots: {}", n)?;
        }

        if let Some(n) = self.deleted_snapshots {
            writeln!(f, "  deleted snapshots: {}", n)?;
        }

        if let Some(n) = self.kept_snapshots {
            writeln!(f, "  kept snapshots: {}", n)?;
        }

        Ok(())
    }
}
