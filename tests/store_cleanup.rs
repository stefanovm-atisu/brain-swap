//! TECHSPEC 4.6 and 10: cleanup of stale temp files and old session files after store writes (FR-05, T-17).

mod common;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::model::{Board, CardId, NewNote};
use brain_swap::core::store::{CreateRequest, create, move_card, park};
use brain_swap::core::template::{builtins, find};
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use jiff::SignedDuration;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);

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

fn minutes(n: i64) -> SignedDuration {
    SignedDuration::from_mins(n)
}

fn days(n: i64) -> SignedDuration {
    SignedDuration::from_hours(24 * n)
}

/// Sets the mtime of `path` (file or folder) to `age` before `env.now`.
fn age(env: &Env, path: &Path, age: SignedDuration) {
    let mtime = SystemTime::from(env.now.timestamp() - age);
    File::open(path).unwrap().set_modified(mtime).unwrap();
}

/// Creates `path` (and its folder) with its mtime `a` before `env.now`.
fn file_aged(env: &Env, path: &Path, a: SignedDuration) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(path).unwrap();
    age(env, path, a);
    path.to_path_buf()
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn note() -> NewNote {
    NewNote {
        at: parse_stamp(NOW, &TimeZone::UTC).unwrap(),
        auto: false,
        doing: "switched to v13".into(),
        next: "rerun the migration test".into(),
        watch_out: String::new(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: None,
        session: None,
    }
}

#[test]
fn ts10_stale_temp_older_than_an_hour_is_removed_after_park() {
    let (_tmp, env, board) = copy();
    let old = file_aged(&env, &board.path.join(".W-5.md.bs-tmp-1"), minutes(61));
    let new = file_aged(&env, &board.path.join(".W-6.md.bs-tmp-1"), minutes(59));
    park(&env, &board, id("W-12"), &note()).unwrap();
    assert!(!old.exists());
    assert!(new.exists());
}

#[test]
fn ts10_old_session_files_are_pruned_after_a_write() {
    let (_tmp, env, board) = copy();
    let a = file_aged(&env, &env.sessions_dir().join("a.toml"), days(31));
    let b = file_aged(&env, &env.sessions_dir().join("b.toml"), days(29));
    let moved = move_card(&env, &board, id("W-12"), "Done").unwrap();
    assert!(moved.changed);
    assert!(!a.exists());
    assert!(b.exists());
}

#[test]
fn ts10_cleanup_never_touches_other_files() {
    let (_tmp, env, board) = copy();
    let readme = board.path.join("README.md");
    age(&env, &readme, days(365));
    let plain = file_aged(&env, &board.path.join("notes.bs-tmp-1"), days(365));
    let template = find(&builtins(), "Feature").unwrap().clone();
    let req = CreateRequest {
        title: "new task",
        template: &template,
        column: None,
        body: None,
    };
    create(&env, &board, &req).unwrap();
    assert!(readme.exists());
    assert!(plain.exists());
}

#[test]
fn ts10_no_op_move_does_not_clean() {
    let (_tmp, env, board) = copy();
    let stale = file_aged(&env, &board.path.join(".W-5.md.bs-tmp-1"), minutes(61));
    let moved = move_card(&env, &board, id("W-12"), "Doing").unwrap();
    assert!(!moved.changed);
    assert!(stale.exists());
}

#[test]
fn ts10_cleanup_failure_never_fails_the_write() {
    let (_tmp, env, board) = copy();
    let dir = board.path.join(".W-5.md.bs-tmp-1");
    fs::create_dir(&dir).unwrap();
    File::create(dir.join("inner")).unwrap();
    age(&env, &dir, SignedDuration::from_hours(2));
    let parked = park(&env, &board, id("W-12"), &note()).unwrap();
    assert_eq!(parked.card.notes.last().unwrap().doing, "switched to v13");
    let on_disk = fs::read_to_string(board.path.join("W-12.md")).unwrap();
    assert!(on_disk.contains("switched to v13"));
    assert!(dir.is_dir());
}
