//! TECHSPEC 4.7, 4.8 R3, R7 and FR-16: parking a switch note through the store.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use brain_swap::core::board::load;
use brain_swap::core::card_file::{append_note, parse, render_note};
use brain_swap::core::env::Env;
use brain_swap::core::error::Error;
use brain_swap::core::model::{Board, Card, CardId, NewNote};
use brain_swap::core::store::{park, temp_path, verify_park};
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);
const CARDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cards");
const READ_ONLY: [&str; 3] = [
    "conflict_markers.md",
    "invalid_frontmatter.md",
    "non_utf8.md",
];

/// A temp home holding a copy of `work_4_9` at `<home>/work`, its env and its board.
fn copy() -> (tempfile::TempDir, Env, Board) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    for e in fs::read_dir(WORK_4_9).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    let env = core_env(tmp.path(), NOW);
    let board = load("work", &dir, &env).unwrap();
    (tmp, env, board)
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn path(board: &Board, s: &str) -> PathBuf {
    board.path.join(format!("{s}.md"))
}

fn note(doing: &str) -> NewNote {
    NewNote {
        at: parse_stamp(NOW, &TimeZone::UTC).unwrap(),
        auto: false,
        doing: doing.into(),
        next: "rerun the migration test".into(),
        watch_out: String::new(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: None,
        session: None,
    }
}

fn card(bytes: &[u8]) -> Card {
    parse(
        id("W-12"),
        Path::new("W-12.md"),
        bytes,
        "Todo",
        &TimeZone::UTC,
    )
    .card
}

#[test]
fn fr16_park_appends_only_to_timeline() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-12");
    let before = fs::read(&p).unwrap();
    let n = note("switched to v13");
    park(&env, &board, id("W-12"), &n).unwrap();
    let after = fs::read(&p).unwrap();
    let block = render_note(&n, "\n").0;
    assert_eq!(after, [&before[..], block.as_bytes()].concat());
    let (b, a) = (card(&before), card(&after));
    assert_eq!(a.notes.len(), b.notes.len() + 1);
    assert_eq!(
        (&a.title, &a.column, &a.body),
        (&b.title, &b.column, &b.body)
    );
}

#[test]
fn fr16_park_never_moves_card() {
    let (_tmp, env, board) = copy();
    let parked = park(&env, &board, id("W-11"), &note("a")).unwrap();
    assert_eq!(parked.card.column, "Todo");
    let reloaded = load("work", &board.path, &env).unwrap();
    assert_eq!(reloaded.card(id("W-11")).unwrap().column, "Todo");
}

#[test]
fn ts4_7_park_readds_missing_timeline() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-11");
    let text = fs::read_to_string(&p).unwrap().replace("## Timeline\n", "");
    fs::write(&p, &text).unwrap();
    let n = note("a");
    park(&env, &board, id("W-11"), &n).unwrap();
    let after = fs::read_to_string(&p).unwrap();
    let block = render_note(&n, "\n").0;
    assert!(after.ends_with(&format!("## Timeline\n{block}")), "{after}");
    assert!(after.starts_with(&text), "{after}");
}

#[test]
fn fr18_park_with_empty_note_writes_nothing() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-12");
    let before = fs::read(&p).unwrap();
    let n = NewNote {
        doing: String::new(),
        next: " ".into(),
        watch_out: String::new(),
        ..note("")
    };
    let err = park(&env, &board, id("W-12"), &n).unwrap_err();
    assert_eq!(err, Error::InvalidInput("empty note".into()));
    assert_eq!(fs::read(&p).unwrap(), before);
}

#[test]
fn ts6_2_park_on_missing_card_is_not_found() {
    let (_tmp, env, board) = copy();
    let err = park(&env, &board, id("W-99"), &note("a")).unwrap_err();
    assert_eq!(err, Error::NotFound("W-99 not found".into()));
    assert!(!path(&board, "W-99").exists());
}

#[test]
fn ts4_7_park_on_read_only_card_is_unreadable() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-12");
    fs::copy(Path::new(CARDS).join("invalid_frontmatter.md"), &p).unwrap();
    let before = fs::read(&p).unwrap();
    let err = park(&env, &board, id("W-12"), &note("a")).unwrap_err();
    assert!(matches!(err, Error::Unreadable(_)), "{err:?}");
    assert_eq!(fs::read(&p).unwrap(), before);
    assert!(!temp_path(&p, env.pid).exists());
}

#[test]
fn nfr08_two_parks_on_one_card_append_both() {
    let (_tmp, env, board) = copy();
    let before = card(&fs::read(path(&board, "W-12")).unwrap());
    park(&env, &board, id("W-12"), &note("first")).unwrap();
    let parked = park(&env, &board, id("W-12"), &note("second")).unwrap();
    let after = card(&fs::read(path(&board, "W-12")).unwrap());
    assert_eq!(
        after,
        Card {
            path: after.path.clone(),
            ..parked.card.clone()
        }
    );
    assert_eq!(after.notes.len(), before.notes.len() + 2);
    let doing: Vec<&str> = after
        .notes
        .iter()
        .rev()
        .take(2)
        .map(|n| n.doing.as_str())
        .collect();
    assert_eq!(doing, ["second", "first"]);
}

#[test]
fn r7_verify_park_accepts_every_writable_fixture() {
    let mut names: Vec<String> = fs::read_dir(CARDS)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| !READ_ONLY.contains(&n.as_str()))
        .collect();
    names.sort();
    assert!(!names.is_empty());
    for name in names {
        let before = fs::read(Path::new(CARDS).join(&name)).unwrap();
        let a = append_note(&name, &before, &note("a")).unwrap();
        assert!(verify_park(&before, &a.bytes, a.inserted.clone()), "{name}");
    }
}

#[test]
fn r7_verify_park_rejects_changed_byte_outside_block() {
    let before = fs::read(Path::new(CARDS).join("w12_4_9.md")).unwrap();
    let a = append_note("w12_4_9.md", &before, &note("a")).unwrap();
    assert!(verify_park(&before, &a.bytes, a.inserted.clone()));
    let mut changed = a.bytes.clone();
    changed[a.inserted.start - 2] ^= 1;
    assert!(!verify_park(&before, &changed, a.inserted.clone()));
    let mut no_note = before.clone();
    no_note.extend_from_slice(b"\nplain line\n");
    let range = before.len()..no_note.len();
    assert!(!verify_park(&before, &no_note, range));
}
