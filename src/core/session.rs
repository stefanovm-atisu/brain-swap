//! Session identity (TECHSPEC 3 I8, 6.4 steps 1, 2 and 4).

use crate::core::env::Env;
use jiff::{SignedDuration, Timestamp};
use std::fs;

pub const WARN_NOT_SUBSTITUTED: &str = "session id not substituted";
pub const WARN_INVALID: &str = "invalid session id";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdCheck {
    Valid,
    Placeholder,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub session: Option<String>,
    pub warnings: Vec<String>,
}

/// I8: empty or `$`, `{`, `}` is a placeholder; else valid when `[A-Za-z0-9._-]{1,128}`.
pub fn check_id(s: &str) -> IdCheck {
    if s.is_empty() || s.contains(['$', '{', '}']) {
        IdCheck::Placeholder
    } else if s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        IdCheck::Valid
    } else {
        IdCheck::Invalid
    }
}

/// Steps 1 (`flag`), 2 (`env.session`, empty counts as unset) and 4 of TECHSPEC 6.4; never fails.
/// A `None` session means steps 1 and 2 gave no valid ID; only the caller may then try step 3
/// (herdr's pane session), `resolve` reads nothing but `env.session` and calls no herdr.
pub fn resolve(flag: Option<&str>, env: &Env) -> Resolution {
    let mut warnings: Vec<String> = Vec::new();
    let env_session = env.session.as_deref().filter(|s| !s.is_empty());
    for id in [flag, env_session].into_iter().flatten() {
        let warn = match check_id(id) {
            IdCheck::Valid => {
                return Resolution {
                    session: Some(id.to_string()),
                    warnings,
                };
            }
            IdCheck::Placeholder => WARN_NOT_SUBSTITUTED,
            IdCheck::Invalid => WARN_INVALID,
        };
        if !warnings.iter().any(|w| w == warn) {
            warnings.push(warn.to_string());
        }
    }
    Resolution {
        session: None,
        warnings,
    }
}

/// Age after which `prune` removes a file from `sessions/` or `panes/` (TECHSPEC 10, T-17).
pub const PRUNE_AFTER: SignedDuration = SignedDuration::from_hours(720);

/// Removes every regular file in `sessions/` and `panes/` whose mtime lies more than
/// `PRUNE_AFTER` before `env.now`; missing folders and every error are ignored.
pub fn prune(env: &Env) {
    let now = env.now.timestamp();
    for dir in [env.sessions_dir(), env.panes_dir()] {
        for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
            let old = entry
                .metadata()
                .ok()
                .filter(|m| m.is_file())
                .and_then(|m| m.modified().ok())
                .and_then(|t| Timestamp::try_from(t).ok())
                .is_some_and(|t| now.duration_since(t) > PRUNE_AFTER);
            if old {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}
