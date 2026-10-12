//! TECHSPEC 6.2, 10 and 11: the config prelude every board subcommand runs first.

mod common;

use brain_swap::cli::cmd::Ctx;
use common::cli::Sandbox;
use common::env::core_env;
use std::fs;

const BROKEN: &str = "default_board = \"work\"\neditor = \"\"\n[boards]\nwork \"~/b\"\n";

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn with_config(config: &str) -> Sandbox {
    let s = Sandbox::new();
    let path = s.config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, config).unwrap();
    s
}

#[test]
fn ts6_2_broken_config_exits_1_naming_line_and_column() {
    let s = with_config(BROKEN);
    let out = s.cmd(&["ls"]).assert().code(1).get_output().clone();
    assert_eq!(text(&out.stdout), "");
    let err = s.redact(&text(&out.stderr));
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with("brain-swap: config <sandbox>/config/brain-swap/config.toml:4:"),
        "{err}"
    );
    insta::assert_snapshot!(err);
}

#[test]
fn ts6_2_broken_config_json_envelope() {
    let s = with_config(BROKEN);
    let out = s
        .cmd(&["--json", "ls"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert_eq!(text(&out.stderr), "");
    let stdout = s.redact(&text(&out.stdout));
    assert!(
        stdout.starts_with(
            "{\"v\":1,\"error\":{\"code\":\"config\",\"message\":\"config <sandbox>/config/brain-swap/config.toml:4:"
        ),
        "{stdout}"
    );
    insta::assert_snapshot!(stdout);
}

#[test]
fn ts6_2_invalid_default_board_exits_1() {
    let s = with_config("default_board = \"nope\"\n[boards]\nwork = \"~/w\"\n");
    let out = s.cmd(&["ls"]).assert().code(1).get_output().clone();
    let err = s.redact(&text(&out.stderr));
    assert!(
        err.starts_with("brain-swap: config <sandbox>/config/brain-swap/config.toml:1:"),
        "{err}"
    );
    let out = s
        .cmd(&["--json", "ls"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let stdout = s.redact(&text(&out.stdout));
    assert!(
        stdout.contains(
            "\"code\":\"config\",\"message\":\"config <sandbox>/config/brain-swap/config.toml:1:"
        ),
        "{stdout}"
    );
}

#[test]
fn ts10_broken_config_touches_no_board() {
    let s = with_config(BROKEN);
    s.cmd(&["new", "x"]).assert().code(1);
    assert!(!s.root().join("home/.local/share/brain-swap").exists());
    assert!(!s.root().join("data/brain-swap").exists());
    assert!(!s.root().join("boards").exists());
}

#[test]
fn ts11_ctx_load_drops_log_inside_board_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), "2026-10-08T11:52:00+03:00");
    let path = env.config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let work = tmp.path().join("boards/work");
    fs::write(
        &path,
        format!(
            "default_board = \"work\"\n[boards]\nwork = \"{}\"\n",
            work.display()
        ),
    )
    .unwrap();
    let warning = "BRAIN_SWAP_LOG inside board work, ignored".to_string();

    env.log = Some(work.join("bs.log"));
    let ctx = Ctx::load(&mut env).unwrap();
    assert_eq!(env.log, None);
    assert!(ctx.warnings.contains(&warning), "{:?}", ctx.warnings);

    let outside = tmp.path().join("bs.log");
    env.log = Some(outside.clone());
    let ctx = Ctx::load(&mut env).unwrap();
    assert_eq!(env.log, Some(outside));
    assert!(!ctx.warnings.contains(&warning), "{:?}", ctx.warnings);
}
