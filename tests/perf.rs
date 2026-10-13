//! NFR-01 on the 300-card board of A-E6-03; the `#[ignore]` tests run with
//! `cargo test --release --test perf -- --ignored --nocapture`.

mod common;

use brain_swap::core::board::load;
use common::big_board::write_big_board;
use common::cli::Sandbox;
use common::env::core_env;
use common::timing::{p95_ms, time_runs};
use std::fs;
use std::path::Path;

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn release() {
    if cfg!(debug_assertions) {
        panic!("run with cargo test --release");
    }
}

/// The perf sandbox: config, the board at `<root>/boards/work`, and `perf-s1` linked to W-296.
fn sandbox() -> Sandbox {
    release();
    let s = Sandbox::new();
    s.write_config();
    write_big_board(&s.root().join("boards/work"), 300, 20);
    s.cmd(&["link", "W-296", "--session", "perf-s1"])
        .assert()
        .success();
    s
}

/// Times `run` with 3 warm-ups and 20 runs, prints and checks the p95.
fn nfr01(command: &str, run: impl FnMut()) {
    let mut samples = time_runs(3, 20, run);
    let p95 = p95_ms(&mut samples);
    println!("nfr01 {command}: p95 {p95:.1} ms");
    assert!(p95 <= 50.0, "nfr01 {command}: p95 {p95} ms");
}

/// Times `brain-swap <args>`, each run exiting 0.
fn nfr01_cmd(args: &[&str]) {
    let s = sandbox();
    nfr01(&args.join(" "), || {
        s.cmd(args).assert().success();
    });
}

/// Writes `<root>/` for the `hyperfine` runs of E6-F1-T4 (A-E6-20).
fn write_hyperfine_board(root: &Path) {
    let _ = root;
    todo!()
}

#[test]
fn nfr01_p95_takes_rank_ceil_of_095_n() {
    let mut twenty: Vec<f64> = (1..=20).map(|i| ((i * 7) % 20 + 1) as f64).collect();
    assert_eq!(p95_ms(&mut twenty), 19.0);
    let mut fifty: Vec<f64> = (1..=50).map(|i| ((i * 13) % 50 + 1) as f64).collect();
    assert_eq!(p95_ms(&mut fifty), 48.0);
}

#[test]
fn nfr01_perf_board_loads_300_cards_with_20_notes() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    write_big_board(&dir, 300, 20);
    let board = load("work", &dir, &core_env(tmp.path(), NOW)).unwrap();
    assert_eq!(board.cards.len(), 300);
    assert!(board.cards.iter().all(|c| c.notes.len() == 20));
    let open = board.open_cards();
    assert_eq!(open.len(), 10);
    assert_eq!(open.iter().filter(|c| c.column == "Todo").count(), 5);
    assert_eq!(open.iter().filter(|c| c.column == "Doing").count(), 5);
    assert_eq!(board.meta.next, 301);
    assert_eq!(board.warnings, Vec::<String>::new());
}

#[test]
#[ignore]
fn nfr01_templates_p95_under_50ms() {
    nfr01_cmd(&["templates"]);
}

#[test]
#[ignore]
fn nfr01_context_p95_under_50ms() {
    let s = sandbox();
    nfr01("context --session perf-s1", || {
        let out = s.cmd(&["context", "--session", "perf-s1"]).assert().code(0);
        let text = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
        assert!(text.contains("reference: W-296"), "{text}");
    });
}

#[test]
#[ignore]
fn nfr01_show_all_p95_under_50ms() {
    nfr01_cmd(&["show", "W-150", "--all"]);
}

#[test]
#[ignore]
fn nfr01_ls_p95_under_50ms() {
    nfr01_cmd(&["ls"]);
}

#[test]
#[ignore]
fn nfr01_ls_open_guess_p95_under_50ms() {
    nfr01_cmd(&["ls", "--open", "--guess"]);
}

#[test]
#[ignore]
fn nfr01_link_p95_under_50ms() {
    nfr01_cmd(&["link", "W-297", "--session", "perf-s2"]);
}

#[test]
#[ignore]
fn nfr01_move_p95_under_50ms() {
    let s = sandbox();
    let mut to = ["Doing", "Todo"].into_iter().cycle();
    nfr01("move W-291 Doing|Todo", || {
        s.cmd(&["move", "W-291", to.next().unwrap()])
            .assert()
            .success();
    });
}

#[test]
#[ignore]
fn nfr01_park_p95_under_50ms() {
    let s = sandbox();
    nfr01("park --card W-298 --session perf-s3", || {
        let out = s
            .cmd(&["park", "--card", "W-298", "--session", "perf-s3"])
            .write_stdin("Doing: timing park\nNext: time it again\nWatch out: nothing\n")
            .assert()
            .success();
        let text = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
        assert!(text.starts_with("parked to W-298: "), "{text}");
    });
}

#[test]
#[ignore]
fn nfr01_new_p95_under_50ms() {
    nfr01_cmd(&["new", "perf", "card"]);
}

#[test]
#[ignore]
fn sm5_write_hyperfine_board() {
    release();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("perf-board");
    write_hyperfine_board(&root);
    let mut env = core_env(&root, NOW);
    env.config_home = root.join("config");
    let config = brain_swap::core::config::load(&env).unwrap().config;
    let work = root.join("data/work");
    assert_eq!(config.board("work").unwrap().1, work);
    assert_eq!(load("work", &work, &env).unwrap().cards.len(), 300);
    let pristine = root.join("data/work.pristine");
    for e in fs::read_dir(&work).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap();
        assert_eq!(
            fs::read(&p).unwrap(),
            fs::read(pristine.join(name)).unwrap()
        );
    }
    assert_eq!(
        fs::read_dir(&work).unwrap().count(),
        fs::read_dir(&pristine).unwrap().count()
    );
    assert!(root.join("state").is_dir());
    let note = fs::read_to_string(root.join("note.txt")).unwrap();
    for part in ["Doing:", "Next:", "Watch out:"] {
        assert!(note.lines().any(|l| l.starts_with(part)), "{note}");
    }
    let sh = fs::read_to_string(root.join("env.sh")).unwrap();
    for var in [
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
        "BRAIN_SWAP_NOW",
    ] {
        assert!(sh.contains(&format!("export {var}=")), "{sh}");
    }
    for var in ["HERDR_ENV", "BRAIN_SWAP_SESSION", "BRAIN_SWAP_LOG"] {
        assert!(
            sh.lines()
                .any(|l| l.starts_with("unset ") && l.contains(var)),
            "{sh}"
        );
    }
}
