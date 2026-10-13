//! TECHSPEC 6.3 `link`, 6.4 session resolution, 6.7 `link`.

mod common;

use brain_swap::core::env::Env;
use brain_swap::core::session::{read_pane_hint, read_reference, reference_path};
use common::cli::Sandbox;
use jiff::tz::TimeZone;
use std::fs;
use std::process::Output;

const UUID: &str = "55583127-a22a-49bc-a803-e777c377a595";

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    s
}

fn run(s: &Sandbox, args: &[&str], code: i32) -> Output {
    s.cmd(args).assert().code(code).get_output().clone()
}

/// The `Env` the sandbox's processes see, for reading references and hints through E1-F7.
fn sandbox_env(s: &mut Sandbox) -> Env {
    let vars = s.scenario_mut().vars();
    Env::from_vars(
        vars,
        s.root().to_path_buf(),
        std::process::id(),
        TimeZone::UTC,
        jiff::Zoned::now(),
    )
    .unwrap()
}

/// The `latest note:` line of `link <id> --session s1`.
fn latest_line(s: &Sandbox, id: &str) -> String {
    let out = text(&run(s, &["link", id, "--session", "s1"], 0).stdout);
    let line = out.lines().find(|l| l.starts_with("latest note: "));
    line.unwrap_or_else(|| panic!("{out}")).to_string()
}

#[test]
fn fr57_link_sets_reference_and_prints_block() {
    let mut s = sandbox();
    let out = s.redact(&text(
        &run(&s, &["link", "W-12", "--session", UUID], 0).stdout,
    ));
    assert_eq!(
        out,
        "linked to W-12: migrate invoices to v13\n\
         card: W-12 migrate invoices to v13\n\
         board: work, column: Doing\n\
         latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by this session\n\
         Doing: migration test passes on staging\n\
         Next: open the MR, ask the DBA for the v13 deploy list\n\
         Watch out: nothing new\n\
         Where: <sandbox>/ati.billing (pane w4V:p9)\n"
    );
    insta::assert_snapshot!(out);
    let env = sandbox_env(&mut s);
    let r = read_reference(&env, UUID).expect("reference");
    assert_eq!(r.card.to_string(), "W-12");
}

#[test]
fn ts6_3_link_by_another_session() {
    let s = sandbox();
    assert_eq!(
        latest_line(&s, "W-12"),
        "latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by another session"
    );
}

#[test]
fn ts6_3_link_note_without_session() {
    let s = sandbox();
    let line = latest_line(&s, "W-9");
    assert!(line.ends_with("1 d ago, no session"), "{line}");
}

#[test]
fn ts6_3_link_unknown_card_exits_3_changes_nothing() {
    let mut s = sandbox();
    run(&s, &["link", "W-12", "--session", "s1"], 0);
    let env = sandbox_env(&mut s);
    let path = reference_path(&env, "s1").unwrap();
    let before = fs::read(&path).unwrap();
    let out = run(&s, &["link", "W-99", "--session", "s1"], 3);
    let err = text(&out.stderr);
    assert_eq!(err, "brain-swap: W-99 not found\n");
    insta::assert_snapshot!("unknown_card_text", err);
    let out = run(&s, &["--json", "link", "W-99", "--session", "s1"], 3);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"not_found\""), "{stdout}");
    insta::assert_snapshot!("unknown_card_json", stdout);
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn fr41_link_without_session_exits_1() {
    let mut s = sandbox();
    let out = run(&s, &["link", "W-12"], 1);
    let err = text(&out.stderr);
    assert_eq!(err, "brain-swap: no session\n");
    insta::assert_snapshot!("no_session_text", err);
    let out = run(&s, &["--json", "link", "W-12"], 1);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"invalid_input\""), "{stdout}");
    insta::assert_snapshot!("no_session_json", stdout);
    let env = sandbox_env(&mut s);
    let files = fs::read_dir(env.sessions_dir())
        .into_iter()
        .flatten()
        .count();
    assert_eq!(files, 0);
}

#[test]
fn fr41_link_placeholder_session_outside_herdr_exits_1() {
    let s = sandbox();
    let out = run(
        &s,
        &["link", "W-12", "--session", "${CLAUDE_SESSION_ID}"],
        1,
    );
    assert_eq!(
        text(&out.stderr),
        "warning: session id not substituted\nbrain-swap: no session\n"
    );
}

#[test]
fn fr41_link_inside_herdr_uses_pane_session() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc
        .clone()
        .herdr("w4V:p9", "w4V:t1", "w4V")
        .fake_scenario("pane-session");
    run(&s, &["link", "W-11", "--session", ""], 0);
    let env = sandbox_env(&mut s);
    let r = read_reference(&env, "herdr-s1").expect("reference for herdr-s1");
    assert_eq!(r.card.to_string(), "W-11");
    let hint = read_pane_hint(&env, "w4V:p9").expect("hint for w4V:p9");
    assert_eq!(hint.card.to_string(), "W-11");
}

#[test]
fn ts6_7_link_json_returns_card() {
    let s = sandbox();
    let out = run(&s, &["--json", "link", "W-12", "--session", UUID], 0);
    let stdout = s.redact(&text(&out.stdout));
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        v["card"]["latest_note"]["by_this_session"], true,
        "{stdout}"
    );
    insta::assert_snapshot!(stdout);
}
