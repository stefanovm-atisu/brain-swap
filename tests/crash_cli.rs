//! TECHSPEC 12.1 (crash, process level), 4.6, I1 to I4: `new` and `park` abort at a failpoint.
#![cfg(feature = "failpoints")]

mod common;

use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::model::{Board, CardId};
use common::cli::Sandbox;
use common::env::core_env;
use common::sound::assert_board_sound;
use std::fs;
use std::path::PathBuf;

const FOUR: [&str; 4] = ["W-7", "W-9", "W-11", "W-12"];

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    s
}

/// Runs `args` with `BRAIN_SWAP_FAILPOINT=name` and `stdin`, asserts a non-zero exit, then
/// drops the failpoint again.
fn crash(s: &mut Sandbox, name: &str, args: &[&str], stdin: &str) {
    let base = s.scenario_mut().clone();
    *s.scenario_mut() = base.clone().failpoint(name);
    s.cmd(args).write_stdin(stdin).assert().failure();
    *s.scenario_mut() = base;
}

fn path(s: &Sandbox, file: &str) -> PathBuf {
    s.root().join("boards/work").join(file)
}

/// The board reloaded through `core::board::load`, with the env it used.
fn board(s: &Sandbox) -> (Env, Board) {
    let env = core_env(&s.root().join("home"), "2026-10-08T11:52:00+03:00");
    let b = load("work", &s.root().join("boards/work"), &env).unwrap();
    (env, b)
}

/// The card IDs `brain-swap ls` lists (it must exit 0), by number; the reload agrees.
fn ids(s: &Sandbox) -> Vec<String> {
    let out = s.cmd(&["ls"]).assert().code(0).get_output().clone();
    let mut ids: Vec<_> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| CardId::parse(l.split(' ').nth(1).unwrap()).unwrap())
        .collect();
    ids.sort_by_key(|id| id.number);
    let mut loaded: Vec<_> = board(s).1.cards.iter().map(|c| c.id).collect();
    loaded.sort_by_key(|id| id.number);
    assert_eq!(ids, loaded);
    ids.iter().map(|id| id.to_string()).collect()
}

/// True when the board folder holds `.<file>.bs-tmp-*`.
fn has_temp(s: &Sandbox, file: &str) -> bool {
    let prefix = format!(".{file}.bs-tmp-");
    fs::read_dir(s.root().join("boards/work"))
        .unwrap()
        .any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(&prefix)
        })
}

/// `new y` without the failpoint creates `id`; then the 12.1 reload check, numbers up to `id` used.
fn new_y_then_sound(s: &Sandbox, id: &str) {
    let out = s.cmd(&["new", "y"]).assert().code(0).get_output().clone();
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("created {id}: y (Todo, work)\n")
    );
    let (env, b) = board(s);
    let used = id[2..].parse().unwrap();
    assert_board_sound(&env, &b, &[used]);
}

#[test]
fn ts12_1_crash_create_after_next_skips_number() {
    let mut s = sandbox();
    crash(&mut s, "create:after_next", &["new", "x"], "");
    let md = fs::read_to_string(path(&s, "board.md")).unwrap();
    assert!(md.contains("next: 14\n"), "{md}");
    assert!(!path(&s, "W-13.md").exists());
    assert_eq!(ids(&s), FOUR);
    new_y_then_sound(&s, "W-14");
}

#[test]
fn ts12_1_crash_create_after_tmp_leaves_ignored_temp() {
    let mut s = sandbox();
    crash(&mut s, "create:after_tmp", &["new", "x"], "");
    assert!(has_temp(&s, "W-13.md"));
    assert!(!path(&s, "W-13.md").exists());
    assert_eq!(ids(&s), FOUR);
    new_y_then_sound(&s, "W-14");
}

#[test]
fn ts12_1_crash_create_after_link_card_parses() {
    let mut s = sandbox();
    crash(&mut s, "create:after_link", &["new", "x"], "");
    let (_, b) = board(&s);
    let card = b.cards.iter().find(|c| c.id.to_string() == "W-13").unwrap();
    assert_eq!(card.title, "x");
    assert_eq!(ids(&s), ["W-7", "W-9", "W-11", "W-12", "W-13"]);
    new_y_then_sound(&s, "W-14");
}

#[test]
fn ts12_1_crash_park_after_tmp_leaves_card_unchanged() {
    let mut s = sandbox();
    let before = fs::read(path(&s, "W-12.md")).unwrap();
    let notes = |s: &Sandbox| {
        let (_, b) = board(s);
        b.cards
            .iter()
            .find(|c| c.id.to_string() == "W-12")
            .unwrap()
            .notes
            .len()
    };
    let n = notes(&s);
    crash(
        &mut s,
        "park:after_tmp",
        &["park", "--card", "W-12"],
        "Doing: x",
    );
    assert_eq!(fs::read(path(&s, "W-12.md")).unwrap(), before);
    assert!(has_temp(&s, "W-12.md"));
    let (env, b) = board(&s);
    assert_board_sound(&env, &b, &[12]);
    s.cmd(&["park", "--card", "W-12"])
        .write_stdin("Doing: x")
        .assert()
        .code(0);
    assert_eq!(notes(&s), n + 1);
}
