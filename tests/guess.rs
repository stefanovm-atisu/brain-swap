//! TECHSPEC 6.6 and FR-38: the best guess over a board's open cards.

mod common;

use std::path::{Path, PathBuf};

use brain_swap::core::board::load;
use brain_swap::core::guess::{Candidate, GuessInput, Tier, best_guess};
use brain_swap::core::model::Board;
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
