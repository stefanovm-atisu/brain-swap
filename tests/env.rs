//! TECHSPEC 3, 5.1, 6.1, 9.1, 9.4, 10: the `Env` value and its paths.

mod common;

use brain_swap::core::env::{Env, encode_file_name};
use brain_swap::core::error::{Error, Result};
use brain_swap::core::time::{format_stamp, parse_stamp};
use common::env::core_env;
use common::spawn::{Scenario, brain_swap};
use jiff::tz::{TimeZone, offset};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn build_at(vars: &[(&str, &str)], tz: TimeZone, clock: &str) -> Result<Env> {
    let clock = parse_stamp(clock, &TimeZone::UTC).unwrap();
    Env::from_vars(
        vars.iter().map(|(k, v)| (k.to_string(), v.to_string())),
        PathBuf::from("/cwd"),
        42,
        tz,
        clock,
    )
}

fn build(vars: &[(&str, &str)]) -> Result<Env> {
    build_at(vars, TimeZone::UTC, "2030-01-01T00:00:00+00:00")
}

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn nfr09_xdg_defaults_under_home_on_every_platform() {
    let env = build(&[("HOME", "/h")]).unwrap();
    assert_eq!(env.config_file(), p("/h/.config/brain-swap/config.toml"));
    assert_eq!(env.templates_dir(), p("/h/.config/brain-swap/templates"));
    assert_eq!(
        env.default_board_dir("work"),
        p("/h/.local/share/brain-swap/work")
    );
    let state = p("/h/.local/state/brain-swap");
    assert_eq!(env.sessions_dir(), state.join("sessions"));
    assert_eq!(env.panes_dir(), state.join("panes"));
    assert_eq!(env.locks_dir(), state.join("locks"));
    assert_eq!(env.edit_dir(), state.join("edit"));
}

#[test]
fn ts5_1_absolute_xdg_values_override() {
    let env = build(&[
        ("HOME", "/h"),
        ("XDG_CONFIG_HOME", "/x/c"),
        ("XDG_DATA_HOME", "/x/d"),
        ("XDG_STATE_HOME", "/x/s"),
    ])
    .unwrap();
    assert_eq!(env.config_file(), p("/x/c/brain-swap/config.toml"));
    assert_eq!(env.templates_dir(), p("/x/c/brain-swap/templates"));
    assert_eq!(env.data_dir(), p("/x/d/brain-swap"));
    assert_eq!(env.default_board_dir("work"), p("/x/d/brain-swap/work"));
    assert_eq!(env.state_dir(), p("/x/s/brain-swap"));
    assert_eq!(env.locks_dir(), p("/x/s/brain-swap/locks"));
}

#[test]
fn ts5_1_relative_or_empty_xdg_is_ignored() {
    let env = build(&[
        ("HOME", "/h"),
        ("XDG_STATE_HOME", "rel"),
        ("XDG_DATA_HOME", ""),
    ])
    .unwrap();
    assert_eq!(env.state_dir(), p("/h/.local/state/brain-swap"));
    assert_eq!(env.data_dir(), p("/h/.local/share/brain-swap"));
}

#[test]
fn ts6_1_brain_swap_now_overrides_clock_in_local_zone() {
    let env = build(&[
        ("HOME", "/h"),
        ("BRAIN_SWAP_NOW", "2026-10-08T11:52:00+03:00"),
    ])
    .unwrap();
    assert_eq!(format_stamp(&env.now), "2026-10-08T08:52:00+00:00");
}

#[test]
fn ts6_1_invalid_brain_swap_now_is_invalid_input() {
    let e = build(&[("HOME", "/h"), ("BRAIN_SWAP_NOW", "tomorrow")]).unwrap_err();
    assert!(matches!(e, Error::InvalidInput(_)), "{e:?}");
    assert_eq!(e.exit_code(), 1);
}

#[test]
fn ts6_1_missing_home_is_io_error() {
    let e = build(&[]).unwrap_err();
    assert_eq!(e, Error::Io("HOME not set".to_string()));
}

#[test]
fn ts6_1_listed_variables_land_in_fields() {
    let env = build(&[
        ("HOME", "/h"),
        ("VISUAL", "vis"),
        ("EDITOR", "ed"),
        ("BRAIN_SWAP_SESSION", "sess"),
        ("BRAIN_SWAP_LOG", "/log"),
        ("CLAUDE_CONFIG_DIR", "/claude"),
        ("HERDR_ENV", "1"),
        ("HERDR_BIN_PATH", "/bin/herdr"),
        ("HERDR_PANE_ID", "pane"),
        ("HERDR_TAB_ID", "tab"),
        ("HERDR_WORKSPACE_ID", "ws"),
    ])
    .unwrap();
    let s = |v: &str| Some(v.to_string());
    assert_eq!(env.visual, s("vis"));
    assert_eq!(env.editor, s("ed"));
    assert_eq!(env.session, s("sess"));
    assert_eq!(env.log, Some(p("/log")));
    assert_eq!(env.claude_config_dir, Some(p("/claude")));
    assert_eq!(env.herdr_env, s("1"));
    assert_eq!(env.herdr_bin_path, Some(p("/bin/herdr")));
    assert_eq!(env.herdr_pane_id, s("pane"));
    assert_eq!(env.herdr_tab_id, s("tab"));
    assert_eq!(env.herdr_workspace_id, s("ws"));
}

#[test]
fn ts6_1_claude_session_variable_is_not_read() {
    let env = build(&[("HOME", "/h"), ("CLAUDE_SESSION_ID", "abc")]).unwrap();
    assert_eq!(env.session, None);
}

#[test]
fn ts9_1_herdr_active_only_when_herdr_env_is_1() {
    let active = |v: Option<&str>| {
        let mut vars = vec![("HOME", "/h")];
        vars.extend(v.map(|v| ("HERDR_ENV", v)));
        build(&vars).unwrap().herdr_active()
    };
    assert!(active(Some("1")));
    assert!(!active(Some("0")));
    assert!(!active(Some("true")));
    assert!(!active(None));
}

#[test]
fn ts10_encode_file_name_escapes_percent_then_slash() {
    assert_eq!(encode_file_name("/home/a%b/work"), "%2Fhome%2Fa%25b%2Fwork");
}

#[test]
fn ts3_clock_is_converted_to_local_zone() {
    let env = build_at(
        &[("HOME", "/h")],
        TimeZone::fixed(offset(3)),
        "2026-10-08T08:52:00+00:00",
    )
    .unwrap();
    assert_eq!(format_stamp(&env.now), "2026-10-08T11:52:00+03:00");
}

#[test]
fn ts3_core_env_keeps_offset_of_now() {
    let env = core_env(Path::new("/h"), "2026-10-08T11:52:00+03:00");
    assert_eq!(format_stamp(&env.now), "2026-10-08T11:52:00+03:00");
    assert_eq!(env.tz, TimeZone::fixed(offset(3)));
}

#[test]
fn ts9_4_from_vars_and_core_env_leave_exe_unset() {
    assert_eq!(build(&[("HOME", "/h")]).unwrap().exe, None);
    let env = core_env(Path::new("/h"), "2026-10-08T11:52:00+03:00");
    assert_eq!(env.exe, None);
}

#[test]
fn ts6_1_non_utf8_variable_does_not_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let out = brain_swap(&Scenario::new(tmp.path()))
        .env("OLDPWD", OsStr::from_bytes(b"/home/u/caf\xe9"))
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(101), "{out:?}");
}
