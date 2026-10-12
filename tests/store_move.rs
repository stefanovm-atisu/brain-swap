//! TECHSPEC 4.8 R2, R7, 6.2 and 10: moving a card between columns.

mod common;

use brain_swap::core::board::load;
use brain_swap::core::card_file::{check_writable, set_column};
use brain_swap::core::error::Error;
use brain_swap::core::model::{Board, CardId};
use brain_swap::core::store::{move_card, verify_move};
use common::bytes::assert_only_lines_changed;
use common::env::core_env;
use std::fs;
use std::path::{Path, PathBuf};

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);
const CARDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cards");

/// A temp home holding a copy of `work_4_9` at `<home>/work`.
fn copy() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    for e in fs::read_dir(WORK_4_9).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    (tmp, dir)
}

fn board(home: &Path, dir: &Path) -> Board {
    load("work", dir, &core_env(home, NOW)).unwrap()
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

/// Every file of `dir` with its bytes, sorted by name.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut v: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let b = fs::read(&p).unwrap();
            (p, b)
        })
        .collect();
    v.sort();
    v
}

#[test]
fn nfr06_move_changes_only_column_line() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    let before = fs::read(&path).unwrap();
    let b = board(tmp.path(), &dir);
    let m = move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "Done").unwrap();
    assert!(m.changed);
    assert_eq!(m.card.column, "Done");
    assert_only_lines_changed(&before, &fs::read(&path).unwrap(), &["column: Done\n"]);
}

#[test]
fn ts6_2_move_to_current_column_writes_nothing() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    let before = fs::read(&path).unwrap();
    let mtime = fs::metadata(&path).unwrap().modified().unwrap();
    let b = board(tmp.path(), &dir);
    let m = move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "doing").unwrap();
    assert!(!m.changed);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), mtime);
}

#[test]
fn ts6_2_move_to_unknown_column_is_not_found() {
    let (tmp, dir) = copy();
    let before = files(&dir);
    let b = board(tmp.path(), &dir);
    let err = move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "Doign").unwrap_err();
    assert_eq!(
        err,
        Error::NotFound("unknown column 'Doign' on work (Todo, Doing, Done)".to_string())
    );
    assert_eq!(files(&dir), before);
}

#[test]
fn ts10_move_reads_file_fresh_inside_lock() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    let b = board(tmp.path(), &dir);
    let note = "\n### 2026-10-08T11:50:00+03:00\n- Doing: appended after load\n";
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(note.as_bytes());
    fs::write(&path, &bytes).unwrap();
    move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "Done").unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.ends_with(note), "{text}");
    assert!(text.contains("\ncolumn: Done\n"), "{text}");
}

#[test]
fn ts4_7_move_of_read_only_card_is_unreadable() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    fs::copy(Path::new(CARDS).join("conflict_markers.md"), &path).unwrap();
    let before = fs::read(&path).unwrap();
    let b = board(tmp.path(), &dir);
    let err = move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "Done").unwrap_err();
    assert!(matches!(err, Error::Unreadable(_)), "{err:?}");
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn nfr06_move_adds_frontmatter_to_card_without_one() {
    let (tmp, dir) = copy();
    let path = dir.join("W-20.md");
    fs::write(&path, "# spike on caching\n").unwrap();
    let b = board(tmp.path(), &dir);
    let m = move_card(&core_env(tmp.path(), NOW), &b, id("W-20"), "Doing").unwrap();
    assert!(m.changed);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "---\ncolumn: Doing\n---\n# spike on caching\n"
    );
}

#[test]
fn ts6_2_move_of_missing_card_is_not_found() {
    let (tmp, dir) = copy();
    let b = board(tmp.path(), &dir);
    let err = move_card(&core_env(tmp.path(), NOW), &b, id("W-99"), "Done").unwrap_err();
    assert_eq!(err, Error::NotFound("W-99 not found".to_string()));
}

#[test]
fn ts6_2_move_to_current_column_in_other_case_writes_nothing() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace("column: Doing", "column: doing")).unwrap();
    let before = fs::read(&path).unwrap();
    let b = board(tmp.path(), &dir);
    let m = move_card(&core_env(tmp.path(), NOW), &b, id("W-12"), "Doing").unwrap();
    assert!(!m.changed);
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn r7_verify_move_accepts_every_writable_fixture() {
    let mut n = 0;
    for e in fs::read_dir(CARDS).unwrap() {
        let name = e.unwrap().file_name().into_string().unwrap();
        let before = fs::read(Path::new(CARDS).join(&name)).unwrap();
        if check_writable(&name, &before).is_err() {
            continue;
        }
        let after = set_column(&name, &before, "Doing").unwrap();
        assert!(verify_move(&before, &after, "Doing"), "{name}");
        n += 1;
    }
    assert!(n > 0);
}

#[test]
fn ts6_2_move_to_current_column_in_other_unicode_case_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("board.md"),
        "---\nletter: W\ncolumns: Надо, Делаю, Готово\n---\n",
    )
    .unwrap();
    let path = dir.join("W-1.md");
    fs::write(&path, "---\ncolumn: готово\n---\n# x\n").unwrap();
    let before = fs::read(&path).unwrap();
    let b = board(tmp.path(), &dir);
    let m = move_card(&core_env(tmp.path(), NOW), &b, id("W-1"), "Готово").unwrap();
    assert!(!m.changed);
    assert_eq!(fs::read(&path).unwrap(), before);
}
