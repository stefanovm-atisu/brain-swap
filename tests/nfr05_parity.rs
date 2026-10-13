//! NFR-05, FR-58, FR-62, TECHSPEC 9.5: the Core CLI flow is the same inside and outside herdr.

mod common;

use common::cli::Sandbox;
use common::fake_herdr::calls;
use std::collections::BTreeSet;

const FLOW: &[&[&str]] = &[
    &["new", "parity", "check", "--session", "parity-s1"],
    &["move", "W-13", "Doing"],
    &["park", "--card", "W-13", "--session", "parity-s1"],
    &["link", "W-13", "--session", "parity-s1"],
    &["context", "--session", "parity-s1"],
    &["show", "W-13", "--all"],
    &["ls"],
    &["ls", "--open", "--guess"],
    &["templates"],
];

fn sandbox(inside: bool) -> Sandbox {
    let mut s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    if inside {
        let sc = s.scenario_mut();
        *sc = sc
            .clone()
            .herdr("w4V:p9", "w4V:t1", "w4V")
            .fake_scenario("pane-session");
    }
    s
}

/// Runs the flow and returns each step's exit code and redacted stdout.
fn flow(s: &Sandbox) -> Vec<(Option<i32>, String)> {
    FLOW.iter()
        .map(|args| {
            let mut c = s.cmd(args);
            if args[0] == "park" {
                c.write_stdin("Doing: a\nNext: b\nWatch out: c\n");
            }
            let out = c.output().unwrap();
            let text = String::from_utf8(out.stdout).unwrap();
            (out.status.code(), s.redact(&text))
        })
        .collect()
}

fn mask(text: &str) -> String {
    [
        " (pane w4V:p9)",
        ", linked here",
        ", same pane",
        ", same folder",
    ]
    .iter()
    .fold(text.to_string(), |t, m| t.replace(m, ""))
}

/// The card IDs (`W-<n>`) named in `text`.
fn ids(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .filter(|w| {
            w.strip_prefix("W-")
                .is_some_and(|n| n.parse::<u32>().is_ok())
        })
        .map(String::from)
        .collect()
}

#[test]
fn nfr05_cli_flow_exit_codes_match_inside_and_outside_herdr() {
    for inside in [false, true] {
        let s = sandbox(inside);
        for (args, (code, out)) in FLOW.iter().zip(flow(&s)) {
            assert_eq!(code, Some(0), "inside={inside} {args:?}: {out}");
        }
    }
}

#[test]
fn nfr05_cli_flow_output_matches_after_masking_the_pane() {
    let outside = flow(&sandbox(false));
    let inside = flow(&sandbox(true));
    for ((args, (_, o)), (_, i)) in FLOW.iter().zip(&outside).zip(&inside) {
        if args == &["ls", "--open", "--guess"] {
            assert_eq!(ids(o), ids(i), "{args:?}");
        } else {
            assert_eq!(mask(o), mask(i), "{args:?}");
        }
    }
}

#[test]
fn nfr05_outside_herdr_runs_no_herdr_command() {
    let s = sandbox(false);
    flow(&s);
    let lines = "not in herdr\nWhere: <sandbox>/elsewhere\n";
    let out = s.cmd(&["locate", "W-13"]).assert().code(0);
    assert_eq!(
        s.redact(&String::from_utf8_lossy(&out.get_output().stdout)),
        lines
    );
    let out = s.cmd(&["jump", "W-13"]).assert().code(6);
    assert_eq!(
        s.redact(&String::from_utf8_lossy(&out.get_output().stdout)),
        lines
    );
    assert!(calls(&s.root().join("home")).is_empty());
}
