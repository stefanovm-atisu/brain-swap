//! TECHSPEC 6.3 and 6.7 `locate`, 9.3 and 9.5 through the fake herdr script (FR-40, FR-60 to FR-63).

mod common;

use common::cli::Sandbox;
use common::fake_herdr::calls;
use serde_json::Value;
use std::time::{Duration, Instant};

const BILLING: &str = "Where: /home/smilen/Work/ati.billing";

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

/// The jump sandbox with fake herdr `scenario`, inside herdr when `inside`.
fn sandbox(scenario: &str, inside: bool) -> Sandbox {
    let mut s = Sandbox::new();
    s.write_config();
    s.copy_board("work_4_9", "work");
    let sc = s.scenario_mut();
    let mut next = sc.clone().fake_scenario(scenario);
    if inside {
        next = next.herdr("w4V:p11", "w4V:t1", "w4V");
    }
    *sc = next;
    s
}

/// Runs `args`, asserts exit 0 and returns stdout with the wall time.
fn locate(s: &Sandbox, args: &[&str]) -> (String, Duration) {
    let start = Instant::now();
    let out = s.cmd(args).assert().code(0).get_output().clone();
    (s.redact(&text(&out.stdout)), start.elapsed())
}

fn log(s: &Sandbox) -> Vec<String> {
    calls(&s.root().join("home"))
}

#[test]
fn fr40_locate_live_pane() {
    let s = sandbox("live-agent", true);
    let (out, _) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out, format!("live w4V:p9\n{BILLING}\n"));
    assert_eq!(log(&s), ["pane get w4V:p9"]);
}

#[test]
fn fr60_locate_moved_pane() {
    let s = sandbox("moved", true);
    let (out, _) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out.lines().next(), Some("moved w7Q:p2"), "{out}");
    assert_eq!(
        log(&s),
        [
            "pane get w4V:p9",
            "workspace list",
            "pane list --workspace w4V",
            "pane list --workspace w7Q"
        ]
    );
}

#[test]
fn fr61_locate_closed_pane() {
    let s = sandbox("closed", true);
    let (out, _) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out, format!("closed w4V:p9\n{BILLING}\n"));
}

#[test]
fn fr62_locate_outside_herdr() {
    let s = sandbox("live-agent", false);
    let (out, _) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out, format!("not in herdr\n{BILLING}\n"));
    assert!(log(&s).is_empty());
}

#[test]
fn fr62_locate_note_without_pane() {
    let s = sandbox("live-agent", true);
    let (out, _) = locate(&s, &["locate", "W-9"]);
    assert_eq!(
        out,
        "no pane recorded\nWhere: /home/smilen/Work/private/brain-swap\n"
    );
    assert!(log(&s).is_empty());
}

#[test]
fn fr63_locate_hang_is_not_responding() {
    let s = sandbox("hang", true);
    let (out, took) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out, format!("herdr not responding\n{BILLING}\n"));
    assert!(took < Duration::from_millis(2500), "{took:?}");
}

#[test]
fn ts9_3_locate_herdr_error() {
    let s = sandbox("herdr-error", true);
    let (out, _) = locate(&s, &["locate", "W-12"]);
    assert_eq!(
        out.lines().next(),
        Some(r#"herdr: {"error":{"code":"internal","message":"socket closed"}}"#),
        "{out}"
    );
}

#[test]
fn as11_locate_answers_within_3_seconds() {
    let s = sandbox("slow-then-hang", true);
    let (out, took) = locate(&s, &["locate", "W-12"]);
    assert_eq!(out, format!("herdr not responding\n{BILLING}\n"));
    assert!(
        took >= Duration::from_millis(2900) && took <= Duration::from_millis(3500),
        "{took:?}"
    );
}

#[test]
fn fr40_locate_never_focuses() {
    for scenario in ["live-agent", "live-shell", "moved"] {
        let s = sandbox(scenario, true);
        locate(&s, &["locate", "W-12"]);
        let calls = log(&s);
        assert!(!calls.is_empty(), "{scenario}");
        assert!(!calls.iter().any(|c| c.contains("focus")), "{calls:?}");
    }
}

#[test]
fn ts6_7_locate_json_per_status() {
    let cases = [
        ("live-agent", true, "W-12", "live", Some("w4V:p9")),
        ("moved", true, "W-12", "moved", Some("w7Q:p2")),
        ("closed", true, "W-12", "closed", Some("w4V:p9")),
        ("hang", true, "W-12", "not_responding", Some("w4V:p9")),
        ("herdr-error", true, "W-12", "herdr_error", Some("w4V:p9")),
        ("live-agent", false, "W-12", "outside", Some("w4V:p9")),
        ("live-agent", true, "W-9", "none", None),
    ];
    for (scenario, inside, id, status, pane) in cases {
        let s = sandbox(scenario, inside);
        let (out, _) = locate(&s, &["--json", "locate", id]);
        assert_eq!(out.lines().count(), 1, "{out}");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["status"], status, "{out}");
        assert_eq!(v["pane"].as_str(), pane, "{out}");
        let cwd = match id {
            "W-9" => "/home/smilen/Work/private/brain-swap",
            _ => "/home/smilen/Work/ati.billing",
        };
        assert_eq!(v["cwd"], cwd, "{out}");
        insta::assert_snapshot!(status, out);
    }
}

#[test]
fn ts6_2_locate_unknown_card_exits_3() {
    let s = sandbox("live-agent", true);
    let out = s
        .cmd(&["locate", "W-99"])
        .assert()
        .code(3)
        .get_output()
        .clone();
    assert_eq!(text(&out.stderr), "brain-swap: W-99 not found\n");
    let mut out = s.cmd(&["--json", "locate", "W-99"]);
    let out = out.assert().code(3).get_output().clone();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "not_found");
    assert!(log(&s).is_empty());
}
