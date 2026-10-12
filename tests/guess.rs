//! TECHSPEC 6.6 and FR-38: the best guess over a board's open cards.

mod common;

use std::path::{Path, PathBuf};

use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::guess::{Candidate, GuessInput, Tier, best_guess};
use brain_swap::core::model::{Board, CardId, SessionRef};
use brain_swap::core::session::{read_reference, write_reference};
use common::env::core_env;
use common::mem_board::{mem_board, mem_card, mem_note};

const TDD: &[&str] = &["Todo", "Doing", "Done"];

/// No pane, no hint, cwd `/elsewhere`.
fn input() -> GuessInput {
    GuessInput {
        pane: None,
        cwd: PathBuf::from("/elsewhere"),
        home: PathBuf::from("/home/u"),
        hint: None,
    }
}

fn order(cs: &[Candidate]) -> Vec<(String, Tier)> {
    cs.iter().map(|c| (c.card.id.to_string(), c.tier)).collect()
}

fn guess(board: &Board) -> Vec<(String, Tier)> {
    order(&best_guess(board, &input()))
}

fn ids(board: &Board) -> Vec<String> {
    guess(board).into_iter().map(|(id, _)| id).collect()
}

fn board_1() -> Board {
    mem_board(
        TDD,
        vec![
            mem_card(
                "W-11",
                "Todo",
                None,
                vec![mem_note("2026-10-08T11:00:00+03:00", None, None)],
            ),
            mem_card(
                "W-12",
                "Doing",
                None,
                vec![mem_note("2026-10-08T10:00:00+03:00", None, None)],
            ),
        ],
    )
}

#[test]
fn fr38_middle_column_card_comes_before_the_rest() {
    assert_eq!(
        guess(&board_1()),
        vec![
            ("W-12".to_string(), Tier::MiddleColumn),
            ("W-11".to_string(), Tier::Other)
        ]
    );
}

#[test]
fn fr38_closed_cards_are_never_candidates() {
    let mut b = board_1();
    b.cards.push(mem_card(
        "W-9",
        "Done",
        None,
        vec![mem_note("12:00", None, None)],
    ));
    assert_eq!(ids(&b), ["W-12", "W-11"]);
}

#[test]
fn ts6_6_tier_ties_to_newest_activity() {
    let b = mem_board(
        TDD,
        vec![
            mem_card("W-3", "Doing", None, vec![mem_note("11:00", None, None)]),
            mem_card("W-4", "Doing", None, vec![mem_note("10:00", None, None)]),
        ],
    );
    assert_eq!(ids(&b), ["W-3", "W-4"]);
}

#[test]
fn ts6_6_equal_activity_ties_to_higher_number() {
    let b = mem_board(
        TDD,
        vec![
            mem_card(
                "W-7",
                "Doing",
                None,
                vec![mem_note("2026-10-08T10:00:00+03:00", None, None)],
            ),
            mem_card(
                "W-12",
                "Doing",
                None,
                vec![mem_note("2026-10-08T07:00:00+00:00", None, None)],
            ),
        ],
    );
    assert_eq!(ids(&b), ["W-12", "W-7"]);
}

#[test]
fn ts6_6_every_middle_column_is_tier_3() {
    let b = mem_board(
        &["Backlog", "Doing", "Review", "Done"],
        vec![
            mem_card("W-1", "Backlog", None, vec![mem_note("11:00", None, None)]),
            mem_card("W-2", "Doing", None, vec![mem_note("10:00", None, None)]),
            mem_card("W-3", "Review", None, vec![mem_note("09:00", None, None)]),
        ],
    );
    assert_eq!(
        guess(&b),
        vec![
            ("W-2".to_string(), Tier::MiddleColumn),
            ("W-3".to_string(), Tier::MiddleColumn),
            ("W-1".to_string(), Tier::Other)
        ]
    );
}

#[test]
fn ts6_6_two_column_board_has_no_middle() {
    let b = mem_board(
        &["Todo", "Done"],
        vec![
            mem_card("W-1", "Todo", None, vec![mem_note("11:00", None, None)]),
            mem_card("W-2", "Todo", None, vec![mem_note("10:00", None, None)]),
        ],
    );
    assert!(guess(&b).iter().all(|(_, t)| *t == Tier::Other));
    assert_eq!(guess(&b).len(), 2);
}

#[test]
fn ts6_6_extra_column_card_is_other() {
    let b = mem_board(
        TDD,
        vec![mem_card(
            "W-5",
            "Doign",
            None,
            vec![mem_note("11:00", None, None)],
        )],
    );
    assert_eq!(guess(&b), vec![("W-5".to_string(), Tier::Other)]);
}

#[test]
fn ts6_6_card_without_activity_sorts_last_in_tier() {
    let b = mem_board(
        TDD,
        vec![
            mem_card("W-1", "Todo", None, vec![mem_note("11:00", None, None)]),
            mem_card("W-2", "Todo", None, vec![]),
        ],
    );
    assert_eq!(ids(&b), ["W-1", "W-2"]);
}

#[test]
fn ts6_6_reason_and_label_per_tier() {
    let got: Vec<_> = [
        Tier::LinkedHere,
        Tier::SamePane,
        Tier::SameFolder,
        Tier::MiddleColumn,
        Tier::Other,
    ]
    .iter()
    .map(|t| (t.reason(), t.label()))
    .collect();
    assert_eq!(
        got,
        [
            ("linked_here", Some("linked here")),
            ("same_pane", Some("same pane")),
            ("same_folder", Some("same folder")),
            ("middle_column", None),
            ("other", None),
        ]
    );
}

#[test]
fn ts4_9_work_board_guess_order() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/boards/work_4_9");
    let tmp = tempfile::tempdir().unwrap();
    let b = load(
        "work",
        &dir,
        &core_env(tmp.path(), "2026-10-08T11:52:00+03:00"),
    )
    .unwrap();
    assert_eq!(
        guess(&b),
        vec![
            ("W-12".to_string(), Tier::MiddleColumn),
            ("W-11".to_string(), Tier::Other)
        ]
    );
}

fn at(pane: Option<&str>, cwd: &str, home: &str) -> GuessInput {
    GuessInput {
        pane: pane.map(str::to_string),
        cwd: PathBuf::from(cwd),
        home: PathBuf::from(home),
        hint: None,
    }
}

fn guess_at(board: &Board, input: &GuessInput) -> Vec<(String, Tier)> {
    order(&best_guess(board, input))
}

/// The tier of the single Todo card `W-1` with `notes`.
fn tier_of(input: &GuessInput, notes: Vec<brain_swap::core::model::Note>) -> Tier {
    let b = mem_board(TDD, vec![mem_card("W-1", "Todo", None, notes)]);
    guess_at(&b, input)[0].1
}

#[test]
fn fr38_same_pane_card_comes_first() {
    let b = mem_board(
        TDD,
        vec![
            mem_card(
                "W-7",
                "Doing",
                None,
                vec![mem_note("11:00", Some("w4V:p11"), None)],
            ),
            mem_card(
                "W-11",
                "Todo",
                None,
                vec![mem_note("09:00", Some("w4V:p9"), None)],
            ),
        ],
    );
    assert_eq!(
        guess_at(&b, &at(Some("w4V:p9"), "/elsewhere", "/home/u")),
        vec![
            ("W-11".to_string(), Tier::SamePane),
            ("W-7".to_string(), Tier::MiddleColumn)
        ]
    );
}

#[test]
fn ts6_6_same_pane_uses_latest_note_with_a_pane() {
    let i = at(Some("w4V:p9"), "/elsewhere", "/home/u");
    let t = tier_of(
        &i,
        vec![
            mem_note("09:00", Some("w4V:p9"), None),
            mem_note("10:00", Some("w4V:p11"), None),
        ],
    );
    assert_ne!(t, Tier::SamePane);
    let t = tier_of(
        &i,
        vec![
            mem_note("09:00", Some("w4V:p9"), None),
            mem_note("10:00", None, Some("/somewhere")),
        ],
    );
    assert_eq!(t, Tier::SamePane);
}

#[test]
fn ts6_6_no_current_pane_means_no_same_pane() {
    let t = tier_of(
        &at(None, "/elsewhere", "/home/u"),
        vec![mem_note("09:00", Some("w4V:p9"), None)],
    );
    assert_eq!(t, Tier::Other);
}

#[test]
fn fr38_same_folder_equal_cwd() {
    let t = tier_of(
        &at(None, "/work/billing", "/home/u"),
        vec![mem_note("09:00", None, Some("/work/billing"))],
    );
    assert_eq!(t, Tier::SameFolder);
}

#[test]
fn ts6_6_note_cwd_inside_current_cwd_is_same_folder() {
    let t = tier_of(
        &at(None, "/work/billing", "/home/u"),
        vec![mem_note("09:00", None, Some("/work/billing/src"))],
    );
    assert_eq!(t, Tier::SameFolder);
}

#[test]
fn ts6_6_current_cwd_inside_note_cwd_is_same_folder() {
    let t = tier_of(
        &at(None, "/work/billing/src/x", "/home/u"),
        vec![mem_note("09:00", None, Some("/work/billing"))],
    );
    assert_eq!(t, Tier::SameFolder);
}

#[test]
fn ts6_6_root_or_home_note_cwd_does_not_contain_current() {
    let t = tier_of(
        &at(None, "/home/u/proj", "/home/u"),
        vec![mem_note("09:00", None, Some("/home/u"))],
    );
    assert_eq!(t, Tier::Other);
    let t = tier_of(
        &at(None, "/tmp/x", "/home/u"),
        vec![mem_note("09:00", None, Some("/"))],
    );
    assert_eq!(t, Tier::Other);
}

#[test]
fn ts6_6_inside_is_component_wise() {
    let b = mem_board(
        TDD,
        vec![
            mem_card(
                "W-1",
                "Todo",
                None,
                vec![mem_note("09:00", None, Some("/work/bill"))],
            ),
            mem_card(
                "W-2",
                "Todo",
                None,
                vec![mem_note("10:00", None, Some("/work/billing2"))],
            ),
        ],
    );
    let g = guess_at(&b, &at(None, "/work/billing", "/home/u"));
    assert!(g.iter().all(|(_, t)| *t != Tier::SameFolder), "{g:?}");
}

#[test]
fn ts6_6_same_folder_uses_latest_note_with_a_cwd() {
    let i = at(None, "/work/billing", "/home/u");
    let t = tier_of(
        &i,
        vec![
            mem_note("09:00", None, Some("/work/billing")),
            mem_note("10:00", None, Some("/other")),
        ],
    );
    assert_ne!(t, Tier::SameFolder);
    let t = tier_of(
        &i,
        vec![
            mem_note("09:00", None, Some("/work/billing")),
            mem_note("10:00", None, None),
        ],
    );
    assert_eq!(t, Tier::SameFolder);
}

#[test]
fn ts6_6_current_home_contains_note_cwds_below_it() {
    let t = tier_of(
        &at(None, "/home/u", "/home/u"),
        vec![mem_note("09:00", None, Some("/home/u/proj"))],
    );
    assert_eq!(t, Tier::SameFolder);
}

#[test]
fn ts6_6_card_takes_its_best_tier() {
    let i = at(Some("w4V:p9"), "/work/billing", "/home/u");
    let one = |n| {
        let b = mem_board(TDD, vec![mem_card("W-1", "Doing", None, vec![n])]);
        guess_at(&b, &i)[0].1
    };
    assert_eq!(
        one(mem_note("09:00", Some("w4V:p9"), Some("/work/billing"))),
        Tier::SamePane
    );
    assert_eq!(
        one(mem_note("09:00", None, Some("/work/billing"))),
        Tier::SameFolder
    );
}

#[test]
fn ts6_6_tier_beats_activity() {
    let b = mem_board(
        TDD,
        vec![
            mem_card(
                "W-1",
                "Todo",
                None,
                vec![mem_note("08:00", Some("w4V:p9"), None)],
            ),
            mem_card(
                "W-2",
                "Todo",
                None,
                vec![mem_note("09:00", None, Some("/work/billing"))],
            ),
            mem_card(
                "W-3",
                "Doing",
                None,
                vec![mem_note("10:00", None, Some("/other"))],
            ),
            mem_card(
                "W-4",
                "Todo",
                None,
                vec![mem_note("11:00", None, Some("/other"))],
            ),
        ],
    );
    let g = guess_at(&b, &at(Some("w4V:p9"), "/work/billing", "/home/u"));
    let ids: Vec<_> = g.into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["W-1", "W-2", "W-3", "W-4"]);
}

#[test]
fn ts6_6_ties_within_same_pane_tier() {
    let i = at(Some("w4V:p9"), "/elsewhere", "/home/u");
    let two = |w5: &str| {
        let b = mem_board(
            TDD,
            vec![
                mem_card(
                    "W-5",
                    "Todo",
                    None,
                    vec![mem_note(w5, Some("w4V:p9"), None)],
                ),
                mem_card(
                    "W-8",
                    "Todo",
                    None,
                    vec![mem_note("10:00", Some("w4V:p9"), None)],
                ),
            ],
        );
        guess_at(&b, &i)
    };
    let same = two("10:00");
    assert!(same.iter().all(|(_, t)| *t == Tier::SamePane));
    assert_eq!(same[0].0, "W-8");
    assert_eq!(two("10:01")[0].0, "W-5");
}

fn hinted(pane: Option<&str>, hint: Option<&str>) -> GuessInput {
    GuessInput {
        hint: hint.map(|h| CardId::parse(h).unwrap()),
        ..at(pane, "/elsewhere", "/home/u")
    }
}

fn work_4_9(env: &Env) -> Board {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/boards/work_4_9");
    load("work", &dir, env).unwrap()
}

/// A herdr env in pane `w4V:p9`, cwd `/work/billing`, with session `old` linked to W-11 there.
fn linked_env(home: &Path) -> Env {
    let mut env = core_env(home, "2026-10-08T11:52:00+03:00");
    env.herdr_env = Some("1".to_string());
    env.herdr_pane_id = Some("w4V:p9".to_string());
    env.cwd = PathBuf::from("/work/billing");
    let r = SessionRef {
        card: CardId::parse("W-11").unwrap(),
        board: "work".to_string(),
        pane: Some("w4V:p9".to_string()),
        cwd: None,
        set_at: env.now.clone(),
    };
    write_reference(&env, "old", &r).unwrap();
    env
}

#[test]
fn fr38_hint_card_is_linked_here_and_first() {
    let b = mem_board(
        TDD,
        vec![
            mem_card("W-11", "Todo", None, vec![]),
            mem_card(
                "W-12",
                "Doing",
                None,
                vec![mem_note("10:00", Some("w4V:p9"), None)],
            ),
        ],
    );
    assert_eq!(
        guess_at(&b, &hinted(Some("w4V:p9"), Some("W-11"))),
        vec![
            ("W-11".to_string(), Tier::LinkedHere),
            ("W-12".to_string(), Tier::SamePane)
        ]
    );
}

#[test]
fn ts6_6_hint_to_card_not_open_here_gives_no_tier_0() {
    let tmp = tempfile::tempdir().unwrap();
    let b = work_4_9(&core_env(tmp.path(), "2026-10-08T11:52:00+03:00"));
    let plain = guess_at(&b, &hinted(None, None));
    for h in ["W-9", "H-3", "W-99"] {
        assert_eq!(guess_at(&b, &hinted(None, Some(h))), plain, "hint {h}");
    }
    assert!(plain.iter().all(|(_, t)| *t != Tier::LinkedHere));
}

#[test]
fn ts6_6_from_env_reads_hint_of_current_pane() {
    let tmp = tempfile::tempdir().unwrap();
    let env = linked_env(tmp.path());
    assert_eq!(
        GuessInput::from_env(&env),
        GuessInput {
            pane: Some("w4V:p9".to_string()),
            cwd: PathBuf::from("/work/billing"),
            home: env.home.clone(),
            hint: Some(CardId::parse("W-11").unwrap()),
        }
    );
}

#[test]
fn ts6_6_outside_herdr_no_pane_and_no_hint() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = linked_env(tmp.path());
    for v in [None, Some("0".to_string())] {
        env.herdr_env = v;
        let i = GuessInput::from_env(&env);
        assert_eq!((i.pane, i.hint), (None, None));
    }
}

#[test]
fn ts6_6_from_env_without_hint_file() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), "2026-10-08T11:52:00+03:00");
    env.herdr_env = Some("1".to_string());
    env.herdr_pane_id = Some("w4V:p9".to_string());
    assert_eq!(GuessInput::from_env(&env).hint, None);
    assert!(!env.state_dir().exists());
}

#[test]
fn as3_new_session_finds_previous_card_linked_here() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), "2026-10-08T11:52:00+03:00");
    env.herdr_env = Some("1".to_string());
    env.herdr_pane_id = Some("w4V:p11".to_string());
    env.cwd = PathBuf::from("/elsewhere");
    let b = work_4_9(&env);
    let r = SessionRef {
        card: CardId::parse("W-11").unwrap(),
        board: "work".to_string(),
        pane: Some("w4V:p11".to_string()),
        cwd: None,
        set_at: env.now.clone(),
    };
    write_reference(&env, "old", &r).unwrap();
    assert_eq!(read_reference(&env, "new"), None);
    let g = best_guess(&b, &GuessInput::from_env(&env));
    assert_eq!(g[0].card.id.to_string(), "W-11");
    assert_eq!(g[0].tier.label(), Some("linked here"));
    assert_eq!(g[1].card.id.to_string(), "W-12");
}
