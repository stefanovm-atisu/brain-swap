//! TECHSPEC 7.1: App state and the initial selection (FR-19, A-27, I5).

mod common;

use brain_swap::core::model::CardId;
use brain_swap::tui::app::{Mode, Selection, initial_selection};
use common::tui::{TuiFixture, write_card};

const D: &str = "2026-10-08T";

fn sel(column: usize, card: Option<u32>) -> Selection {
    Selection {
        column,
        card: card.map(|number| CardId {
            letter: 'W',
            number,
        }),
    }
}

fn at(time: &str) -> String {
    format!("{D}{time}+03:00")
}

/// W-1 Todo no notes, W-2 Doing 11:00, W-3 Todo 10:00, W-4 Done 11:30.
fn newest_board() -> TuiFixture {
    let fx = TuiFixture::new(None);
    let d = &fx.board_dir;
    write_card(d, "W-1", "Todo", &at("09:00:00"), &[]);
    write_card(
        d,
        "W-2",
        "Doing",
        &at("08:00:00"),
        &[(&at("11:00:00"), "n")],
    );
    write_card(d, "W-3", "Todo", &at("08:00:00"), &[(&at("10:00:00"), "n")]);
    write_card(d, "W-4", "Done", &at("08:00:00"), &[(&at("11:30:00"), "n")]);
    fx
}

/// W-1 and W-2 both in Done.
fn closed_board() -> TuiFixture {
    let fx = TuiFixture::new(None);
    write_card(&fx.board_dir, "W-1", "Done", &at("09:00:00"), &[]);
    write_card(&fx.board_dir, "W-2", "Done", &at("10:00:00"), &[]);
    fx
}

#[test]
fn fr19_selects_open_card_with_newest_activity() {
    let app = newest_board().app();
    assert_eq!(app.sel, sel(1, Some(2)));
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr19_activity_tie_selects_higher_number() {
    let fx = TuiFixture::new(None);
    let t = at("10:00:00");
    write_card(&fx.board_dir, "W-5", "Todo", &at("08:00:00"), &[(&t, "n")]);
    write_card(&fx.board_dir, "W-7", "Doing", &at("08:00:00"), &[(&t, "n")]);
    assert_eq!(fx.app().sel, sel(1, Some(7)));
}

#[test]
fn fr19_no_open_card_focuses_first_column() {
    assert_eq!(closed_board().app().sel, sel(0, None));
}

#[test]
fn fr19_empty_board_focuses_first_column() {
    let fx = TuiFixture::new(None);
    assert!(!fx.board_dir.exists());
    assert_eq!(fx.app().sel, sel(0, None));
}

#[test]
fn i5_card_in_extra_column_can_be_selected_first() {
    let fx = TuiFixture::new(None);
    write_card(&fx.board_dir, "W-1", "Todo", &at("09:00:00"), &[]);
    write_card(
        &fx.board_dir,
        "W-2",
        "Doign",
        &at("08:00:00"),
        &[(&at("11:00:00"), "n")],
    );
    let app = fx.app();
    assert_eq!(app.sel, sel(3, Some(2)));
    assert_eq!(app.board.column_list()[3].name, "Doign (unknown)");
}

#[test]
fn fr19_initial_selection_snapshot() {
    let both = [newest_board().board(), closed_board().board()].map(|b| initial_selection(&b));
    insta::assert_debug_snapshot!("initial_selection", both);
}
