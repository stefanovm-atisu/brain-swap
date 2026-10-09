//! Errors, exit codes and error codes (TECHSPEC 6.2, 11).

use std::fmt;
use std::path::PathBuf;

/// Exit 6: `jump` did not focus the pane; it has no error object (6.2).
pub const EXIT_JUMP_NOT_PERFORMED: i32 = 6;

/// Every failure of brain-swap, one line each.
///
/// Exit codes (FR-36, TECHSPEC 6.2):
/// - 0 success
/// - 1 runtime error: `io`, `unreadable`, `invalid_input`, `config`, `verify_failed`
/// - 2 usage, or the TUI without a terminal: `usage`
/// - 3 not found (card, board, template or column): `not_found`
/// - 4 no reference (`park` without `--card`): `no_reference`
/// - 5 busy (no lock in 2 seconds): `busy`
/// - 6 jump not performed ([`EXIT_JUMP_NOT_PERFORMED`], no error object)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Io(String),
    Unreadable(String),
    InvalidInput(String),
    Config {
        path: PathBuf,
        line: usize,
        col: usize,
        msg: String,
    },
    Verify(String),
    Usage(String),
    NotFound(String),
    NoReference(String),
    Busy(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage(_) => 2,
            Error::NotFound(_) => 3,
            Error::NoReference(_) => 4,
            Error::Busy(_) => 5,
            _ => 1,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Error::Io(_) => "io",
            Error::Unreadable(_) => "unreadable",
            Error::InvalidInput(_) => "invalid_input",
            Error::Config { .. } => "config",
            Error::Verify(_) => "verify_failed",
            Error::Usage(_) => "usage",
            Error::NotFound(_) => "not_found",
            Error::NoReference(_) => "no_reference",
            Error::Busy(_) => "busy",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config {
                path,
                line,
                col,
                msg,
            } => write!(f, "config {}:{line}:{col}: {msg}", path.display()),
            Error::Io(m)
            | Error::Unreadable(m)
            | Error::InvalidInput(m)
            | Error::Verify(m)
            | Error::Usage(m)
            | Error::NotFound(m)
            | Error::NoReference(m)
            | Error::Busy(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(m: &str) -> String {
        m.to_string()
    }

    fn config() -> Error {
        Error::Config {
            path: "/h/.config/brain-swap/config.toml".into(),
            line: 4,
            col: 7,
            msg: s("expected '='"),
        }
    }

    #[test]
    fn fr36_not_found_exits_3() {
        let e = Error::NotFound(s("W-99 not found"));
        assert_eq!((e.exit_code(), e.code()), (3, "not_found"));
    }

    #[test]
    fn fr36_no_reference_exits_4() {
        let e = Error::NoReference(s("no session reference"));
        assert_eq!((e.exit_code(), e.code()), (4, "no_reference"));
    }

    #[test]
    fn fr36_busy_exits_5() {
        let e = Error::Busy(s("board is busy"));
        assert_eq!((e.exit_code(), e.code()), (5, "busy"));
    }

    #[test]
    fn ts6_2_runtime_errors_exit_1() {
        let cases = [
            (Error::Io(s("x")), "io"),
            (Error::Unreadable(s("x")), "unreadable"),
            (Error::InvalidInput(s("x")), "invalid_input"),
            (config(), "config"),
            (Error::Verify(s("x")), "verify_failed"),
        ];
        for (e, code) in cases {
            assert_eq!((e.exit_code(), e.code()), (1, code));
        }
    }

    #[test]
    fn ts6_2_usage_exits_2() {
        let e = Error::Usage(s("unknown flag"));
        assert_eq!((e.exit_code(), e.code()), (2, "usage"));
    }

    #[test]
    fn fr36_exit_codes_are_distinct() {
        let codes = [
            Error::NotFound(s("x")).exit_code(),
            Error::NoReference(s("x")).exit_code(),
            Error::Busy(s("x")).exit_code(),
            EXIT_JUMP_NOT_PERFORMED,
        ];
        assert_eq!(codes, [3, 4, 5, 6]);
        for (i, a) in codes.iter().enumerate() {
            assert!(![0, 1, 2].contains(a));
            assert!(!codes[i + 1..].contains(a));
        }
    }

    #[test]
    fn ts11_config_display_names_file_line_col() {
        assert_eq!(
            config().to_string(),
            "config /h/.config/brain-swap/config.toml:4:7: expected '='"
        );
    }

    #[test]
    fn ts11_display_is_one_line() {
        let all = [
            Error::Io(s("a")),
            Error::Unreadable(s("a")),
            Error::InvalidInput(s("a")),
            config(),
            Error::Verify(s("a")),
            Error::Usage(s("a")),
            Error::NotFound(s("a")),
            Error::NoReference(s("a")),
            Error::Busy(s("a")),
        ];
        for e in all {
            assert!(!e.to_string().contains('\n'), "{e:?}");
        }
    }
}
