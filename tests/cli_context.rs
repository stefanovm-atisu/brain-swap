//! TECHSPEC 6.3 `context` with a reference (FR-37), its picker without one (FR-56, FR-57) and its 6.7 JSON.

mod common;

use brain_swap::core::env::Env;
use brain_swap::core::model::{CardId, SessionRef};
use brain_swap::core::session::write_reference;
use brain_swap::core::time::parse_stamp;
use common::cli::Sandbox;
use jiff::tz::TimeZone;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const UUID: &str = "55583127-a22a-49bc-a803-e777c377a595";

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    s
}

/// Writes the reference of `session` to `card` on `board` through E1-F7.
fn refer(s: &mut Sandbox, session: &str, card: &str, board: &str) {
    let vars = s.scenario_mut().vars();
    let env = Env::from_vars(
        vars,
        s.root().to_path_buf(),
        std::process::id(),
        TimeZone::UTC,
        parse_stamp("2026-10-08T11:52:00+03:00", &TimeZone::UTC).unwrap(),
    )
    .unwrap();
    let r = SessionRef {
        card: CardId::parse(card).unwrap(),
        board: board.to_string(),
        pane: None,
        cwd: None,
        set_at: env.now.clone(),
    };
    write_reference(&env, session, &r).unwrap();
}

/// Every file under `dir` by path.
fn tree(dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            tree(&p, files);
        } else {
            files.insert(p.display().to_string(), fs::read(&p).unwrap());
        }
    }
}

/// Runs `args`, checks exit 0 and that nothing under the sandbox changed, returns stdout.
fn context(s: &Sandbox, args: &[&str]) -> String {
    let (mut before, mut after) = (BTreeMap::new(), BTreeMap::new());
    tree(s.root(), &mut before);
    let out = s.cmd(args).assert().code(0).get_output().stdout.clone();
    tree(s.root(), &mut after);
    assert_eq!(before, after, "context wrote");
    String::from_utf8(out).unwrap()
}

fn line(text: &str, n: usize) -> &str {
    text.lines().nth(n - 1).unwrap()
}

#[test]
fn fr37_context_with_reference_and_note() {
    let mut s = sandbox();
    refer(&mut s, UUID, "W-12", "work");
    let out = context(&s, &["context", "--session", UUID]);
    insta::assert_snapshot!(s.redact(&out));
}

#[test]
fn fr37_context_note_by_another_session() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-12", "work");
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(
        line(&out, 6),
        "latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by another session"
    );
}

#[test]
fn fr37_context_auto_note() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-7", "work");
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(
        line(&out, 6),
        "latest note: 2026-10-08T09:40:00+03:00, 2 h ago, auto, by another session"
    );
}

#[test]
fn fr37_context_reference_without_notes() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-11", "work");
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(out.lines().last(), Some("latest note: none"));
    insta::assert_snapshot!(s.redact(&out));
}

#[test]
fn fr37_context_note_without_session() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-9", "work");
    let out = context(&s, &["context", "--session", "s1"]);
    assert!(line(&out, 6).ends_with("1 d ago, no session"), "{out}");
}

#[test]
fn fr37_context_reference_on_another_board() {
    let mut s = sandbox();
    let home = s.root().join("boards/home");
    fs::create_dir_all(&home).unwrap();
    let board = "---\nletter: H\ncolumns: Todo, Doing, Done\nnext: 2\n---\n# home\n";
    fs::write(home.join("board.md"), board).unwrap();
    let card = "---\ncolumn: Todo\n---\n# water the plants\n\n## Timeline\n";
    fs::write(home.join("H-1.md"), card).unwrap();
    refer(&mut s, "s1", "H-1", "home");
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(line(&out, 3), "reference: H-1");
    assert_eq!(line(&out, 5), "board: home, column: Todo");
}

#[test]
fn ts6_7_context_json_with_reference() {
    let mut s = sandbox();
    refer(&mut s, UUID, "W-12", "work");
    let out = context(&s, &["--json", "context", "--session", UUID]);
    assert_eq!(out.lines().count(), 1, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let keys = [
        "v",
        "session",
        "reference",
        "card",
        "candidates",
        "other_boards",
        "problem",
        "warnings",
    ];
    assert_eq!(v.as_object().unwrap().len(), keys.len(), "{out}");
    // First occurrences: `session` also sits inside the note's place, after the top-level one.
    let at: Vec<usize> = keys
        .iter()
        .map(|k| out.find(&format!("\"{k}\":")).unwrap())
        .collect();
    assert!(at.is_sorted(), "{out}");
    assert_eq!(v["reference"], "W-12");
    assert_eq!(v["card"]["latest_note"]["by_this_session"], true);
    assert_eq!(v["candidates"], serde_json::json!([]));
    assert_eq!(v["other_boards"], serde_json::json!([]));
    assert!(v["problem"].is_null());
    assert_eq!(v["warnings"], serde_json::json!([]));
    insta::assert_snapshot!(s.redact(&out));
}

#[test]
fn ts6_7_context_json_note_without_session_is_null() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-9", "work");
    let out = context(&s, &["--json", "context", "--session", "s1"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["card"]["latest_note"].is_object(), "{out}");
    assert!(
        v["card"]["latest_note"]["by_this_session"].is_null(),
        "{out}"
    );
}

#[test]
fn fr56_context_without_reference_lists_picker() {
    let s = sandbox();
    let out = context(&s, &["context", "--session", "s9"]);
    let ls = context(&s, &["ls", "--open", "--guess"]);
    assert_eq!(
        out,
        format!("brain-swap context v1\nsession: s9\nreference: none\n{ls}")
    );
    assert_eq!(line(&out, 8), "o. other board: home, personal");
    insta::assert_snapshot!(s.redact(&out));
}

#[test]
fn fr56_context_deleted_card_reference() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-12", "work");
    fs::remove_file(s.root().join("boards/work/W-12.md")).unwrap();
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(line(&out, 3), "reference: none (W-12 not found)");
    assert!(line(&out, 5).starts_with("1. W-7 "), "{out}");
    assert!(line(&out, 6).starts_with("2. W-11 "), "{out}");
}

#[test]
fn fr56_context_reference_to_unconfigured_board() {
    let mut s = sandbox();
    refer(&mut s, "s1", "H-1", "home");
    let config = fs::read_to_string(s.config_file()).unwrap();
    let config: String = config
        .lines()
        .filter(|l| !l.starts_with("home "))
        .map(|l| format!("{l}\n"))
        .collect();
    fs::write(s.config_file(), config).unwrap();
    let out = context(&s, &["context", "--session", "s1"]);
    assert_eq!(line(&out, 3), "reference: none (H-1 not found)");
}

#[test]
fn fr57_new_session_in_same_pane_sees_linked_here() {
    let mut s = sandbox();
    let scenario = s.scenario_mut();
    *scenario = scenario.clone().herdr("w4V:p11", "w4V:t1", "w4V");
    s.cmd(&["link", "W-11", "--session", "old"])
        .assert()
        .code(0);
    let out = context(&s, &["context", "--session", "new"]);
    assert_eq!(
        line(&out, 5),
        "1. W-11 invoice PDF shows the wrong VAT [Todo] no note, linked here"
    );
}

#[test]
fn fr56_context_board_flag_lists_named_board() {
    let s = sandbox();
    let home = s.root().join("boards/home");
    fs::create_dir_all(&home).unwrap();
    let board = "---\nletter: H\ncolumns: Todo, Doing, Done\nnext: 2\n---\n# home\n";
    fs::write(home.join("board.md"), board).unwrap();
    let card = "---\ncolumn: Todo\n---\n# water the plants\n\n## Timeline\n";
    fs::write(home.join("H-1.md"), card).unwrap();
    let out = context(&s, &["--board", "home", "context", "--session", "s9"]);
    assert_eq!(line(&out, 4), "open cards on home, best guess first:");
    assert_eq!(out.lines().last(), Some("o. other board: work, personal"));
}

#[test]
fn fr56_context_no_session_lists_picker_without_warning() {
    let s = sandbox();
    let out = s.cmd(&["context"]).assert().code(0).get_output().clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(line(&stdout, 2), "session: none");
    assert_eq!(line(&stdout, 3), "reference: none");
    assert!(
        !stdout.lines().any(|l| l.starts_with("warning:")),
        "{stdout}"
    );
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn ts6_7_context_json_candidates() {
    let mut s = sandbox();
    refer(&mut s, "s1", "W-12", "work");
    fs::remove_file(s.root().join("boards/work/W-12.md")).unwrap();
    let out = context(&s, &["--json", "context", "--session", "s1"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["reference"].is_null(), "{out}");
    assert!(v["card"].is_null(), "{out}");
    let c = v["candidates"].as_array().unwrap();
    let got: Vec<_> = c
        .iter()
        .map(|c| (c["id"].clone(), c["rank"].clone(), c["reason"].clone()))
        .collect();
    let want = serde_json::json!([["W-7", 1, "middle_column"], ["W-11", 2, "other"]]);
    assert_eq!(serde_json::json!(got), want, "{out}");
    assert_eq!(v["other_boards"], serde_json::json!(["home", "personal"]));
    assert!(v["problem"].is_null(), "{out}");
    assert_eq!(v["warnings"], serde_json::json!(["W-12 not found"]));
    insta::assert_snapshot!(s.redact(&out));
}
