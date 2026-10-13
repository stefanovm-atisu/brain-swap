//! TECHSPEC 6.3 and 6.7 `jump` through the fake herdr script (FR-59, FR-60, AS-10).

mod common;

use common::cli::Sandbox;
use common::fake_herdr::calls;

/// The jump sandbox inside herdr with fake herdr `scenario`, plus `tests/fixtures/jump/W-14.md`.
fn sandbox(scenario: &str) -> Sandbox {
    let mut s = Sandbox::new();
    s.write_config();
    s.copy_board("work_4_9", "work");
    let sc = s.scenario_mut();
    *sc = sc
        .clone()
        .fake_scenario(scenario)
        .herdr("w4V:p11", "w4V:t1", "w4V");
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jump/W-14.md");
    std::fs::copy(from, s.root().join("boards/work/W-14.md")).unwrap();
    s
}

/// Runs `args`, asserts exit 0 and returns the redacted stdout.
fn jump(s: &Sandbox, args: &[&str]) -> String {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    s.redact(&String::from_utf8(out.stdout).unwrap())
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
