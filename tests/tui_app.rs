//! TECHSPEC 7.1: App state and the initial selection (FR-19, A-27, I5).

mod common;

use brain_swap::core::keys::{Action, Key, KeyContext, KeyMap, KeyPress};
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{
    Effect, Input, Mode, Selection, context, initial_selection, key_press, update,
};
use common::tui::{TuiFixture, key, press, write_card};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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

fn work_4_9() -> brain_swap::tui::app::App {
    TuiFixture::new(Some("work_4_9")).app()
}

fn ev(code: KeyCode, m: KeyModifiers) -> Option<KeyPress> {
    key_press(&KeyEvent::new(code, m))
}

#[test]
fn ts7_1_quit_key_emits_quit() {
    assert_eq!(press(&mut work_4_9(), &["q"]), [Effect::Quit]);
    assert_eq!(press(&mut work_4_9(), &["esc"]), [Effect::Quit]);
}

#[test]
fn ts5_3_char_event_maps_to_char_key() {
    assert_eq!(
        ev(KeyCode::Char('j'), KeyModifiers::NONE),
        Some(KeyPress {
            key: Key::Char('j'),
            ctrl: false,
            alt: false,
            shift: false
        })
    );
}

#[test]
fn ts5_3_upper_case_matches_with_and_without_shift() {
    for m in [KeyModifiers::SHIFT, KeyModifiers::NONE] {
        let p = ev(KeyCode::Char('H'), m).unwrap();
        let action = KeyMap::defaults().action(KeyContext::Board, &p);
        assert_eq!(action, Some(Action::MoveLeft));
    }
}

#[test]
fn ts5_3_named_and_function_keys_map() {
    let cases = [
        (KeyCode::Enter, Key::Enter),
        (KeyCode::Esc, Key::Esc),
        (KeyCode::Tab, Key::Tab),
        (KeyCode::BackTab, Key::BackTab),
        (KeyCode::Char(' '), Key::Space),
        (KeyCode::Backspace, Key::Backspace),
        (KeyCode::Delete, Key::Delete),
        (KeyCode::Up, Key::Up),
        (KeyCode::Down, Key::Down),
        (KeyCode::Left, Key::Left),
        (KeyCode::Right, Key::Right),
        (KeyCode::Home, Key::Home),
        (KeyCode::End, Key::End),
        (KeyCode::PageUp, Key::PageUp),
        (KeyCode::PageDown, Key::PageDown),
        (KeyCode::F(1), Key::F(1)),
        (KeyCode::F(12), Key::F(12)),
    ];
    for (code, k) in cases {
        let p = ev(code, KeyModifiers::NONE).map(|p| p.key);
        assert_eq!(p, Some(k), "{code:?}");
    }
    assert_eq!(ev(KeyCode::F(13), KeyModifiers::NONE), None);
    assert_eq!(ev(KeyCode::Insert, KeyModifiers::NONE), None);
}

#[test]
fn ts5_3_ctrl_and_alt_carry_over() {
    let d = ev(KeyCode::Char('d'), KeyModifiers::CONTROL).unwrap();
    let x = ev(KeyCode::Char('x'), KeyModifiers::ALT).unwrap();
    assert!(d.ctrl && !d.alt);
    assert!(x.alt && !x.ctrl);
    assert_eq!(KeyMap::defaults().action(KeyContext::Board, &d), None);
}

#[test]
fn ts7_1_key_release_is_ignored() {
    let mut app = work_4_9();
    let mut q = key("q");
    q.kind = KeyEventKind::Release;
    assert_eq!(update(&mut app, Input::Key(q)), []);
}

#[test]
fn ts5_2_rebound_quit_key() {
    let mut app = work_4_9();
    let (keys, warnings) = KeyMap::build(&[("quit".to_string(), vec!["x".to_string()])]);
    assert_eq!(warnings, Vec::<String>::new());
    app.keys = keys;
    assert_eq!(press(&mut app, &["x"]), [Effect::Quit]);
    assert_eq!(press(&mut app, &["q"]), []);
}

#[test]
fn ts5_2_board_ignores_scroll_keys() {
    let mut app = work_4_9();
    assert_eq!(press(&mut app, &["ctrl+d", "ctrl+u"]), []);
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn ts5_2_mode_maps_to_key_context() {
    let card = CardId {
        letter: 'W',
        number: 1,
    };
    let back = || Box::new(Mode::Board);
    let cases = [
        (Mode::Board, Some(KeyContext::Board)),
        (
            Mode::Detail {
                card,
                note: 0,
                scroll: 0,
            },
            Some(KeyContext::Detail),
        ),
        (Mode::TemplatePick { sel: 0 }, Some(KeyContext::Picker)),
        (Mode::BoardPick { sel: 0 }, Some(KeyContext::Picker)),
        (
            Mode::TitleInput {
                template: "Feature".into(),
                buf: String::new(),
            },
            Some(KeyContext::TitleInput),
        ),
        (Mode::Help { back: back() }, None),
        (
            Mode::Message {
                text: "m".into(),
                back: back(),
            },
            None,
        ),
        (Mode::ConfigError("broken".into()), None),
    ];
    for (mode, ctx) in cases {
        assert_eq!(context(&mode), ctx, "{mode:?}");
    }
}

#[test]
fn ts5_3_backtab_binding_matches_shift_backtab_event() {
    let mut app = work_4_9();
    let (keys, warnings) = KeyMap::build(&[("quit".to_string(), vec!["backtab".to_string()])]);
    assert_eq!(warnings, Vec::<String>::new());
    app.keys = keys;
    let ev = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(update(&mut app, Input::Key(ev)), [Effect::Quit]);
}
