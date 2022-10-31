use crate::prelude::*;
use std::fmt;

pub struct PrettyLxdInstanceName<'a> {
    print_remote: bool,
    remote: &'a LxdRemoteName,
    print_project: bool,
    project: &'a LxdProjectName,
    instance: &'a LxdInstanceName,
}

impl<'a> PrettyLxdInstanceName<'a> {
    pub fn new(
        print_remote: bool,
        remote: &'a LxdRemoteName,
        print_project: bool,
        project: &'a LxdProjectName,
        instance: &'a LxdInstanceName,
    ) -> Self {
        Self {
            print_remote,
            remote,
            print_project,
            project,
            instance,
        }
    }
}

impl fmt::Display for PrettyLxdInstanceName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.print_remote {
            write!(f, "{}:", self.remote)?;
        }

        if self.print_project {
            write!(f, "{}/", self.project)?;
        }

        write!(f, "{}", self.instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(true, true, "remote:project/instance")]
    #[test_case(true, false, "remote:instance")]
    #[test_case(false, true, "project/instance")]
    #[test_case(false, false, "instance")]
    fn display(print_remote: bool, print_project: bool, expected: &str) {
        let actual = PrettyLxdInstanceName::new(
            print_remote,
            &LxdRemoteName::new("remote"),
            print_project,
            &LxdProjectName::new("project"),
            &LxdInstanceName::new("instance"),
        )
        .to_string();

        assert_eq!(expected, actual);
    }
}
