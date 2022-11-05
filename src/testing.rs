use crate::prelude::*;
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use std::fmt::Write;

#[macro_export]
macro_rules! assert_stdout {
    ($expected:literal, $actual:expr) => {
        $crate::testing::assert_out(
            indoc::indoc!($expected).trim(),
            String::from_utf8_lossy(&$actual).trim(),
        );
    };
}

#[macro_export]
macro_rules! assert_result {
    ($expected:literal, $actual:expr) => {
        let actual = format!("{:?}", $actual.unwrap_err());

        pa::assert_str_eq!(indoc::indoc!($expected).trim(), actual);
    };
}

#[macro_export]
macro_rules! assert_lxd {
    ($expected:literal, $actual:expr) => {
        pa::assert_str_eq!(indoc::indoc!($expected), $actual.to_string());
    };
}

#[track_caller]
pub fn assert_out(expected: impl AsRef<str>, actual: impl AsRef<str>) {
    let actual = sanitize_ansi_codes(actual);
    let actual = sanitize_empty_lines(actual);
    let expected = sanitize_empty_lines(expected);

    pa::assert_str_eq!(expected, actual);
}

fn sanitize_ansi_codes(s: impl AsRef<str>) -> String {
    let mut out = String::new();
    let mut active_modes = Vec::new();

    for item in s.as_ref().ansi_parse() {
        match item {
            Output::TextBlock(text) => {
                _ = write!(out, "{}", text);
            }

            Output::Escape(escape) => match escape {
                AnsiSequence::SetGraphicsMode(modes) => {
                    for mode in modes {
                        match mode {
                            0 => {
                                while let Some(mode) = active_modes.pop() {
                                    _ = write!(out, "</{}>", mode);
                                }
                            }

                            1 => {
                                _ = write!(out, "<b>");
                                active_modes.push("b");
                            }

                            3 => {
                                _ = write!(out, "<i>");
                                active_modes.push("i");
                            }

                            color @ 30..=37 => {
                                _ = write!(out, "<fg={}>", color);
                                active_modes.push("fg");
                            }

                            mode => {
                                panic!("Unrecognized SetGraphicsMode: {}", mode);
                            }
                        }
                    }
                }

                escape => {
                    panic!("Unrecognized escape: {:?}", escape);
                }
            },
        }
    }

    out
}

fn sanitize_empty_lines(s: impl AsRef<str>) -> String {
    s.as_ref()
        .lines()
        .map(|line| if line.trim().is_empty() { "" } else { line })
        .join("\n")
}
