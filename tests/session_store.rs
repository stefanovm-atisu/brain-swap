//! TECHSPEC 3 (I8) and 6.4: session ID check and resolution.

mod common;

use brain_swap::core::session::{IdCheck, Resolution, check_id, resolve};
use common::env::core_env;
use std::path::Path;

const UUID: &str = "55583127-a22a-49bc-a803-e777c377a595";

fn run(flag: Option<&str>, session: Option<&str>) -> Resolution {
    let mut env = core_env(Path::new("/h"), "2026-10-08T11:52:00+03:00");
    env.session = session.map(str::to_string);
    resolve(flag, &env)
}

fn res(session: Option<&str>, warnings: &[&str]) -> Resolution {
    Resolution {
        session: session.map(str::to_string),
        warnings: warnings.iter().map(|w| w.to_string()).collect(),
    }
}

#[test]
fn fr41_valid_flag_resolves_without_warning() {
    assert_eq!(run(Some(UUID), None), res(Some(UUID), &[]));
}

#[test]
fn i8_valid_ids_match_pattern() {
    for s in ["a", "A.b_c-1", &"x".repeat(128)] {
        assert_eq!(check_id(s), IdCheck::Valid, "{s}");
    }
}

#[test]
fn i8_empty_dollar_or_brace_is_placeholder() {
    for s in ["", "${CLAUDE_SESSION_ID}", "$X", "{x", "x}"] {
        assert_eq!(check_id(s), IdCheck::Placeholder, "{s}");
    }
}

#[test]
fn i8_other_bad_ids_are_invalid() {
    for s in ["a b", "a/b", "é", &"x".repeat(129)] {
        assert_eq!(check_id(s), IdCheck::Invalid, "{s}");
    }
}

#[test]
fn fr41_empty_flag_warns_and_falls_back_to_env() {
    assert_eq!(
        run(Some(""), Some("b")),
        res(Some("b"), &["session id not substituted"])
    );
}

#[test]
fn fr41_placeholder_flag_without_env_resolves_none() {
    assert_eq!(
        run(Some("${CLAUDE_SESSION_ID}"), None),
        res(None, &["session id not substituted"])
    );
}

#[test]
fn fr41_invalid_flag_resolves_none_with_warning() {
    assert_eq!(run(Some("a b"), None), res(None, &["invalid session id"]));
}

#[test]
fn ts6_4_missing_flag_uses_env_without_warning() {
    assert_eq!(run(None, Some("b")), res(Some("b"), &[]));
}

#[test]
fn ts6_4_valid_flag_wins_over_env() {
    assert_eq!(run(Some("a"), Some("b")).session.as_deref(), Some("a"));
}

#[test]
fn ts6_4_nothing_resolves_silently() {
    assert_eq!(run(None, None), res(None, &[]));
}

#[test]
fn ts6_4_invalid_env_session_warns_and_falls_through() {
    assert_eq!(
        run(None, Some("${X}")),
        res(None, &["session id not substituted"])
    );
    assert_eq!(run(None, Some("a b")), res(None, &["invalid session id"]));
    assert_eq!(run(None, Some("")), res(None, &[]));
}

#[test]
fn ts6_4_same_warning_given_once() {
    assert_eq!(
        run(Some(""), Some("${X}")),
        res(None, &["session id not substituted"])
    );
}
