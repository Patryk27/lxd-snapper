use crate::prelude::*;
use lib_lxd::LxdClient;

pub struct Environment<'a> {
    pub time: fn() -> DateTime<Utc>,
    pub stdout: &'a mut dyn Write,
    pub config: &'a Config,
    pub lxd: &'a mut dyn LxdClient,
}

impl<'a> Environment<'a> {
    #[cfg(test)]
    pub fn test(stdout: &'a mut dyn Write, config: &'a Config, lxd: &'a mut dyn LxdClient) -> Self {
        use chrono::TimeZone;

        Self {
            time: || Utc.timestamp(0, 0),
            stdout,
            config,
            lxd,
        }
    }

    pub fn time(&self) -> DateTime<Utc> {
        (self.time)()
    }
}
