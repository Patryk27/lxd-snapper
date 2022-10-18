mod backup;
mod backup_and_prune;
mod debug_list_instances;
mod debug_nuke;
mod prune;
mod validate;

pub use self::{
    backup::*, backup_and_prune::*, debug_list_instances::*, debug_nuke::*, prune::*, validate::*,
};

#[cfg(test)]
#[macro_export]
macro_rules! assert_out {
    ($expected:literal, $actual:expr) => {
        pa::assert_str_eq!(indoc::indoc!($expected), String::from_utf8_lossy(&$actual));
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_err {
    ($expected:literal, $actual:expr) => {
        let actual = format!("{:?}", $actual.unwrap_err());

        pa::assert_str_eq!(indoc::indoc!($expected).trim(), actual);
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_lxd {
    ($expected:literal, $actual:expr) => {
        pa::assert_str_eq!(indoc::indoc!($expected), $actual.to_string());
    };
}
