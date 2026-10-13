//! TECHSPEC 6.3 `context` with a reference (FR-37) and its 6.7 JSON.

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
