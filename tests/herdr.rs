//! TECHSPEC 9.1, 9.5: `capture` builds the herdr place from `HERDR_*` and runs no herdr command.
//! TECHSPEC 9.2: `pane_session` reads the current pane's agent session in at most one call.
//! TECHSPEC 9.3: `locate` turns a note's place into a `PaneStatus`.

mod common;

use brain_swap::adapters::herdr::{LivePane, PaneStatus, capture, locate, pane_session};
use brain_swap::adapters::runner::{Answer, FakeRunner};
use brain_swap::core::env::Env;
use brain_swap::core::model::{HerdrPlace, Place};
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

/// The place of W-12's second note in the 4.9 board.
fn w12_place() -> Place {
    Place {
        cwd: Some("/home/smilen/Work/ati.billing".into()),
        herdr: Some(HerdrPlace {
            pane: "w4V:p9".into(),
            tab: Some("w4V:t1".into()),
            workspace: Some("w4V".into()),
        }),
        session: Some("55583127-a22a-49bc-a803-e777c377a595".into()),
    }
}

fn in_herdr() -> Env {
    env(Some("1"), None, None, None)
}

/// `pane_agent.json` with its pane object edited.
fn agent_with(edit: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>)) -> String {
    let mut v: serde_json::Value = serde_json::from_str(AGENT).unwrap();
    edit(v["result"]["pane"].as_object_mut().unwrap());
    v.to_string()
}

fn live(status: PaneStatus) -> LivePane {
    match status {
        PaneStatus::Live(p) => p,
        other => panic!("expected Live, got {other:?}"),
    }
}

#[test]
fn fr62_outside_herdr_is_outside_without_call() {
    let fake = FakeRunner::new();
    let e = env(None, None, None, None);
    assert_eq!(locate(&w12_place(), &e, &fake), PaneStatus::OutsideHerdr);
    assert!(fake.calls().is_empty());
}

#[test]
fn nfr05_pane_variables_alone_do_not_enable_herdr() {
    let fake = FakeRunner::new();
    let e = env(None, Some("w4V:p11"), None, None);
    assert_eq!(locate(&w12_place(), &e, &fake), PaneStatus::OutsideHerdr);
    assert!(fake.calls().is_empty());
}

#[test]
fn fr62_note_without_pane_is_no_pane_without_call() {
    let fake = FakeRunner::new();
    let place = Place {
        cwd: Some("/home/smilen/Work/private/brain-swap".into()),
        ..Place::default()
    };
    assert_eq!(locate(&place, &in_herdr(), &fake), PaneStatus::NoPane);
    assert!(fake.calls().is_empty());
}

#[test]
fn fr40_live_pane_is_live() {
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok(AGENT));
    assert_eq!(
        locate(&w12_place(), &in_herdr(), &fake),
        PaneStatus::Live(LivePane {
            pane: "w4V:p9".into(),
            tab: "w4V:t1".into(),
            workspace: "w4V".into(),
            agent: true,
        })
    );
    assert_eq!(fake.calls(), ["pane get w4V:p9"]);
    assert_eq!(fake.timeouts(), [Duration::from_secs(2)]);
}

#[test]
fn ts9_3_live_ids_win_over_stored() {
    let answer = agent_with(|p| p["tab_id"] = "w4V:t3".into());
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok(&answer));
    assert_eq!(live(locate(&w12_place(), &in_herdr(), &fake)).tab, "w4V:t3");
}

#[test]
fn ts9_3_live_shell_pane_has_no_agent() {
    let fake = FakeRunner::new().on("pane get w4V:p3", Answer::ok(SHELL));
    let place = Place {
        herdr: Some(HerdrPlace {
            pane: "w4V:p3".into(),
            tab: None,
            workspace: None,
        }),
        ..Place::default()
    };
    let p = live(locate(&place, &in_herdr(), &fake));
    assert_eq!(p.tab, "w4V:t2");
    assert!(!p.agent);
}

#[test]
fn ts9_3_live_answer_without_tab_keeps_stored_tab() {
    let answer = agent_with(|p| {
        p.remove("tab_id");
    });
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok(&answer));
    assert_eq!(live(locate(&w12_place(), &in_herdr(), &fake)).tab, "w4V:t1");
}

#[test]
fn ts9_3_unreadable_live_answer_is_herdr_error() {
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok("not json"));
    assert_eq!(
        locate(&w12_place(), &in_herdr(), &fake),
        PaneStatus::HerdrError("pane get: unreadable answer".into())
    );
}

#[test]
fn fr60_old_id_of_moved_pane_is_moved() {
    let answer = agent_with(|p| {
        p["pane_id"] = "w7Q:p2".into();
        p["tab_id"] = "w7Q:t1".into();
        p["workspace_id"] = "w7Q".into();
    });
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok(&answer));
    assert_eq!(
        locate(&w12_place(), &in_herdr(), &fake),
        PaneStatus::Moved(LivePane {
            pane: "w7Q:p2".into(),
            tab: "w7Q:t1".into(),
            workspace: "w7Q".into(),
            agent: true,
        })
    );
    assert_eq!(fake.calls(), ["pane get w4V:p9"]);
}
