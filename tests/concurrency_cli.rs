//! TECHSPEC 10 and 12.1 (concurrency, process level), FR-16, AS-8: parallel `park`, `move` and `new`.

mod common;

use brain_swap::core::board::load;
use brain_swap::core::model::Board;
use common::cli::Sandbox;
use common::env::core_env;
use std::fs;
use std::path::PathBuf;
use std::sync::Barrier;
use std::thread;

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn sandbox(fixture: &str) -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board(fixture, "work");
    s
}

fn dir(s: &Sandbox) -> PathBuf {
    s.root().join("boards/work")
}

fn board(s: &Sandbox) -> Board {
    let b = load("work", &dir(s), &core_env(s.root(), NOW)).unwrap();
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
    b
}

/// Starts every `(args, stdin)` as a process from its own thread, all released by one barrier,
/// and returns the exit codes. Exit 5 (lock wait over 2 s, nothing written) is retried up to 10 times.
fn run(s: &Sandbox, jobs: Vec<(Vec<String>, String)>) -> Vec<i32> {
    let barrier = Barrier::new(jobs.len());
    thread::scope(|t| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|(args, stdin)| {
                let barrier = &barrier;
                t.spawn(move || {
                    let args: Vec<&str> = args.iter().map(String::as_str).collect();
                    barrier.wait();
                    let mut code = 5;
                    for _ in 0..10 {
                        let out = s.cmd(&args).write_stdin(stdin.as_str()).output().unwrap();
                        code = out.status.code().unwrap();
                        if code != 5 {
                            break;
                        }
                    }
                    code
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn job(args: &[&str], stdin: &str) -> (Vec<String>, String) {
    (args.iter().map(|a| a.to_string()).collect(), stdin.into())
}

/// The block `park` appends at the default `BRAIN_SWAP_NOW` from `<root>/elsewhere`.
fn block(s: &Sandbox, doing: &str) -> String {
    format!(
        "\n### 2026-10-08T08:52:00+00:00\n- Doing: {doing}\n- Next:\n- Watch out:\n<!-- where: cwd={}/elsewhere -->\n",
        s.root().display()
    )
}

#[test]
fn ts12_1_parallel_parks_and_moves_lose_nothing() {
    let s = sandbox("pair");
    let mut jobs = Vec::new();
    for i in 1..=20 {
        let card = if i % 2 == 0 { "W-1" } else { "W-2" };
        jobs.push(job(&["park", "--card", card], &format!("Doing: n{i}")));
    }
    for i in 0..5 {
        let card = if i % 2 == 0 { "W-1" } else { "W-2" };
        let column = ["Doing", "Todo"][(i / 2) % 2];
        jobs.push(job(&["move", card, column], ""));
    }
    assert_eq!(run(&s, jobs), vec![0; 25]);
    let b = board(&s);
    assert_eq!(b.cards.len(), 2);
    let mut doings: Vec<String> = b
        .cards
        .iter()
        .flat_map(|c| c.notes.iter().map(|n| n.doing.clone()))
        .collect();
    doings.sort();
    let mut want: Vec<String> = (1..=20).map(|i| format!("n{i}")).collect();
    want.sort();
    assert_eq!(doings, want);
    for c in &b.cards {
        assert!(["Todo", "Doing"].contains(&c.column.as_str()), "{c:?}");
    }
}

#[test]
fn ts12_1_parallel_creations_get_distinct_ids() {
    let s = sandbox("pair");
    let jobs = (0..10)
        .map(|i| job(&["new", &format!("task {i}")], ""))
        .collect();
    assert_eq!(run(&s, jobs), vec![0; 10]);
    let b = board(&s);
    let mut numbers: Vec<u32> = b.cards.iter().map(|c| c.id.number).collect();
    numbers.sort();
    numbers.dedup();
    assert_eq!(numbers.len(), 12, "{numbers:?}");
    let files = fs::read_dir(dir(&s)).unwrap().count();
    assert_eq!(files, 13);
    assert!(b.meta.next > *numbers.last().unwrap(), "{}", b.meta.next);
}

#[test]
fn as8_two_parks_on_different_cards_in_one_second() {
    let s = sandbox("work");
    let read = |name: &str| fs::read(dir(&s).join(name)).unwrap();
    let names = ["W-9.md", "W-11.md", "board.md", "W-12.md", "W-7.md"];
    let before = names.map(read);
    let jobs = vec![
        job(&["park", "--card", "W-12"], "Doing: twelve"),
        job(&["park", "--card", "W-7"], "Doing: seven"),
    ];
    assert_eq!(run(&s, jobs), vec![0, 0]);
    let after = names.map(read);
    assert_eq!(after[..3], before[..3]);
    assert_eq!(
        after[3],
        [&before[3][..], block(&s, "twelve").as_bytes()].concat()
    );
    assert_eq!(
        after[4],
        [&before[4][..], block(&s, "seven").as_bytes()].concat()
    );
}

#[test]
fn as8_two_parks_on_one_card_append_both() {
    let s = sandbox("work");
    let path = dir(&s).join("W-12.md");
    let before = fs::read_to_string(&path).unwrap();
    let jobs = vec![
        job(&["park", "--card", "W-12"], "Doing: one"),
        job(&["park", "--card", "W-12"], "Doing: two"),
    ];
    assert_eq!(run(&s, jobs), vec![0, 0]);
    let after = fs::read_to_string(&path).unwrap();
    let [one, two] = ["one", "two"].map(|d| block(&s, d));
    assert!(
        after == before.clone() + &one + &two || after == before + &two + &one,
        "{after}"
    );
}
