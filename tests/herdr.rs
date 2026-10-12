//! TECHSPEC 9.1, 9.5: `capture` builds the herdr place from `HERDR_*` and runs no herdr command.

mod common;

use brain_swap::adapters::herdr::capture;
use brain_swap::adapters::runner::FakeRunner;
use brain_swap::core::env::Env;
use brain_swap::core::model::HerdrPlace;
use common::env::core_env;

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
