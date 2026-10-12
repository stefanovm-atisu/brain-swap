//! TECHSPEC 6.2: usage errors exit 2 and `--help` documents the exit codes.

mod common;

use common::cli::Sandbox;

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s
}

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn first_stderr_line(s: &Sandbox, args: &[&str]) -> String {
    let out = s.cmd(args).assert().code(2).get_output().clone();
    let err = text(&out.stderr);
    let first = err.lines().next().unwrap_or("").to_string();
    assert!(first.starts_with("brain-swap: "), "{err}");
    first
}

#[test]
fn ts6_2_unknown_subcommand_exits_2() {
    let s = sandbox();
    let out = s
        .cmd(&["frobnicate"])
        .assert()
        .code(2)
        .stdout("")
        .get_output()
        .clone();
    let err = text(&out.stderr);
    let first = err.lines().next().unwrap_or("");
    assert!(first.starts_with("brain-swap: "), "{err}");
    assert!(first.contains("frobnicate"), "{err}");
    insta::assert_snapshot!(s.redact(&err));
}

#[test]
fn ts6_2_missing_argument_exits_2() {
    first_stderr_line(&sandbox(), &["move", "W-12"]);
}

#[test]
fn ts6_2_invalid_card_id_exits_2() {
    sandbox().cmd(&["show", "W12"]).assert().code(2);
}

#[test]
fn ts6_2_usage_error_json_envelope() {
    let s = sandbox();
    let out = s
        .cmd(&["--json", "frobnicate"])
        .assert()
        .code(2)
        .stderr("")
        .get_output()
        .clone();
    let stdout = text(&out.stdout);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["v"], 1);
    assert_eq!(v["error"]["code"], "usage");
    assert!(v["error"]["message"].is_string(), "{stdout}");
    insta::assert_snapshot!(s.redact(&stdout));
}

#[test]
fn fr36_help_documents_exit_codes() {
    let s = sandbox();
    let out = s.cmd(&["--help"]).assert().code(0).get_output().clone();
    let stdout = text(&out.stdout);
    let tail: Vec<&str> = stdout.trim_end().lines().rev().take(7).collect();
    let want = [
        "0 success",
        "1 runtime error",
        "2 usage",
        "3 not found",
        "4 no reference",
        "5 busy",
        "6 jump not performed",
    ];
    for (line, w) in tail.iter().rev().zip(want) {
        assert!(line.trim_start().starts_with(w), "{w}: {stdout}");
    }
    insta::assert_snapshot!(s.redact(&stdout));
}
