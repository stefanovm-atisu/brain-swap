//! TECHSPEC 9.1: `ProcessRunner` starts the herdr binary with plain arguments.

mod common;

use brain_swap::adapters::runner::{ProcessRunner, RunOutcome, RunResult, Runner};
use common::env::core_env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

const NOW: &str = "2030-01-01T00:00:00+00:00";

/// Runs `args` against an `sh` script with `body`, written into `tmp` and named through `herdr_bin_path`.
fn run_script(tmp: &Path, body: &str, args: &[&str]) -> RunResult {
    let path = tmp.join("herdr");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let mut env = core_env(tmp, NOW);
    env.herdr_bin_path = Some(path);
    ProcessRunner::new(&env).run(args, Duration::from_secs(2))
}

fn exited(code: i32, stdout: &str, stderr: &str) -> RunOutcome {
    RunOutcome::Exited {
        code: Some(code),
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    }
}

#[test]
fn ts9_1_runner_returns_exit_code_and_output() {
    let tmp = tempfile::tempdir().unwrap();
    let r = run_script(
        tmp.path(),
        "printf out; printf err >&2; exit 3",
        &["pane", "get", "w4V:p9"],
    );
    assert_eq!(r.outcome, exited(3, "out", "err"));
    assert!(!r.succeeded());
}

#[test]
fn ts9_1_exit_0_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let r = run_script(tmp.path(), r#"printf '{"result":{}}'"#, &["pane", "get"]);
    assert_eq!(r.outcome, exited(0, r#"{"result":{}}"#, ""));
    assert!(r.succeeded());
}

#[test]
fn ts9_1_runner_passes_arguments_without_shell() {
    let tmp = tempfile::tempdir().unwrap();
    let r = run_script(
        tmp.path(),
        r#"for a; do printf '%s\n' "$a"; done"#,
        &["pane", "get", "w 4V;$x"],
    );
    assert_eq!(r.outcome, exited(0, "pane\nget\nw 4V;$x\n", ""));
}

#[test]
fn ts9_1_program_is_herdr_bin_path_else_herdr() {
    let tmp = tempfile::tempdir().unwrap();
    let program = |bin: Option<&str>| {
        let mut env = core_env(tmp.path(), NOW);
        env.herdr_bin_path = bin.map(PathBuf::from);
        ProcessRunner::new(&env).program().to_path_buf()
    };
    assert_eq!(program(Some("/opt/h/herdr")), Path::new("/opt/h/herdr"));
    assert_eq!(program(None), Path::new("herdr"));
    assert_eq!(program(Some("")), Path::new("herdr"));
}

#[test]
fn ts9_1_missing_binary_is_spawn_failed() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    env.herdr_bin_path = Some(tmp.path().join("nope"));
    let r = ProcessRunner::new(&env).run(&["pane", "get"], Duration::from_secs(2));
    assert!(matches!(r.outcome, RunOutcome::SpawnFailed(_)), "{r:?}");
    assert!(!r.succeeded());
}

#[test]
fn ts9_1_child_stdin_is_null() {
    let tmp = tempfile::tempdir().unwrap();
    let r = run_script(tmp.path(), "cat >/dev/null; printf done", &["pane", "get"]);
    assert_eq!(r.outcome, exited(0, "done", ""));
    assert!(r.took < Duration::from_secs(1), "{:?}", r.took);
}
