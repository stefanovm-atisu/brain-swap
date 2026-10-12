//! TECHSPEC 4.1, 4.6 and 10: discovering and loading the cards of a board folder.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use brain_swap::core::board::{Column, load};
use brain_swap::core::error::Error;
use brain_swap::core::model::{Board, Card, CardId};
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);

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

fn load_at(home: &Path, dir: &Path) -> Board {
    load("work", dir, &core_env(home, NOW)).unwrap()
}

fn ids(b: &Board) -> Vec<String> {
    let mut ids: Vec<_> = b.cards.iter().map(|c| c.id).collect();
    ids.sort();
    ids.iter().map(|i| i.to_string()).collect()
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn set_next(dir: &Path, next: u32) {
    let md = format!("---\nletter: W\ncolumns: Todo, Doing, Done\nnext: {next}\n---\n# work\n");
    fs::write(dir.join("board.md"), md).unwrap();
}

#[test]
fn fr04_missing_folder_reads_as_empty_board() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    let b = load_at(tmp.path(), &dir);
    assert!(b.cards.is_empty());
    assert_eq!(b.meta.next, 1);
    assert_eq!(b.meta.letter, 'W');
    assert_eq!(b.meta.columns, ["Todo", "Doing", "Done"]);
    assert!(!dir.exists());
}

#[test]
fn fr01_loads_one_card_per_file() {
    let (tmp, dir) = copy();
    // A read-only folder proves loading writes nothing (FR-04, 12.2).
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
    let b = load_at(tmp.path(), &dir);
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(ids(&b), ["W-9", "W-11", "W-12"]);
    assert!(b.cards.iter().all(|c| !c.path.ends_with("README.md")));
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
    assert_eq!((b.name.as_str(), b.path.as_path()), ("work", dir.as_path()));
    let names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 5);
}

#[test]
fn ts4_1_other_letter_card_is_ignored_with_warning() {
    let (tmp, dir) = copy();
    fs::write(dir.join("H-3.md"), "# home card\n").unwrap();
    let b = load_at(tmp.path(), &dir);
    assert!(b.cards.iter().all(|c| c.id != id("H-3")));
    assert_eq!(
        b.warnings,
        ["H-3.md: letter H is not this board's W, ignored"]
    );
}

#[test]
fn ts4_1_subfolders_and_temp_files_are_ignored() {
    let (tmp, dir) = copy();
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/W-1.md"), "# nested\n").unwrap();
    fs::write(dir.join(".W-12.md.bs-tmp-4711"), "# half written\n").unwrap();
    fs::write(dir.join("w-12.md"), "# lower case\n").unwrap();
    fs::create_dir(dir.join("W-2.md")).unwrap();
    let b = load_at(tmp.path(), &dir);
    assert_eq!(ids(&b), ["W-9", "W-11", "W-12"]);
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
}

#[test]
fn ts4_6_next_is_max_of_board_md_and_highest_plus_one() {
    let (tmp, dir) = copy();
    set_next(&dir, 5);
    assert_eq!(load_at(tmp.path(), &dir).meta.next, 13);
    set_next(&dir, 20);
    assert_eq!(load_at(tmp.path(), &dir).meta.next, 20);
}

#[test]
fn ts10_bad_card_does_not_stop_loading() {
    let (tmp, dir) = copy();
    fs::write(dir.join("W-5.md"), b"# bad \xff bytes\n").unwrap();
    let b = load_at(tmp.path(), &dir);
    let w5 = b.cards.iter().find(|c| c.id == id("W-5")).expect("W-5");
    assert!(!w5.writable);
    assert!(b.cards.iter().any(|c| c.id == id("W-12")));
    assert!(
        b.warnings.iter().any(|w| w.starts_with("W-5.md")),
        "{:?}",
        b.warnings
    );
}

#[test]
fn ts10_unlistable_folder_is_io_error() {
    let (tmp, dir) = copy();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&dir).is_ok() {
        // Root ignores permissions; nothing to check.
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let r = load("work", &dir, &core_env(tmp.path(), NOW));
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    match r {
        Err(Error::Io(m)) => assert!(m.starts_with("board work: "), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn ts3_card_mtime_comes_from_file() {
    let (tmp, dir) = copy();
    let at = parse_stamp("2026-10-06T17:10:00+03:00", &TimeZone::UTC).unwrap();
    let f = fs::File::options()
        .write(true)
        .open(dir.join("W-9.md"))
        .unwrap();
    f.set_modified(at.timestamp().into()).unwrap();
    drop(f);
    let b = load_at(tmp.path(), &dir);
    let w9 = b.cards.iter().find(|c| c.id == id("W-9")).unwrap();
    assert_eq!(
        w9.mtime.as_ref().map(|m| m.timestamp()),
        Some(at.timestamp())
    );
}

/// A fresh board at `<tmp>/work` with `columns`, its files written in the order given.
fn board(columns: &str, files: &[(&str, String)]) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    let md = format!("---\nletter: W\ncolumns: {columns}\n---\n# work\n");
    fs::write(dir.join("board.md"), md).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    (tmp, dir)
}

/// Card text in `column` with an optional `created` and notes `(heading stamp, placed)`.
fn card(column: &str, created: Option<&str>, notes: &[(&str, bool)]) -> String {
    let mut s = format!("---\ncolumn: {column}\n");
    if let Some(c) = created {
        s += &format!("created: {c}\n");
    }
    s += "---\n# t\n\n## Timeline\n";
    for (at, placed) in notes {
        s += &format!("\n### {at}\n- Doing: a\n- Next: b\n- Watch out: c\n");
        if *placed {
            s += "<!-- where: cwd=/a -->\n";
        }
    }
    s
}

fn order(b: &Board, column: usize) -> Vec<String> {
    b.cards_in(column)
        .iter()
        .map(|c| c.id.to_string())
        .collect()
}

fn get<'a>(b: &'a Board, s: &str) -> &'a Card {
    b.cards.iter().find(|c| c.id == id(s)).expect(s)
}

fn one_card(notes: &[(&str, bool)]) -> Board {
    let (tmp, dir) = board(
        "Todo, Doing, Done",
        &[("W-1.md", card("Todo", None, notes))],
    );
    load_at(tmp.path(), &dir)
}

#[test]
fn fr08_card_in_last_column_is_not_open() {
    let (tmp, dir) = copy();
    let b = load_at(tmp.path(), &dir);
    assert!(!b.is_open(get(&b, "W-9")));
    assert!(b.is_open(get(&b, "W-12")));
    assert!(b.is_open(get(&b, "W-11")));
}

#[test]
fn fr08_custom_last_column_is_done() {
    let files = [("W-1.md", card("Shipped", None, &[]))];
    let (tmp, dir) = board("Backlog, Review, Shipped", &files);
    let b = load_at(tmp.path(), &dir);
    assert!(!b.is_open(get(&b, "W-1")));
}

#[test]
fn i5_unknown_column_is_extra_and_open() {
    let (tmp, dir) = copy();
    fs::write(dir.join("W-13.md"), card("Doign", None, &[])).unwrap();
    let b = load_at(tmp.path(), &dir);
    let cols = b.column_list();
    assert_eq!(
        cols.last(),
        Some(&Column {
            name: "Doign (unknown)".to_string(),
            unknown: true
        })
    );
    assert_eq!(order(&b, cols.len() - 1), ["W-13"]);
    assert!(b.is_open(get(&b, "W-13")));
    assert!(
        b.warnings
            .contains(&"W-13: unknown column 'Doign'".to_string()),
        "{:?}",
        b.warnings
    );
}

#[test]
fn ts4_7_extra_columns_in_order_of_first_appearance() {
    let files = [
        ("W-5.md", card("Alpha", None, &[])),
        ("W-3.md", card("Zeta", None, &[])),
    ];
    let (tmp, dir) = board("Todo, Doing, Done", &files);
    let b = load_at(tmp.path(), &dir);
    let extra: Vec<_> = b
        .column_list()
        .into_iter()
        .filter(|c| c.unknown)
        .map(|c| c.name)
        .collect();
    assert_eq!(extra, ["Zeta (unknown)", "Alpha (unknown)"]);
}

#[test]
fn ts4_7_column_matches_ignoring_case() {
    let files = [("W-1.md", card("doing", None, &[]))];
    let (tmp, dir) = board("Todo, Doing, Done", &files);
    let b = load_at(tmp.path(), &dir);
    assert_eq!(order(&b, 1), ["W-1"]);
    assert_eq!(get(&b, "W-1").column, "Doing");
    assert_eq!(b.column_list().len(), 3);
}

#[test]
fn fr09_cards_ordered_newest_activity_first() {
    let files = [
        (
            "W-1.md",
            card("Todo", None, &[("2026-10-08T10:00:00+03:00", true)]),
        ),
        (
            "W-2.md",
            card("Todo", None, &[("2026-10-08T11:00:00+03:00", true)]),
        ),
        (
            "W-3.md",
            card("Todo", Some("2026-10-08T10:30:00+03:00"), &[]),
        ),
    ];
    let (tmp, dir) = board("Todo, Doing, Done", &files);
    assert_eq!(order(&load_at(tmp.path(), &dir), 0), ["W-2", "W-3", "W-1"]);
}

#[test]
fn fr09_equal_activity_ties_to_higher_number() {
    let at = Some("2026-10-08T10:00:00+03:00");
    let (w7, w12) = (
        ("W-7.md", card("Todo", at, &[])),
        ("W-12.md", card("Todo", at, &[])),
    );
    for files in [[w7.clone(), w12.clone()], [w12, w7]] {
        let (tmp, dir) = board("Todo, Doing, Done", &files);
        assert_eq!(order(&load_at(tmp.path(), &dir), 0), ["W-12", "W-7"]);
    }
}

#[test]
fn fr09_activity_falls_back_to_created_then_mtime() {
    let b = one_card(&[]);
    let c = get(&b, "W-1");
    assert!(c.mtime.is_some());
    assert_eq!(c.activity(), c.mtime.as_ref());
}

#[test]
fn fr09_activity_compares_instants_across_offsets() {
    let files = [
        (
            "W-1.md",
            card("Todo", None, &[("2026-10-08T10:00:00+03:00", true)]),
        ),
        (
            "W-2.md",
            card("Todo", None, &[("2026-10-08T08:30:00+00:00", true)]),
        ),
    ];
    let (tmp, dir) = board("Todo, Doing, Done", &files);
    assert_eq!(order(&load_at(tmp.path(), &dir), 0), ["W-2", "W-1"]);
}

#[test]
fn fr09_activity_uses_last_note_not_newest_stamp() {
    let b = one_card(&[
        ("2026-10-08T11:00:00+03:00", true),
        ("2026-10-08T10:00:00+03:00", true),
    ]);
    let c = get(&b, "W-1");
    assert!(c.notes[1].at.is_some());
    assert_eq!(c.activity(), c.notes[1].at.as_ref());
}

#[test]
fn ts3_jump_note_is_last_note_with_place() {
    let b = one_card(&[
        ("2026-10-08T09:00:00+03:00", true),
        ("2026-10-08T10:00:00+03:00", true),
        ("2026-10-08T11:00:00+03:00", false),
    ]);
    assert_eq!(get(&b, "W-1").jump_note(), Some(1));
}

#[test]
fn ts4_7_bad_stamp_note_with_place_is_jump_target() {
    let b = one_card(&[("2026-10-08T09:00:00+03:00", false), ("someday", true)]);
    let c = get(&b, "W-1");
    assert_eq!(c.notes[1].at, None);
    assert_eq!(c.jump_note(), Some(1));
}

#[test]
fn ts3_jump_note_none_without_placed_notes() {
    let b = one_card(&[
        ("2026-10-08T09:00:00+03:00", false),
        ("2026-10-08T10:00:00+03:00", false),
    ]);
    assert_eq!(get(&b, "W-1").jump_note(), None);
}
