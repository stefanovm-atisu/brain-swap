//! TECHSPEC 12.1: the fake herdr script answers canned JSON per `FAKE_HERDR_SCENARIO`
//! and logs argv to `$HOME/fake-herdr.log`.

mod common;

use common::fake_herdr::{calls, scenarios};
use common::spawn::{FAKE_HERDR, Scenario, command};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

fn run(s: &Scenario, args: &str) -> Output {
    command(s, Path::new(FAKE_HERDR))
        .args(args.split(' '))
        .output()
        .unwrap()
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut all = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            all.extend(files(&p));
        }
        all.push(p);
    }
    all.sort();
    all
}

fn pane(out: &Output) -> Value {
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["result"]["pane"].clone()
}

#[test]
fn ts12_1_fake_herdr_logs_each_call() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Scenario::new(tmp.path()).fake_scenario("pane-session");
    let home = tmp.path().join("home");
    let before = files(tmp.path());
    run(&s, "pane get w4V:p9");
    run(&s, "pane get w4V:p9");
    assert_eq!(calls(&home), ["pane get w4V:p9", "pane get w4V:p9"]);
    let mut want = before;
    want.push(home.join("fake-herdr.log"));
    want.sort();
    assert_eq!(files(tmp.path()), want);
}

#[test]
fn ts12_1_fake_herdr_answers_scenario_json() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        &Scenario::new(tmp.path()).fake_scenario("pane-session"),
        "pane get w4V:p9",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(pane(&out)["agent_session"]["value"], "herdr-s1");
}

#[test]
fn ts12_1_fake_herdr_pane_without_agent() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        &Scenario::new(tmp.path()).fake_scenario("pane-no-agent"),
        "pane get w4V:p9",
    );
    let p = pane(&out);
    assert!(p.get("agent_session").is_none());
    assert_eq!(p["agent_status"], "unknown");
}

#[test]
fn ts12_1_fake_herdr_unexpected_call_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        &Scenario::new(tmp.path()).fake_scenario("pane-session"),
        "pane get w4V:p1",
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "fake herdr: unexpected call: pane get w4V:p1\n"
    );
    assert_eq!(calls(&tmp.path().join("home")), ["pane get w4V:p1"]);
}

#[test]
fn ts12_1_fake_herdr_without_scenario_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(&Scenario::new(tmp.path()), "workspace list");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "fake herdr: no scenario\n"
    );
    assert_eq!(calls(&tmp.path().join("home")), ["workspace list"]);
}

#[test]
fn ts12_1_fake_herdr_unknown_scenario_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        &Scenario::new(tmp.path()).fake_scenario("nope"),
        "workspace list",
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "fake herdr: unknown scenario nope\n"
    );
}

#[test]
fn ts12_1_fake_herdr_hang_keeps_running() {
    let tmp = tempfile::tempdir().unwrap();
    let mut child = command(
        &Scenario::new(tmp.path()).fake_scenario("hang"),
        Path::new(FAKE_HERDR),
    )
    .args(["pane", "get", "w4V:p9"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let running = child.try_wait().unwrap().is_none();
    child.kill().unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(running);
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
}

#[test]
fn ts12_1_every_scenario_parses_as_sh() {
    let tmp = tempfile::tempdir().unwrap();
    let names = scenarios();
    for want in ["hang", "pane-no-agent", "pane-session"] {
        assert!(names.iter().any(|n| n == want), "missing scenario {want}");
    }
    let dir = Path::new(FAKE_HERDR).with_file_name("scenarios");
    for name in names {
        let status = command(&Scenario::new(tmp.path()), Path::new("/bin/sh"))
            .arg("-n")
            .arg(dir.join(format!("{name}.sh")))
            .status()
            .unwrap();
        assert!(status.success(), "{name}.sh");
    }
}
