//! TECHSPEC 7.1 and 7.7: the jump key in Board and Detail (FR-25, FR-28).

mod common;

use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Effect, Mode, Selection};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};

fn id(number: u32) -> CardId {
    CardId {
        letter: 'W',
        number,
    }
}

/// The `work_4_9` app with `card` selected in `column`.
fn board_on(fx: &TuiFixture, column: usize, card: u32) -> App {
    let mut app = fx.app();
    app.sel = Selection {
        column,
        card: Some(id(card)),
    };
    app
}

fn work() -> TuiFixture {
    TuiFixture::new(Some("work_4_9"))
}

fn no_note_yet(back: Mode) -> Mode {
    Mode::Message {
        text: "no note yet".into(),
        back: Box::new(back),
    }
}

#[test]
fn fr25_enter_on_board_emits_jump_to_the_jump_note() {
    let mut app = board_on(&work(), 1, 12);
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(12), 1)]);
    assert_eq!(app.status.as_deref(), Some("jumping..."));
    let buf = render(&app, 80, 24);
    assert_no_colour(&buf);
    insta::assert_snapshot!(rows(&buf).join("\n"));
}

#[test]
fn fr25_jump_skips_a_newer_unplaced_note() {
    let fx = work();
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jump/W-14.md");
    std::fs::copy(from, fx.board_dir.join("W-14.md")).unwrap();
    let mut app = board_on(&fx, 1, 14);
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(14), 1)]);
}

#[test]
fn ts7_7_board_jump_without_jump_note_shows_no_note_yet() {
    let mut app = board_on(&work(), 0, 11);
    assert!(press(&mut app, &["enter"]).is_empty());
    assert_eq!(app.mode, no_note_yet(Mode::Board));
}

#[test]
fn fr28_detail_jump_targets_the_shown_note() {
    let mut app = board_on(&work(), 1, 12);
    press(&mut app, &["o", "j"]);
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(12), 0)]);
}

#[test]
fn fr28_detail_opens_on_the_newest_note_as_target() {
    let mut app = board_on(&work(), 1, 12);
    press(&mut app, &["o"]);
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(12), 1)]);
}

#[test]
fn ts7_7_detail_jump_without_notes_shows_no_note_yet() {
    let mut app = board_on(&work(), 0, 11);
    press(&mut app, &["o"]);
    let detail = app.mode.clone();
    assert!(matches!(detail, Mode::Detail { .. }));
    assert!(press(&mut app, &["enter"]).is_empty());
    assert_eq!(app.mode, no_note_yet(detail));
}

#[test]
fn fr25_jump_follows_the_configured_key() {
    let mut app = board_on(&work(), 1, 12);
    app.keys = KeyMap::build(&[("jump".into(), vec!["g".into()])]).0;
    assert_eq!(press(&mut app, &["g"]), [Effect::Jump(id(12), 1)]);
    let effects = press(&mut app, &["enter"]);
    assert!(!effects.iter().any(|e| matches!(e, Effect::Jump(..))));
}

#[test]
fn ts7_7_board_jump_without_a_selected_card_does_nothing() {
    let mut app = TuiFixture::new(None).app();
    assert_eq!(app.sel.card, None);
    let status = app.status.clone();
    assert!(press(&mut app, &["enter"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
    assert_eq!(app.status, status);
}
