//! TECHSPEC 9.1, 9.5: `capture` builds the herdr place from `HERDR_*` and runs no herdr command.
//! TECHSPEC 9.2: `pane_session` reads the current pane's agent session in at most one call.

mod common;

use brain_swap::adapters::herdr::{capture, pane_session};
use brain_swap::adapters::runner::{Answer, FakeRunner};
use brain_swap::core::env::Env;
use brain_swap::core::model::HerdrPlace;
use common::env::core_env;
use std::time::Duration;

const AGENT: &str = include_str!("fixtures/herdr/pane_agent.json");
const SHELL: &str = include_str!("fixtures/herdr/pane_shell.json");
const NOT_FOUND: &str = include_str!("fixtures/herdr/pane_not_found.stderr");

fn env(herdr: Option<&str>, pane: Option<&str>, tab: Option<&str>, ws: Option<&str>) -> Env {
    let mut env = core_env(std::path::Path::new("/nonexistent"), "2026-05-01 09:00");
    env.herdr_env = herdr.map(String::from);
    env.herdr_pane_id = pane.map(String::from);
    env.herdr_tab_id = tab.map(String::from);
    env.herdr_workspace_id = ws.map(String::from);
    env
}

#[test]
fn fr15_capture_builds_place_from_herdr_variables() {
    let fake = FakeRunner::new();
    let e = env(Some("1"), Some("w4V:p9"), Some("w4V:t1"), Some("w4V"));
    assert_eq!(
        capture(&e, &fake),
        Some(HerdrPlace {
            pane: "w4V:p9".into(),
            tab: Some("w4V:t1".into()),
            workspace: Some("w4V".into()),
        })
    );
    assert!(fake.calls().is_empty());
}

#[test]
fn fr58_no_capture_outside_herdr() {
    let fake = FakeRunner::new();
    let e = env(None, Some("w4V:p9"), Some("w4V:t1"), Some("w4V"));
    assert_eq!(capture(&e, &fake), None);
}

#[test]
fn fr58_herdr_env_must_be_1() {
    let fake = FakeRunner::new();
    for v in ["0", "true"] {
        let e = env(Some(v), Some("w4V:p9"), Some("w4V:t1"), Some("w4V"));
        assert_eq!(capture(&e, &fake), None, "HERDR_ENV={v}");
    }
}

#[test]
fn ts9_1_empty_tab_and_missing_workspace_are_absent() {
    let fake = FakeRunner::new();
    let e = env(Some("1"), Some("w4V:p9"), Some(""), None);
    assert_eq!(
        capture(&e, &fake),
        Some(HerdrPlace {
            pane: "w4V:p9".into(),
            tab: None,
            workspace: None,
        })
    );
}

#[test]
fn ts9_1_missing_pane_id_captures_nothing() {
    let fake = FakeRunner::new();
    let e = env(Some("1"), None, Some("w4V:t1"), None);
    assert_eq!(capture(&e, &fake), None);
}

fn in_pane() -> Env {
    env(Some("1"), Some("w4V:p9"), None, None)
}

fn answering(answer: Answer) -> FakeRunner {
    FakeRunner::new().on("pane get w4V:p9", answer)
}

#[test]
fn fr41_pane_session_returns_agent_session_value() {
    let fake = answering(Answer::ok(AGENT));
    assert_eq!(
        pane_session(&in_pane(), &fake),
        Some("55583127-a22a-49bc-a803-e777c377a595".into())
    );
    assert_eq!(fake.calls(), ["pane get w4V:p9"]);
    assert_eq!(fake.timeouts(), [Duration::from_secs(2)]);
}

#[test]
fn fr41_pane_without_agent_returns_none() {
    let fake = answering(Answer::ok(SHELL));
    assert_eq!(pane_session(&in_pane(), &fake), None);
    assert_eq!(fake.calls().len(), 1);
}

#[test]
fn fr58_pane_session_outside_herdr_calls_nothing() {
    let fake = FakeRunner::new();
    let e = env(None, Some("w4V:p9"), None, None);
    assert_eq!(pane_session(&e, &fake), None);
    assert!(fake.calls().is_empty());
}

#[test]
fn ts9_2_missing_pane_id_calls_nothing() {
    let fake = FakeRunner::new();
    let e = env(Some("1"), None, None, None);
    assert_eq!(pane_session(&e, &fake), None);
    assert!(fake.calls().is_empty());
}

#[test]
fn ts9_2_pane_not_found_returns_none() {
    let fake = answering(Answer::fail(1, NOT_FOUND));
    assert_eq!(pane_session(&in_pane(), &fake), None);
    assert_eq!(fake.calls().len(), 1);
}

#[test]
fn ts9_2_timeout_returns_none() {
    let fake = answering(Answer::Hang);
    assert_eq!(pane_session(&in_pane(), &fake), None);
    assert_eq!(fake.calls().len(), 1);
}

#[test]
fn ts9_2_unreadable_answer_returns_none() {
    let fake = answering(Answer::ok("not json"));
    assert_eq!(pane_session(&in_pane(), &fake), None);
    assert_eq!(fake.calls().len(), 1);
}
