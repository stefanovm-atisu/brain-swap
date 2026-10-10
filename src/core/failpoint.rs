//! Failpoints for crash tests (TECHSPEC 12.1), active only with feature `failpoints`.

use crate::core::env::Env;
use crate::core::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failpoint {
    pub name: String,
    /// True when read from `BRAIN_SWAP_FAILPOINT`: the process aborts instead of returning.
    pub abort: bool,
}

pub const NAMES: [&str; 5] = [
    "create:after_next",
    "create:after_tmp",
    "create:after_link",
    "park:after_tmp",
    "edit:after_tmp",
];

/// Stops the caller at `name` when `env.failpoint` names it; a no-op without the feature.
pub fn check(env: &Env, name: &str) -> Result<()> {
    match &env.failpoint {
        Some(fp) if cfg!(feature = "failpoints") && fp.name == name => {
            if fp.abort {
                std::process::abort();
            }
            Err(Error::Io(format!("failpoint {name}")))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::tz::TimeZone;

    fn env(fp: Option<&str>) -> Env {
        let mut vars = vec![("HOME".to_string(), "/h".to_string())];
        if let Some(name) = fp {
            vars.push(("BRAIN_SWAP_FAILPOINT".to_string(), name.to_string()));
        }
        let now = jiff::Timestamp::UNIX_EPOCH.to_zoned(TimeZone::UTC);
        Env::from_vars(vars, "/cwd".into(), 1, TimeZone::UTC, now).unwrap()
    }

    fn with(name: &str) -> Env {
        let mut e = env(None);
        e.failpoint = Some(Failpoint {
            name: name.to_string(),
            abort: false,
        });
        e
    }

    #[cfg(feature = "failpoints")]
    #[test]
    fn ts12_1_named_failpoint_returns_io_error() {
        assert_eq!(
            check(&with("park:after_tmp"), "park:after_tmp"),
            Err(Error::Io("failpoint park:after_tmp".to_string()))
        );
    }

    #[test]
    fn ts12_1_other_failpoint_name_passes() {
        assert_eq!(check(&with("park:after_tmp"), "create:after_tmp"), Ok(()));
    }

    #[test]
    fn ts12_1_no_failpoint_passes() {
        let e = env(None);
        assert_eq!(e.failpoint, None);
        for name in NAMES {
            assert_eq!(check(&e, name), Ok(()));
        }
    }

    #[cfg(feature = "failpoints")]
    #[test]
    fn ts12_1_env_reads_failpoint_with_abort() {
        assert_eq!(
            env(Some("park:after_tmp")).failpoint,
            Some(Failpoint {
                name: "park:after_tmp".to_string(),
                abort: true
            })
        );
    }

    #[cfg(not(feature = "failpoints"))]
    #[test]
    fn ts12_1_failpoint_ignored_without_feature() {
        let e = env(Some("park:after_tmp"));
        assert_eq!(e.failpoint, None);
        assert_eq!(check(&e, "park:after_tmp"), Ok(()));
    }
}
