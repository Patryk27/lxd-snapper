mod backup;
mod backup_and_prune;
mod debug_nuke;
mod prune;
mod query_instances;
mod validate;

pub use self::{
    backup::*, backup_and_prune::*, debug_nuke::*, prune::*, query_instances::*, validate::*,
};

#[cfg(test)]
#[macro_export]
macro_rules! assert_out {
    ($expected:literal, $actual:expr) => {
        pretty_assertions::assert_eq!(indoc::indoc!($expected), String::from_utf8_lossy(&$actual));
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_err {
    ($expected:literal, $actual:expr) => {
        let actual = format!("{:?}", $actual.unwrap_err());

        pretty_assertions::assert_eq!(indoc::indoc!($expected).trim(), actual);
    };
}
