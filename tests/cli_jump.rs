//! TECHSPEC 6.3 and 6.7 `jump` through the fake herdr script (FR-59, FR-60, AS-10).

mod common;

use common::cli::Sandbox;
use common::fake_herdr::calls;
use serde_json::Value;
use std::time::{Duration, Instant};

const BILLING: &str = "Where: /home/smilen/Work/ati.billing";

/// The jump sandbox inside herdr with fake herdr `scenario`, plus `tests/fixtures/jump/W-14.md`.
fn sandbox(scenario: &str) -> Sandbox {
    sandbox_at(scenario, true)
}

/// The jump sandbox with fake herdr `scenario`, inside herdr when `inside`.
fn sandbox_at(scenario: &str, inside: bool) -> Sandbox {
    let mut s = Sandbox::new();
    s.write_config();
    s.copy_board("work_4_9", "work");
    let sc = s.scenario_mut();
    let mut next = sc.clone().fake_scenario(scenario);
    if inside {
        next = next.herdr("w4V:p11", "w4V:t1", "w4V");
    }
    *sc = next;
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jump/W-14.md");
    std::fs::copy(from, s.root().join("boards/work/W-14.md")).unwrap();
    s
}

/// Runs `args`, asserts exit 0 and returns the redacted stdout.
fn jump(s: &Sandbox, args: &[&str]) -> String {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    s.redact(&String::from_utf8(out.stdout).unwrap())
}

/// Runs `args`, asserts exit 6, an empty stderr and, with `--json`, no `error` key; returns
/// the redacted stdout and the wall time.
fn no_jump(s: &Sandbox, args: &[&str]) -> (String, Duration) {
    let start = Instant::now();
    let out = s.cmd(args).assert().code(6).stderr("").get_output().clone();
    let elapsed = start.elapsed();
    let stdout = s.redact(&String::from_utf8(out.stdout).unwrap());
    if args.contains(&"--json") {
        let v: Value = serde_json::from_str(&stdout).unwrap();
        assert!(v.get("error").is_none(), "{stdout}");
    }
    (stdout, elapsed)
}

fn log(s: &Sandbox) -> Vec<String> {
    calls(&s.root().join("home"))
}

#[test]
fn fr59_jump_focuses_the_agent_pane() {
    let s = sandbox("live-agent");
    assert_eq!(
        jump(&s, &["jump", "W-12"]),
        "jumped to W-12 (pane w4V:p9)\n"
    );
    assert_eq!(log(&s), ["pane get w4V:p9", "agent focus w4V:p9"]);
}

#[test]
fn fr59_jump_without_agent_focuses_the_tab() {
    let s = sandbox("live-shell");
    assert_eq!(
        jump(&s, &["jump", "W-12"]),
        "jumped to W-12 (tab w4V:t1; pane not focusable)\n"
    );
    assert_eq!(
        log(&s),
        ["pane get w4V:p9", "workspace focus w4V", "tab focus w4V:t1"]
    );
}

#[test]
fn as10_jump_to_a_moved_pane_reports_its_new_id() {
    let s = sandbox("moved");
    assert_eq!(jump(&s, &["jump", "W-12"]), "pane moved: now w7Q:p2\n");
    assert_eq!(
        log(&s).last().map(String::as_str),
        Some("agent focus w7Q:p2")
    );
}

#[test]
fn ts6_7_jump_json_names_what_was_focused() {
    for scenario in ["live-agent", "live-shell", "moved"] {
        let s = sandbox(scenario);
        let out = jump(&s, &["--json", "jump", "W-12"]);
        insta::assert_snapshot!(scenario, out);
    }
}

#[test]
fn ts6_3_jump_note_flag_targets_that_note() {
    let s = sandbox("live-shell");
    assert_eq!(
        jump(&s, &["jump", "W-14", "--note", "1"]),
        "jumped to W-14 (tab w4V:t2; pane not focusable)\n"
    );
    assert_eq!(log(&s).first().map(String::as_str), Some("pane get w4V:p3"));
}

#[test]
fn ts6_3_jump_failed_focus_exits_6() {
    let s = sandbox("focus-fails");
    let (out, _) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("focus failed: w4V:p9\n{BILLING}\n"));
}

#[test]
fn ts6_3_jump_failed_focus_json_keeps_status() {
    let s = sandbox("focus-fails");
    let (out, _) = no_jump(&s, &["--json", "jump", "W-12"]);
    assert_eq!(
        out.trim_end(),
        r#"{"v":1,"status":"live","pane":"w4V:p9","cwd":"/home/smilen/Work/ati.billing","focused":false}"#
    );
}

#[test]
fn ts6_3_jump_focus_timeout_exits_6_not_responding() {
    let s = sandbox("focus-hangs");
    let (out, _) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("herdr not responding\n{BILLING}\n"));
    let (out, _) = no_jump(&s, &["--json", "jump", "W-12"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["status"], "not_responding", "{out}");
    assert_eq!(v["focused"], false, "{out}");
}

#[test]
fn fr61_jump_to_a_closed_pane_exits_6() {
    let s = sandbox("closed");
    let (out, _) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("closed w4V:p9\n{BILLING}\n"));
    no_jump(&s, &["--json", "jump", "W-12"]);
}

#[test]
fn fr62_jump_outside_herdr_exits_6_without_herdr_call() {
    let s = sandbox_at("live-agent", false);
    let (out, _) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("not in herdr\n{BILLING}\n"));
    no_jump(&s, &["--json", "jump", "W-12"]);
    assert!(log(&s).is_empty());
}

#[test]
fn fr62_jump_to_a_note_without_pane_exits_6() {
    let s = sandbox("live-agent");
    let (out, _) = no_jump(&s, &["jump", "W-9"]);
    assert_eq!(
        out,
        "no pane recorded\nWhere: /home/smilen/Work/private/brain-swap\n"
    );
    no_jump(&s, &["--json", "jump", "W-9"]);
}

#[test]
fn fr63_jump_while_herdr_hangs_exits_6() {
    let s = sandbox("hang");
    let (out, took) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("herdr not responding\n{BILLING}\n"));
    assert!(took < Duration::from_millis(2500), "{took:?}");
}

#[test]
fn ts9_3_jump_herdr_error_exits_6() {
    let s = sandbox("herdr-error");
    let (out, _) = no_jump(&s, &["jump", "W-12"]);
    assert!(out.starts_with("herdr: "), "{out}");
    no_jump(&s, &["--json", "jump", "W-12"]);
}

#[test]
fn ts6_3_jump_card_without_note_exits_6() {
    let s = sandbox("live-agent");
    let (out, _) = no_jump(&s, &["jump", "W-11"]);
    assert_eq!(out, "no note yet\n");
    let (out, _) = no_jump(&s, &["--json", "jump", "W-11"]);
    assert_eq!(
        out.trim_end(),
        r#"{"v":1,"status":"no_note","pane":null,"cwd":null,"focused":false}"#
    );
}

#[test]
fn as11_jump_budget_is_shared_with_focus() {
    let s = sandbox("slow-live-focus-hangs");
    let (out, took) = no_jump(&s, &["jump", "W-12"]);
    assert_eq!(out, format!("herdr not responding\n{BILLING}\n"));
    assert!(
        took >= Duration::from_millis(2900) && took <= Duration::from_millis(3500),
        "{took:?}"
    );
}
