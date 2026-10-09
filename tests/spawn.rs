//! TECHSPEC 12.1: the hermetic spawn helper.

mod common;

use common::spawn::{FAKE_HERDR, Scenario, command};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The pairs `/usr/bin/env` prints when spawned through `command`.
fn child_env(s: &Scenario) -> BTreeMap<String, String> {
    let out = command(s, Path::new("/usr/bin/env")).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| {
            let (k, v) = l.split_once('=').unwrap();
            (k.to_string(), v.to_string())
        })
        .collect()
}

fn defaults(tmp: &Path) -> BTreeMap<String, String> {
    let home = tmp.join("home");
    let h = |p: &str| home.join(p).to_string_lossy().into_owned();
    [
        ("PATH", "/usr/bin:/bin".to_string()),
        ("HOME", home.to_string_lossy().into_owned()),
        ("XDG_CONFIG_HOME", h(".config")),
        ("XDG_DATA_HOME", h(".local/share")),
        ("XDG_STATE_HOME", h(".local/state")),
        ("TZ", "UTC".to_string()),
        ("BRAIN_SWAP_NOW", "2026-10-08T11:52:00+03:00".to_string()),
        ("HERDR_BIN_PATH", FAKE_HERDR.to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

#[test]
fn ts12_1_child_sees_only_listed_variables() {
    let tmp = tempfile::tempdir().unwrap();
    let got = child_env(&Scenario::new(tmp.path()));
    let mut names: Vec<&str> = got.keys().map(String::as_str).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "BRAIN_SWAP_NOW",
            "HERDR_BIN_PATH",
            "HOME",
            "PATH",
            "TZ",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME"
        ]
    );
    assert_eq!(got, defaults(tmp.path()));
}

#[test]
fn ts12_1_herdr_setter_adds_herdr_variables() {
    let tmp = tempfile::tempdir().unwrap();
    let got = child_env(&Scenario::new(tmp.path()).herdr("w4V:p9", "w4V:t1", "w4V"));
    let mut want = defaults(tmp.path());
    for (k, v) in [
        ("HERDR_ENV", "1"),
        ("HERDR_PANE_ID", "w4V:p9"),
        ("HERDR_TAB_ID", "w4V:t1"),
        ("HERDR_WORKSPACE_ID", "w4V"),
    ] {
        want.insert(k.to_string(), v.to_string());
    }
    assert_eq!(got, want);
}

#[test]
fn ts12_1_optional_variables_only_when_set() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let new = || Scenario::new(t);
    let log = t.join("log.txt");
    let cases: Vec<(Scenario, &str, &str)> = vec![
        (new().fake_scenario("idle"), "FAKE_HERDR_SCENARIO", "idle"),
        (
            new().failpoint("create:after_tmp"),
            "BRAIN_SWAP_FAILPOINT",
            "create:after_tmp",
        ),
        (new().session("s1"), "BRAIN_SWAP_SESSION", "s1"),
        (new().log(&log), "BRAIN_SWAP_LOG", log.to_str().unwrap()),
        (new().visual("vi"), "VISUAL", "vi"),
        (new().editor("nano"), "EDITOR", "nano"),
    ];
    for (s, k, v) in cases {
        let mut want = defaults(t);
        want.insert(k.to_string(), v.to_string());
        assert_eq!(child_env(&s), want, "{k}");
    }

    let s = new().xdg(&t.join("c"), &t.join("d"), &t.join("s"));
    let mut want = defaults(t);
    for (k, d) in [
        ("XDG_CONFIG_HOME", "c"),
        ("XDG_DATA_HOME", "d"),
        ("XDG_STATE_HOME", "s"),
    ] {
        want.insert(k.to_string(), t.join(d).to_string_lossy().into_owned());
    }
    assert_eq!(child_env(&s), want);

    let mut want = defaults(t);
    want.remove("XDG_CONFIG_HOME");
    assert_eq!(child_env(&new().unset("XDG_CONFIG_HOME")), want);

    let mut want = defaults(t);
    want.insert("PATH".to_string(), "/opt/bin:/usr/bin:/bin".to_string());
    assert_eq!(child_env(&new().path("/opt/bin:/usr/bin:/bin")), want);
}

#[test]
fn ts12_1_scenario_creates_home() {
    let tmp = tempfile::tempdir().unwrap();
    Scenario::new(tmp.path());
    let entries: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries, ["home"]);
    assert!(tmp.path().join("home").is_dir());
}

#[test]
fn ts12_1_vars_match_child_environment() {
    let tmp = tempfile::tempdir().unwrap();
    let s = Scenario::new(tmp.path())
        .herdr("w4V:p9", "w4V:t1", "w4V")
        .session("s1")
        .unset("XDG_STATE_HOME");
    let got: Vec<(String, String)> = child_env(&s).into_iter().collect();
    assert_eq!(s.vars(), got);
}

#[test]
#[should_panic(expected = "HERDR_BIN_PATH")]
fn ts12_1_unset_refuses_non_xdg_names() {
    let tmp = tempfile::tempdir().unwrap();
    Scenario::new(tmp.path()).unset("HERDR_BIN_PATH");
}
