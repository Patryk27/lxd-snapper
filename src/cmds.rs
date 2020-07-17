crate use self::{backup::*, backup_and_prune::*, nuke::*, prune::*, validate::*};

mod backup;
mod backup_and_prune;
mod nuke;
mod prune;
mod validate;

#[cfg(test)]
#[macro_export]
macro_rules! assert_out {
    ( $expected:literal, $actual:expr ) => {
        pretty_assertions::assert_eq!(indoc::indoc!($expected), String::from_utf8_lossy(&$actual));
    };
}
