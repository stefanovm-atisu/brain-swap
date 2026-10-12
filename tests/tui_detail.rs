//! TECHSPEC 5.2, 7.1 and 7.2: opening the detail view (FR-26, FR-27).

mod common;

use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Mode, Selection};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};

const W12: CardId = CardId {
    letter: 'W',
    number: 12,
};

fn w12_detail() -> Mode {
    Mode::Detail {
        card: W12,
        note: 1,
        scroll: 0,
    }
}

/// The `work_4_9` app with W-12 (Doing) selected.
fn board_on_w12() -> App {
    let mut app = TuiFixture::new(Some("work_4_9")).app();
    app.sel = Selection {
        column: 1,
        card: Some(W12),
    };
    app
}

fn detail_on_w12() -> App {
    let mut app = board_on_w12();
    press(&mut app, &["o"]);
    app
}

fn detail_rows() -> Vec<String> {
    rows(&render(&detail_on_w12(), 80, 24))
}

#[test]
fn fr26_open_shows_newest_note() {
    let mut app = board_on_w12();
    assert!(press(&mut app, &["o"]).is_empty());
    assert_eq!(app.mode, w12_detail());
}

#[test]
fn fr26_open_without_selected_card_does_nothing() {
    let mut app = TuiFixture::new(None).app();
    assert!(press(&mut app, &["o"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr27_detail_header_shows_id_title_column_and_board() {
    let buf = render(&detail_on_w12(), 80, 24);
    assert_eq!(rows(&buf)[0], " W-12 migrate invoices to v13 (Doing, work)");
    assert_no_colour(&buf);
}

#[test]
fn fr26_note_line_shows_indicator_stamp_and_age() {
    assert_eq!(
        detail_rows()[1],
        " note 2/2, 2026-10-08T11:12:40+03:00, 39 min ago"
    );
}

#[test]
fn fr27_detail_shows_three_parts() {
    assert_eq!(
        detail_rows()[2..5],
        [
            " Doing:      migration test passes on staging",
            " Next:       open the MR, ask the DBA for the v13 deploy list",
            " Watch out:  nothing new",
        ]
    );
}

#[test]
fn fr27_where_line_shows_cwd_and_pane() {
    assert_eq!(
        detail_rows()[5],
        " Where: /home/smilen/Work/ati.billing (pane w4V:p9)"
    );
}

#[test]
fn ts5_2_quit_in_detail_returns_to_board() {
    for k in ["q", "esc"] {
        let mut app = detail_on_w12();
        assert_eq!(app.mode, w12_detail(), "{k}");
        assert!(press(&mut app, &[k]).is_empty(), "{k}");
        assert_eq!(app.mode, Mode::Board, "{k}");
        assert_eq!(app.sel.card, Some(W12), "{k}");
    }
}

#[test]
fn ts5_2_board_keys_do_nothing_in_detail() {
    let mut app = detail_on_w12();
    assert!(press(&mut app, &["n", "H", "L", "b", "l"]).is_empty());
    assert_eq!(app.mode, w12_detail());
}
