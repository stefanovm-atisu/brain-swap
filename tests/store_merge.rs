//! TECHSPEC 4.7, 4.8 R6, 7.6 and 11: merging an editor's bytes with notes parked meanwhile.

mod common;

use std::fs;
use std::path::PathBuf;

use brain_swap::core::board::load;
use brain_swap::core::card_file::render_note;
use brain_swap::core::env::Env;
use brain_swap::core::error::Error;
use brain_swap::core::model::{Board, CardId, NewNote};
use brain_swap::core::store::merge_edit;
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);

/// A temp home holding a copy of `work_4_9` at `<home>/work`, its env, its board and `W-12.md`.
fn copy() -> (tempfile::TempDir, Env, Board, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    for e in fs::read_dir(WORK_4_9).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    let env = core_env(tmp.path(), NOW);
    let board = load("work", &dir, &env).unwrap();
    let p = dir.join("W-12.md");
    (tmp, env, board, p)
}

fn w12() -> CardId {
    CardId::parse("W-12").unwrap()
}

/// A rendered note block, starting with its blank separator line.
fn block(doing: &str) -> String {
    let n = NewNote {
        at: parse_stamp(NOW, &TimeZone::UTC).unwrap(),
        auto: false,
        doing: doing.into(),
        next: "rerun the migration test".into(),
        watch_out: String::new(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: None,
        session: None,
    };
    render_note(&n, "\n").0
}

const TITLE: &str = "# migrate invoices to v13\n";
const NEW_TITLE: &str = "# migrate invoices to v13 (edited)\n";

#[test]
fn r6_unchanged_file_gets_users_bytes() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let edited = c0.replace(TITLE, NEW_TITLE);
    let m = merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    assert!(m.written);
    assert_eq!(fs::read_to_string(&p).unwrap(), edited);
}

#[test]
fn r6_note_parked_meanwhile_survives_edit() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let b = block("parked meanwhile");
    fs::write(&p, format!("{c0}{b}")).unwrap();
    let edited = c0.replace(TITLE, NEW_TITLE);
    let m = merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    assert_eq!(m.added_notes, 1);
    assert_eq!(fs::read_to_string(&p).unwrap(), format!("{edited}{b}"));
}

#[test]
fn r6_two_notes_in_one_second_are_both_kept() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let (b1, b2) = (block("first"), block("second"));
    fs::write(&p, format!("{c0}{b1}{b2}")).unwrap();
    let edited = c0.replace(TITLE, NEW_TITLE);
    let m = merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    assert_eq!(m.added_notes, 2);
    assert_eq!(fs::read_to_string(&p).unwrap(), format!("{edited}{b1}{b2}"));
}

#[test]
fn r6_note_deleted_by_user_stays_deleted() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let first = "\n### 2026-10-08T10:31:05+03:00 auto\n";
    let start = c0.find(first).unwrap();
    let end = c0[start + 1..].find("\n### ").unwrap() + start + 1;
    let edited = format!("{}{}", &c0[..start], &c0[end..]);
    let b = block("new one");
    fs::write(&p, format!("{c0}{b}")).unwrap();
    merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    let after = fs::read_to_string(&p).unwrap();
    assert!(!after.contains("2026-10-08T10:31:05+03:00 auto"), "{after}");
    assert_eq!(after, format!("{edited}{b}"));
}

#[test]
fn r6_file_column_kept_when_user_left_column() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    fs::write(&p, c0.replace("column: Doing", "column: Done")).unwrap();
    let edited = c0.replace(TITLE, NEW_TITLE);
    merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    let after = fs::read_to_string(&p).unwrap();
    assert_eq!(after, edited.replace("column: Doing", "column: Done"));
}

#[test]
fn r6_user_column_change_wins() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    fs::write(&p, c0.replace("column: Doing", "column: Done")).unwrap();
    let edited = c0.replace("column: Doing", "column: Todo");
    merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    let after = fs::read_to_string(&p).unwrap();
    assert!(after.contains("column: Todo\n"), "{after}");
    assert!(!after.contains("column: Done"), "{after}");
}

#[test]
fn r6_missing_timeline_in_edit_is_readded_for_merged_notes() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let edited = c0[..c0.find("## Timeline\n").unwrap()].to_string();
    let b = block("parked meanwhile");
    fs::write(&p, format!("{c0}{b}")).unwrap();
    merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    let after = fs::read_to_string(&p).unwrap();
    assert!(after.starts_with(&edited), "{after}");
    assert!(after.ends_with(&format!("## Timeline\n{b}")), "{after}");
}

#[test]
fn r6_no_change_writes_nothing() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let f = format!("{c0}{}", block("parked meanwhile"));
    fs::write(&p, &f).unwrap();
    let m = merge_edit(&env, &board, w12(), c0.as_bytes(), c0.as_bytes()).unwrap();
    assert!(!m.written);
    assert_eq!(fs::read_to_string(&p).unwrap(), f);
}

#[test]
fn ts11_merge_is_logged() {
    let (tmp, mut env, board, p) = copy();
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    let c0 = fs::read_to_string(&p).unwrap();
    fs::write(&p, format!("{c0}{}", block("parked meanwhile"))).unwrap();
    let edited = c0.replace(TITLE, NEW_TITLE);
    merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap();
    let text = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "{text}");
    assert!(lines[0].ends_with(" merge W-12 +1 notes"), "{text}");
}

#[test]
fn ts7_6_read_only_edit_with_notes_to_merge_writes_nothing() {
    let (_tmp, env, board, p) = copy();
    let c0 = fs::read_to_string(&p).unwrap();
    let f = format!("{c0}{}", block("parked meanwhile"));
    fs::write(&p, &f).unwrap();
    let edited = c0.replace(TITLE, &format!("{TITLE}<<<<<<< HEAD\n"));
    let err = merge_edit(&env, &board, w12(), c0.as_bytes(), edited.as_bytes()).unwrap_err();
    assert!(matches!(err, Error::Unreadable(_)), "{err:?}");
    assert_eq!(fs::read_to_string(&p).unwrap(), f);
}
