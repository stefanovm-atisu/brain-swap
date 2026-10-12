//! TECHSPEC 3 (I1 to I4), 4.6 and 12.1: in-process crash tests through failpoints on `Env` (NFR-08).
#![cfg(feature = "failpoints")]

mod common;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use brain_swap::core::board::load;
use brain_swap::core::card_file::parse;
use brain_swap::core::env::Env;
use brain_swap::core::error::{Error, Result};
use brain_swap::core::failpoint::Failpoint;
use brain_swap::core::model::{Board, CardId, NewNote};
use brain_swap::core::store::{CreateRequest, Created, create, merge_edit, park, temp_path};
use brain_swap::core::template::{builtins, find};
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use common::sound::assert_board_sound;
use jiff::SignedDuration;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);
const USED: [u32; 3] = [9, 11, 12];

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

/// `env` with failpoint `name` set to return instead of aborting.
fn failing(env: &Env, name: &str) -> Env {
    let mut e = env.clone();
    e.failpoint = Some(Failpoint {
        name: name.to_string(),
        abort: false,
    });
    e
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn path(board: &Board, s: &str) -> PathBuf {
    board.path.join(format!("{s}.md"))
}

fn new(env: &Env, board: &Board) -> Result<Created> {
    let template = find(&builtins(), "Feature").unwrap().clone();
    let req = CreateRequest {
        title: "new task",
        template: &template,
        column: None,
        body: None,
    };
    create(env, board, &req)
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

fn assert_io(r: Result<impl std::fmt::Debug>) {
    assert!(matches!(r, Err(Error::Io(_))), "{r:?}");
}

/// The card at `p` parses as writable without warnings.
fn assert_parses(env: &Env, p: &Path, s: &str) {
    let parsed = parse(id(s), p, &fs::read(p).unwrap(), "Todo", &env.tz);
    assert!(parsed.card.writable && parsed.warnings.is_empty(), "{s}");
}

/// Test 2's scenario: `create:after_tmp` fails and leaves `.W-13.md.bs-tmp-<pid>`.
fn crash_after_tmp() -> (tempfile::TempDir, Env, Board, PathBuf) {
    let (tmp, env, board) = copy();
    assert_io(new(&failing(&env, "create:after_tmp"), &board));
    let temp = temp_path(&path(&board, "W-13"), env.pid);
    (tmp, env, board, temp)
}

#[test]
fn nfr08_crash_after_next_only_skips_a_number() {
    let (_tmp, env, board) = copy();
    assert_io(new(&failing(&env, "create:after_next"), &board));
    let md = fs::read_to_string(board.path.join("board.md")).unwrap();
    assert!(md.contains("next: 14\n"), "{md}");
    assert!(!path(&board, "W-13").exists());
    assert_eq!(
        assert_board_sound(&env, &board, &[9, 11, 12, 13]),
        id("W-14")
    );
}

#[test]
fn nfr08_crash_after_tmp_leaves_ignored_temp() {
    let (_tmp, env, board, temp) = crash_after_tmp();
    assert!(temp.exists(), "{}", temp.display());
    let reloaded = load("work", &board.path, &env).unwrap();
    assert!(reloaded.card(id("W-13")).is_none());
    assert_eq!(reloaded.cards.len(), USED.len());
    assert_eq!(
        assert_board_sound(&env, &board, &[9, 11, 12, 13]),
        id("W-14")
    );
}

#[test]
fn nfr08_crash_after_link_leaves_card_and_temp() {
    let (_tmp, env, board) = copy();
    assert_io(new(&failing(&env, "create:after_link"), &board));
    let w13 = path(&board, "W-13");
    assert_parses(&env, &w13, "W-13");
    assert!(temp_path(&w13, env.pid).exists());
    assert_eq!(
        assert_board_sound(&env, &board, &[9, 11, 12, 13]),
        id("W-14")
    );
}

#[test]
fn nfr08_crash_during_park_keeps_old_card() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-12");
    let before = fs::read(&p).unwrap();
    assert_io(park(
        &failing(&env, "park:after_tmp"),
        &board,
        id("W-12"),
        &note("switching"),
    ));
    assert_eq!(fs::read(&p).unwrap(), before);
    assert_parses(&env, &p, "W-12");
    assert_board_sound(&env, &board, &USED);
}

#[test]
fn nfr08_crash_during_edit_merge_keeps_file() {
    let (_tmp, env, board) = copy();
    let p = path(&board, "W-12");
    let c0 = fs::read(&p).unwrap();
    park(&env, &board, id("W-12"), &note("parked meanwhile")).unwrap();
    let f = fs::read(&p).unwrap();
    let edited = String::from_utf8(c0.clone()).unwrap().replace(
        "# migrate invoices to v13\n",
        "# migrate invoices to v13 (edited)\n",
    );
    assert_io(merge_edit(
        &failing(&env, "edit:after_tmp"),
        &board,
        id("W-12"),
        &c0,
        edited.as_bytes(),
    ));
    assert_eq!(fs::read(&p).unwrap(), f);
    assert_parses(&env, &p, "W-12");
    assert_board_sound(&env, &board, &USED);
}

#[test]
fn nfr08_crash_temp_removed_after_an_hour() {
    let (_tmp, env, board, old) = crash_after_tmp();
    let stamp = SystemTime::from(env.now.timestamp());
    File::open(&old).unwrap().set_modified(stamp).unwrap();
    let young = board.path.join(".W-11.md.bs-tmp-1");
    fs::write(&young, "partial").unwrap();
    let two_minutes = SystemTime::from(env.now.timestamp() + SignedDuration::from_mins(2));
    File::open(&young)
        .unwrap()
        .set_modified(two_minutes)
        .unwrap();
    let mut later = env.clone();
    later.now = env.now.checked_add(SignedDuration::from_mins(61)).unwrap();
    park(&later, &board, id("W-11"), &note("an hour on")).unwrap();
    assert!(!old.exists(), "{}", old.display());
    assert!(young.exists(), "{}", young.display());
}
