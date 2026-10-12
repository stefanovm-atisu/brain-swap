//! TECHSPEC 5.2 and 7.3: Board navigation, vertical scroll and the column window.

mod common;

use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Selection};
use brain_swap::tui::view_board::visible_columns;
use common::tui::{TuiFixture, press, render, rows, write_card};

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn sel(column: usize, card: &str) -> Selection {
    Selection {
        column,
        card: Some(id(card)),
    }
}

fn at(time: &str) -> String {
    format!("2026-10-08T{time}:00+03:00")
}

/// Todo holds W-3, W-2, W-1 (notes at 10:20, 10:10, 10:00); `doing` adds W-4 in Doing.
fn todo_board(doing: bool) -> App {
    let fx = TuiFixture::new(None);
    let d = &fx.board_dir;
    for (n, t) in [("W-1", "10:00"), ("W-2", "10:10"), ("W-3", "10:20")] {
        write_card(d, n, "Todo", &at("09:00"), &[(&at(t), "n")]);
    }
    if doing {
        write_card(d, "W-4", "Doing", &at("09:00"), &[]);
    }
    let mut app = fx.app();
    app.sel = sel(0, "W-3");
    app
}

fn fixture(name: &str, s: Selection) -> App {
    let mut app = TuiFixture::new(Some(name)).app();
    app.sel = s;
    app
}

#[test]
fn ts7_3_down_moves_to_next_card_in_column() {
    let mut app = todo_board(false);
    assert_eq!(press(&mut app, &["j"]), []);
    assert_eq!(app.sel, sel(0, "W-2"));
}

#[test]
fn ts7_3_up_and_down_stop_at_ends() {
    let mut app = todo_board(false);
    press(&mut app, &["k"]);
    assert_eq!(app.sel, sel(0, "W-3"));
    app.sel = sel(0, "W-1");
    press(&mut app, &["j"]);
    assert_eq!(app.sel, sel(0, "W-1"));
}

#[test]
fn ts7_3_right_and_left_move_focus_one_column() {
    let mut app = fixture("work_4_9", sel(0, "W-11"));
    press(&mut app, &["l"]);
    assert_eq!(app.sel, sel(1, "W-12"));
    press(&mut app, &["h"]);
    assert_eq!(app.sel, sel(0, "W-11"));
}

#[test]
fn ts7_3_left_and_right_stop_at_first_and_last_column() {
    let mut app = fixture("work_4_9", sel(0, "W-11"));
    press(&mut app, &["h"]);
    assert_eq!(app.sel, sel(0, "W-11"));
    let mut app = fixture("as7", sel(0, "W-11"));
    let last = app.board.column_list().len() - 1;
    assert!(app.board.column_list()[last].unknown);
    app.sel = sel(last, "W-13");
    press(&mut app, &["l"]);
    assert_eq!(app.sel, sel(last, "W-13"));
}

#[test]
fn ts7_3_column_change_keeps_row_clamped() {
    let mut app = todo_board(true);
    app.sel = sel(0, "W-1");
    press(&mut app, &["l"]);
    assert_eq!(app.sel, sel(1, "W-4"));
}

#[test]
fn ts7_3_empty_column_has_no_selection() {
    let mut app = todo_board(false);
    press(&mut app, &["l"]);
    let empty = Selection {
        column: 1,
        card: None,
    };
    assert_eq!(app.sel, empty);
    assert_eq!(press(&mut app, &["j", "k"]), []);
    assert_eq!(app.sel, empty);
}

#[test]
fn ts7_3_column_scrolls_to_keep_selection_visible() {
    let fx = TuiFixture::new(None);
    for n in 1..=30 {
        let note = format!("09:{n:02}");
        write_card(
            &fx.board_dir,
            &format!("W-{n}"),
            "Todo",
            &at("08:00"),
            &[(&at(&note), "n")],
        );
    }
    write_card(&fx.board_dir, "W-31", "Doing", &at("08:00"), &[]);
    let mut app = fx.app();
    assert_eq!(app.sel, sel(0, "W-30"));
    press(&mut app, &["j"; 25]);
    assert_eq!(app.sel, sel(0, "W-5"));
    let screen = rows(&render(&app, 80, 24)).join("\n");
    assert!(screen.contains("W-5 card W-5"), "{screen}");
    assert!(!screen.contains("W-30"), "{screen}");
    assert!(screen.contains("W-31 card W-31"), "{screen}");
}

#[test]
fn ts7_3_visible_columns_centre_on_focus() {
    assert_eq!(visible_columns(6, 0, 80), 0..3);
    assert_eq!(visible_columns(6, 3, 80), 2..5);
    assert_eq!(visible_columns(6, 5, 80), 3..6);
}

#[test]
fn ts7_3_narrow_columns_scroll_around_focused() {
    let app = fixture("six_columns", sel(3, "S-4"));
    let row = &rows(&render(&app, 80, 24))[1];
    for h in ["C (1)", "> D (1)", "E (1)"] {
        assert!(row.contains(h), "{h} in {row}");
    }
    for h in ["A (", "B (", "F ("] {
        assert!(!row.contains(h), "{h} in {row}");
    }
}

#[test]
fn ts5_2_rebound_navigation_keys() {
    let mut app = todo_board(false);
    let (keys, warnings) = KeyMap::build(&[("down".to_string(), vec!["x".to_string()])]);
    assert_eq!(warnings, Vec::<String>::new());
    app.keys = keys;
    press(&mut app, &["x"]);
    assert_eq!(app.sel, sel(0, "W-2"));
    press(&mut app, &["j"]);
    assert_eq!(app.sel, sel(0, "W-2"));
}
