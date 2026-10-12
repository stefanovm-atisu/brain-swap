//! TECHSPEC 10 and 12.1 (concurrency, threads), NFR-08, AS-8: parallel writes through the store.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;

use brain_swap::core::board::load;
use brain_swap::core::card_file::{note_blocks, render_note};
use brain_swap::core::env::Env;
use brain_swap::core::error::{Error, Result};
use brain_swap::core::model::{Board, CardId, NewNote};
use brain_swap::core::store::{CreateRequest, create, move_card, park};
use brain_swap::core::template::{builtins, find};
use brain_swap::core::time::parse_stamp;
use common::env::core_env;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);
const W7: &str = "---\ncolumn: Doing\ntemplate: Chore\ncreated: 2026-10-08T09:00:00+03:00\n---\n# rotate the staging keys\n\n## Timeline\n";

/// One thread's store call, repeatable because a `Busy` call is retried.
type Job<T> = Box<dyn Fn(&Env, &Board) -> Result<T> + Send>;

/// A temp home holding a copy of `work_4_9` plus `W-7` in Doing at `<home>/work`.
fn copy() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    for e in fs::read_dir(WORK_4_9).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    fs::write(dir.join("W-7.md"), W7).unwrap();
    (tmp, dir)
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
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

fn notes(dir: &Path, card: &str) -> usize {
    note_blocks(&fs::read_to_string(dir.join(format!("{card}.md"))).unwrap()).len()
}

/// Runs each job on its own thread with its own `Env` (pid 1000 + index), all released by one barrier.
/// A `Busy` result (lock wait over 2 s, nothing written) is retried up to 10 times, as its text asks.
fn run<T: Send + 'static>(home: &Path, dir: &Path, jobs: Vec<Job<T>>) -> Vec<Result<T>> {
    let barrier = Arc::new(Barrier::new(jobs.len()));
    let handles: Vec<_> = jobs
        .into_iter()
        .enumerate()
        .map(|(i, job)| {
            let mut env = core_env(home, NOW);
            env.pid = 1000 + i as u32;
            let board = load("work", dir, &env).unwrap();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                let mut r = job(&env, &board);
                for _ in 0..10 {
                    if !matches!(r, Err(Error::Busy(_))) {
                        break;
                    }
                    r = job(&env, &board);
                }
                r
            })
        })
        .collect();
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Round one: 10 parks on W-12, 10 on W-7, 5 moves alternating W-12 and W-7 between Done and Doing.
fn round_one(home: &Path, dir: &Path) -> Vec<Result<()>> {
    let mut jobs: Vec<Job<()>> = Vec::new();
    for i in 0..20 {
        let card = if i % 2 == 0 { "W-12" } else { "W-7" };
        jobs.push(Box::new(move |env, board| {
            park(env, board, id(card), &note(&format!("park {i}"))).map(|_| ())
        }));
    }
    for i in 0..5 {
        let card = if i % 2 == 0 { "W-12" } else { "W-7" };
        let column = ["Done", "Doing"][(i / 2) % 2];
        jobs.push(Box::new(move |env, board| {
            move_card(env, board, id(card), column).map(|_| ())
        }));
    }
    run(home, dir, jobs)
}

#[test]
fn nfr08_parallel_parks_and_moves_lose_nothing() {
    let (tmp, dir) = copy();
    let before = (notes(&dir, "W-12"), notes(&dir, "W-7"));
    let results = round_one(tmp.path(), &dir);
    assert_eq!(results.len(), 25);
    for r in &results {
        assert!(r.is_ok(), "{r:?}");
    }
    let board = load("work", &dir, &core_env(tmp.path(), NOW)).unwrap();
    assert!(board.warnings.is_empty(), "{:?}", board.warnings);
    assert_eq!(notes(&dir, "W-12"), before.0 + 10);
    assert_eq!(notes(&dir, "W-7"), before.1 + 10);
    for card in ["W-12", "W-7"] {
        let c = board.card(id(card)).unwrap();
        assert_eq!(c.notes.len(), notes(&dir, card));
        assert!(["Done", "Doing"].contains(&c.column.as_str()), "{c:?}");
    }
}

#[test]
fn nfr08_parallel_creates_get_distinct_ids() {
    let (tmp, dir) = copy();
    for r in round_one(tmp.path(), &dir) {
        r.unwrap();
    }
    let mut jobs: Vec<Job<CardId>> = Vec::new();
    for i in 0..10 {
        jobs.push(Box::new(move |env, board| {
            let template = find(&builtins(), "Feature").unwrap().clone();
            let title = format!("task {i}");
            let req = CreateRequest {
                title: &title,
                template: &template,
                column: None,
                body: None,
            };
            create(env, board, &req).map(|c| c.card.id)
        }));
    }
    let mut ids: Vec<CardId> = run(tmp.path(), &dir, jobs)
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    ids.sort_by_key(|i| i.number);
    ids.dedup();
    assert_eq!(ids.len(), 10, "{ids:?}");
    let board = load("work", &dir, &core_env(tmp.path(), NOW)).unwrap();
    assert!(board.warnings.is_empty(), "{:?}", board.warnings);
    assert!(board.meta.next > ids.last().unwrap().number);
    for i in &ids {
        assert!(board.card(*i).is_some(), "{i:?} missing");
    }
    assert_eq!(board.cards.len(), 4 + 10);
}

#[test]
fn as8_parks_on_two_cards_in_one_second_both_land() {
    let (tmp, dir) = copy();
    let files = ["W-12", "W-7"].map(|c| dir.join(format!("{c}.md")));
    let before = files.clone().map(|p| fs::read(p).unwrap());
    let mut jobs: Vec<Job<()>> = Vec::new();
    for card in ["W-12", "W-7"] {
        jobs.push(Box::new(move |env, board| {
            park(env, board, id(card), &note(card)).map(|_| ())
        }));
    }
    for r in run(tmp.path(), &dir, jobs) {
        r.unwrap();
    }
    for (i, card) in ["W-12", "W-7"].into_iter().enumerate() {
        let block = render_note(&note(card), "\n").0;
        let after = fs::read(&files[i]).unwrap();
        assert_eq!(after, [&before[i][..], block.as_bytes()].concat(), "{card}");
    }
}

#[test]
fn as8_two_parks_on_one_card_both_append() {
    let (tmp, dir) = copy();
    let path = dir.join("W-12.md");
    let before = fs::read(&path).unwrap();
    let mut jobs: Vec<Job<()>> = Vec::new();
    for doing in ["session A", "session B"] {
        jobs.push(Box::new(move |env, board| {
            park(env, board, id("W-12"), &note(doing)).map(|_| ())
        }));
    }
    for r in run(tmp.path(), &dir, jobs) {
        r.unwrap();
    }
    let after = fs::read(&path).unwrap();
    let [a, b] = ["session A", "session B"].map(|d| render_note(&note(d), "\n").0);
    let ab = [&before[..], a.as_bytes(), b.as_bytes()].concat();
    let ba = [&before[..], b.as_bytes(), a.as_bytes()].concat();
    assert!(
        after == ab || after == ba,
        "{}",
        String::from_utf8_lossy(&after)
    );
}
