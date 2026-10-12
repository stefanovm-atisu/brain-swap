//! TECHSPEC 9.1: `ProcessRunner` starts the herdr binary with plain arguments; 11: each call is logged.

mod common;

use brain_swap::adapters::herdr::CALL_TIMEOUT;
use brain_swap::adapters::runner::{ProcessRunner, RunOutcome, RunResult, Runner};
use brain_swap::core::env::Env;
use common::env::core_env;
use common::spawn::{Scenario, command};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

const NOW: &str = "2030-01-01T00:00:00+00:00";

/// Runs `args` against an `sh` script with `body`, written into `tmp` and named through `herdr_bin_path`.
fn run_script(tmp: &Path, body: &str, args: &[&str]) -> RunResult {
    run_timed(tmp, body, args, Duration::from_secs(2))
}

fn run_timed(tmp: &Path, body: &str, args: &[&str], timeout: Duration) -> RunResult {
    run_env(&script_env(tmp, body, NOW), args, timeout)
}

/// An `Env` at `now` whose `herdr_bin_path` names an `sh` script with `body`, written into `tmp`.
fn script_env(tmp: &Path, body: &str, now: &str) -> Env {
    let path = tmp.join("herdr");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let mut env = core_env(tmp, now);
    env.herdr_bin_path = Some(path);
    env
}

fn run_env(env: &Env, args: &[&str], timeout: Duration) -> RunResult {
    // A sibling test's fork can briefly inherit our write fd of the script: exec then fails with ETXTBSY.
    for _ in 0..50 {
        // A log that does not exist yet counts as empty, so its busy line is trimmed too.
        let logged = env
            .log
            .as_ref()
            .map(|l| fs::metadata(l).map_or(0, |m| m.len()));
        let r = ProcessRunner::new(env).run(args, timeout);
        if !matches!(&r.outcome, RunOutcome::SpawnFailed(e) if e.contains("os error 26")) {
            return r;
        }
        // Drop the busy attempt's log line, so a retry leaves one line per call.
        if let (Some(log), Some(len)) = (&env.log, logged) {
            let _ = fs::OpenOptions::new()
                .write(true)
                .open(log)
                .and_then(|f| f.set_len(len));
        }
    }
    panic!("script stayed busy (ETXTBSY) for 50 attempts");
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

/// Runs `sh -c <script>` through the 12.1 spawn helper and reports whether it exited 0.
fn sh(tmp: &Path, script: &str) -> bool {
    command(&Scenario::new(tmp), Path::new("/bin/sh"))
        .args(["-c", script])
        .status()
        .unwrap()
        .success()
}

fn read_pid(path: &Path) -> String {
    fs::read_to_string(path).unwrap().trim().to_string()
}

#[test]
fn ts9_3_hung_child_is_killed_at_2s() {
    let tmp = tempfile::tempdir().unwrap();
    let pid = tmp.path().join("pid");
    let body = format!("echo $$ > {}\nexec sleep 30", pid.display());
    let r = run_timed(tmp.path(), &body, &["pane", "get"], CALL_TIMEOUT);
    assert_eq!(r.outcome, RunOutcome::TimedOut);
    assert!(
        r.took >= Duration::from_secs(2) && r.took <= Duration::from_millis(2500),
        "{:?}",
        r.took
    );
    assert!(!sh(
        tmp.path(),
        &format!("kill -0 {} 2>/dev/null", read_pid(&pid))
    ));
}

#[test]
fn ts9_3_shorter_timeout_is_honoured() {
    let tmp = tempfile::tempdir().unwrap();
    let pid = tmp.path().join("pid");
    let body = format!("echo $$ > {}\nexec sleep 30", pid.display());
    let r = run_timed(
        tmp.path(),
        &body,
        &["pane", "get"],
        Duration::from_millis(200),
    );
    assert_eq!(r.outcome, RunOutcome::TimedOut);
    assert!(
        r.took >= Duration::from_millis(200) && r.took <= Duration::from_millis(700),
        "{:?}",
        r.took
    );
}

#[test]
fn ts9_3_quick_child_is_not_cut() {
    let tmp = tempfile::tempdir().unwrap();
    let r = run_timed(tmp.path(), "exit 0", &["pane", "get"], CALL_TIMEOUT);
    assert!(
        matches!(r.outcome, RunOutcome::Exited { code: Some(0), .. }),
        "{r:?}"
    );
    assert!(r.took < Duration::from_secs(1), "{:?}", r.took);
}

#[test]
fn ts9_3_grandchild_holding_pipe_does_not_block() {
    let tmp = tempfile::tempdir().unwrap();
    let bg = tmp.path().join("bg");
    let body = format!("sleep 30 &\necho $! > {}\nexec sleep 30", bg.display());
    let r = run_timed(
        tmp.path(),
        &body,
        &["pane", "get"],
        Duration::from_millis(200),
    );
    let killed = sh(tmp.path(), &format!("kill {}", read_pid(&bg)));
    assert_eq!(r.outcome, RunOutcome::TimedOut);
    assert!(r.took <= Duration::from_millis(700), "{:?}", r.took);
    assert!(killed);
}

#[test]
fn ts9_3_exited_child_with_background_holder_does_not_block() {
    let tmp = tempfile::tempdir().unwrap();
    let bg = tmp.path().join("bg");
    let body = format!(
        "echo '{{}}'\nsleep 30 &\necho $! > {}\nexit 0",
        bg.display()
    );
    let r = run_timed(
        tmp.path(),
        &body,
        &["pane", "get"],
        Duration::from_millis(200),
    );
    sh(tmp.path(), &format!("kill {} 2>/dev/null", read_pid(&bg)));
    assert!(r.took <= Duration::from_millis(700), "{r:?}");
}

const LOG_NOW: &str = "2026-10-08T11:52:00+03:00";

fn log_lines(log: &Path) -> Vec<String> {
    fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

#[test]
fn ts11_runner_logs_argv_exit_and_ms() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = script_env(tmp.path(), "exit 3", LOG_NOW);
    env.log = Some(tmp.path().join("bs.log"));
    run_env(&env, &["pane", "get", "w4V:p9"], CALL_TIMEOUT);
    let lines = log_lines(&tmp.path().join("bs.log"));
    assert_eq!(lines.len(), 1, "{lines:?}");
    let prefix = "2026-10-08T11:52:00+03:00 herdr pane get w4V:p9 exit=3 ms=";
    let ms = lines[0].strip_prefix(prefix).expect(&lines[0]);
    ms.parse::<u64>().unwrap();
    // A log path that cannot be written never changes the result.
    env.log = Some(tmp.path().to_path_buf());
    let r = run_env(&env, &["pane", "get", "w4V:p9"], CALL_TIMEOUT);
    assert!(
        matches!(r.outcome, RunOutcome::Exited { code: Some(3), .. }),
        "{r:?}"
    );
}

#[test]
fn ts11_timed_out_call_is_logged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = script_env(tmp.path(), "exec sleep 30", LOG_NOW);
    env.log = Some(tmp.path().join("bs.log"));
    run_env(&env, &["pane", "get"], Duration::from_millis(200));
    let lines = log_lines(&tmp.path().join("bs.log"));
    assert_eq!(lines.len(), 1, "{lines:?}");
    let (_, ms) = lines[0].rsplit_once(" exit=timeout ms=").expect(&lines[0]);
    assert!(ms.parse::<u64>().unwrap() >= 200, "{ms}");
}

#[test]
fn ts11_spawn_failure_is_logged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), LOG_NOW);
    env.herdr_bin_path = Some(tmp.path().join("nope"));
    env.log = Some(tmp.path().join("bs.log"));
    ProcessRunner::new(&env).run(&["workspace", "list"], CALL_TIMEOUT);
    let lines = log_lines(&tmp.path().join("bs.log"));
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].contains("herdr workspace list exit=spawn_failed ms="),
        "{}",
        lines[0]
    );
}

#[test]
fn ts11_each_call_adds_one_line() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = script_env(tmp.path(), "exit 0", LOG_NOW);
    env.log = Some(tmp.path().join("bs.log"));
    run_env(&env, &["pane", "get", "w4V:p9"], CALL_TIMEOUT);
    run_env(&env, &["workspace", "list"], CALL_TIMEOUT);
    let lines = log_lines(&tmp.path().join("bs.log"));
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].contains(" herdr pane get w4V:p9 exit=0 ms="),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains(" herdr workspace list exit=0 ms="),
        "{}",
        lines[1]
    );
}

#[test]
fn ts11_no_log_without_brain_swap_log() {
    let tmp = tempfile::tempdir().unwrap();
    let env = script_env(tmp.path(), "exit 0", LOG_NOW);
    assert_eq!(env.log, None);
    run_env(&env, &["pane", "get", "w4V:p9"], CALL_TIMEOUT);
    let names: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["herdr"]);
}
