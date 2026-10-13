//! TECHSPEC 7.1 and 7.7: the jump key in Board and Detail (FR-25, FR-28).

mod common;

use brain_swap::adapters::herdr::PaneStatus;
use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Effect, Input, Mode, Outcome, Selection, update};
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

const BILLING: &str = "/home/smilen/Work/ati.billing";

fn not_jumped(status: PaneStatus, cwd: Option<&str>) -> Input {
    Input::Done(Outcome::NotJumped(status, cwd.map(Into::into)))
}

fn message(text: &str, back: Mode) -> Mode {
    Mode::Message {
        text: text.into(),
        back: Box::new(back),
    }
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

#[test]
fn fr59_jumped_outcome_quits() {
    let mut app = board_on(&work(), 1, 12);
    press(&mut app, &["enter"]);
    assert_eq!(
        update(&mut app, Input::Done(Outcome::Jumped)),
        [Effect::Quit]
    );
}

#[test]
fn fr61_closed_outcome_shows_pane_closed_cwd_and_link_hint() {
    let mut app = board_on(&work(), 1, 12);
    assert!(update(&mut app, not_jumped(PaneStatus::Closed, Some(BILLING))).is_empty());
    let text = format!("pane closed\n{BILLING}\nopen a session there and run /bs-link W-12");
    assert_eq!(app.mode, message(&text, Mode::Board));
    let buf = render(&app, 80, 24);
    assert_no_colour(&buf);
    insta::assert_snapshot!(rows(&buf).join("\n"));
}

#[test]
fn fr62_outside_outcome_shows_not_in_herdr_and_cwd() {
    let mut app = board_on(&work(), 1, 12);
    update(
        &mut app,
        not_jumped(PaneStatus::OutsideHerdr, Some(BILLING)),
    );
    let text = format!("not in herdr\n{BILLING}");
    assert_eq!(app.mode, message(&text, Mode::Board));
}

#[test]
fn fr62_no_pane_outcome_shows_no_pane_recorded_and_cwd() {
    let mut app = board_on(&work(), 2, 9);
    let cwd = "/home/smilen/Work/private/brain-swap";
    update(&mut app, not_jumped(PaneStatus::NoPane, Some(cwd)));
    let text = format!("no pane recorded\n{cwd}");
    assert_eq!(app.mode, message(&text, Mode::Board));
}

#[test]
fn fr63_not_responding_outcome_shows_message_and_cwd() {
    let mut app = board_on(&work(), 1, 12);
    update(
        &mut app,
        not_jumped(PaneStatus::NotResponding, Some(BILLING)),
    );
    let text = format!("herdr not responding\n{BILLING}");
    assert_eq!(app.mode, message(&text, Mode::Board));
    let buf = render(&app, 80, 24);
    assert_no_colour(&buf);
    insta::assert_snapshot!(rows(&buf).join("\n"));
}

#[test]
fn ts7_7_herdr_error_outcome_shows_the_message() {
    let mut app = board_on(&work(), 1, 12);
    let status = PaneStatus::HerdrError("socket closed".into());
    update(&mut app, not_jumped(status, Some(BILLING)));
    let Mode::Message { text, .. } = &app.mode else {
        panic!("{:?}", app.mode);
    };
    assert_eq!(text.lines().next(), Some("herdr: socket closed"));
}

#[test]
fn ts7_7_failed_focus_outcome_shows_pane_and_cwd() {
    let mut app = board_on(&work(), 1, 12);
    let outcome = Outcome::FocusFailed("w4V:p9".into(), Some(BILLING.into()));
    assert!(update(&mut app, Input::Done(outcome)).is_empty());
    let text = format!("focus failed: w4V:p9\n{BILLING}");
    assert_eq!(app.mode, message(&text, Mode::Board));
}

#[test]
fn ts7_7_message_without_cwd_has_one_line() {
    let mut app = board_on(&work(), 1, 12);
    update(&mut app, not_jumped(PaneStatus::NoPane, None));
    assert_eq!(app.mode, message("no pane recorded", Mode::Board));
}

#[test]
fn ts7_7_outcome_in_detail_returns_to_detail() {
    let mut app = board_on(&work(), 1, 12);
    press(&mut app, &["o", "j"]);
    let detail = app.mode.clone();
    assert!(matches!(detail, Mode::Detail { note: 0, .. }));
    update(&mut app, not_jumped(PaneStatus::Closed, Some(BILLING)));
    assert!(matches!(&app.mode, Mode::Message { back, .. } if **back == detail));
    press(&mut app, &["x"]);
    assert_eq!(app.mode, detail);
}

#[test]
fn fr63_tui_stays_usable_after_the_message() {
    let mut app = board_on(&work(), 1, 12);
    update(
        &mut app,
        not_jumped(PaneStatus::NotResponding, Some(BILLING)),
    );
    assert!(matches!(app.mode, Mode::Message { .. }));
    press(&mut app, &["x"]);
    assert_eq!(app.mode, Mode::Board);
    press(&mut app, &["l"]);
    assert_eq!(app.sel.card, Some(id(9)));
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(9), 0)]);
}

#[test]
fn ts7_7_jumping_status_is_cleared() {
    let mut app = board_on(&work(), 1, 12);
    assert_eq!(press(&mut app, &["enter"]), [Effect::Jump(id(12), 1)]);
    update(&mut app, not_jumped(PaneStatus::Closed, Some(BILLING)));
    assert_eq!(app.status, None);
}
