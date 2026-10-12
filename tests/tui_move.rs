//! FR-29, TECHSPEC 4.7, 7.1 and 7.3: `H` and `L` move the selected card.

mod common;

use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Effect, Input, Mode, Selection};
use common::tui::{TuiFixture, key, press};
use std::fs;

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn sel(column: usize, card: &str) -> Selection {
    Selection {
        column,
        card: Some(id(card)),
    }
}

fn mv(card: &str, column: &str) -> Effect {
    Effect::Move(id(card), column.to_string())
}

fn fixture(name: &str, s: Selection) -> App {
    let mut app = TuiFixture::new(Some(name)).app();
    app.sel = s;
    app
}

#[test]
fn fr29_l_emits_move_to_next_column() {
    let mut app = fixture("work_4_9", sel(0, "W-11"));
    assert_eq!(press(&mut app, &["L"]), [mv("W-11", "Doing")]);
}

#[test]
fn fr29_h_emits_move_to_previous_column() {
    let mut app = fixture("work_4_9", sel(1, "W-12"));
    assert_eq!(press(&mut app, &["H"]), [mv("W-12", "Todo")]);
}

#[test]
fn ts7_3_h_on_first_column_does_nothing() {
    let mut app = fixture("work_4_9", sel(0, "W-11"));
    assert_eq!(press(&mut app, &["H"]), []);
}

#[test]
fn ts7_3_l_on_last_configured_column_does_nothing() {
    let mut app = fixture("work_4_9", sel(2, "W-9"));
    assert_eq!(press(&mut app, &["L"]), []);
}

#[test]
fn ts4_7_moves_from_extra_column_target_first_column() {
    let mut app = TuiFixture::new(Some("as7")).app();
    let extra = app
        .board
        .column_list()
        .iter()
        .position(|c| c.name == "Doign (unknown)")
        .unwrap();
    app.sel = sel(extra, "W-13");
    assert_eq!(press(&mut app, &["H"]), [mv("W-13", "Todo")]);
    assert_eq!(press(&mut app, &["L"]), [mv("W-13", "Todo")]);
}

#[test]
fn fr29_move_without_selected_card_does_nothing() {
    let mut app = TuiFixture::new(None).app();
    assert_eq!(app.sel.card, None);
    assert_eq!(press(&mut app, &["L"]), []);
}

#[test]
fn fr29_moved_card_stays_selected() {
    let fx = TuiFixture::new(Some("work_4_9"));
    let (mut rt, _, _) = fx.runtime(80, 24);
    assert_eq!(rt.app().sel, sel(0, "W-11"));
    assert_eq!(rt.dispatch(Input::Key(key("L"))), None);
    let text = fs::read_to_string(fx.board_dir.join("W-11.md")).unwrap();
    assert!(text.contains("\ncolumn: Doing\n"), "{text}");
    assert_eq!(rt.app().sel, sel(1, "W-11"));
    assert_eq!(rt.app().status, None);
    assert_eq!(rt.app().mode, Mode::Board);
}

#[test]
fn fr29_move_failure_sets_status_line() {
    let fx = TuiFixture::new(Some("work_4_9"));
    let (mut rt, _, _) = fx.runtime(80, 24);
    let path = fx.board_dir.join("W-11.md");
    let bytes = b"---\ncolumn: Todo\n---\n# x\n<<<<<<< ours\n".to_vec();
    fs::write(&path, &bytes).unwrap();
    rt.dispatch(Input::Key(key("L")));
    assert_eq!(
        rt.app().status.as_deref(),
        Some("W-11.md:5: conflict marker, read only")
    );
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(rt.app().sel, sel(0, "W-11"));
}

#[test]
fn ts5_2_rebound_move_keys() {
    let mut app = fixture("work_4_9", sel(0, "W-11"));
    app.keys = KeyMap::build(&[("move_right".into(), vec!["m".into()])]).0;
    assert_eq!(press(&mut app, &["m"]), [mv("W-11", "Doing")]);
    assert_eq!(press(&mut app, &["L"]), []);
}
