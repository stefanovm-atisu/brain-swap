//! TECHSPEC 7.1 and 7.2: the Help and Message screens (FR-32, NFR-10).

mod common;

use brain_swap::core::keys::KeyMap;
use brain_swap::tui::app::{App, Mode};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};

fn work_4_9() -> App {
    TuiFixture::new(Some("work_4_9")).app()
}

fn help() -> App {
    let mut app = work_4_9();
    app.mode = Mode::Help {
        back: Box::new(Mode::Board),
    };
    app
}

fn message() -> App {
    let mut app = work_4_9();
    app.mode = Mode::Message {
        text: "pane closed\n/w".into(),
        back: Box::new(Mode::Board),
    };
    app
}

#[test]
fn fr32_help_key_opens_help() {
    let mut app = work_4_9();
    assert!(press(&mut app, &["?"]).is_empty());
    assert_eq!(
        app.mode,
        Mode::Help {
            back: Box::new(Mode::Board)
        }
    );
}

#[test]
fn ts7_1_any_key_in_help_returns() {
    for k in ["x", "q", "esc"] {
        let mut app = help();
        assert!(press(&mut app, &[k]).is_empty(), "{k}");
        assert_eq!(app.mode, Mode::Board, "{k}");
    }
}

#[test]
fn fr32_help_lists_every_action_with_default_keys() {
    let rows = rows(&render(&help(), 80, 24));
    assert_eq!(rows[0], " help");
    assert_eq!(
        rows[1..18],
        [
            " up            k, up",
            " down          j, down",
            " left          h, left",
            " right         l, right",
            " move_left     H",
            " move_right    L",
            " open          o",
            " jump          enter",
            " new           n",
            " edit          e",
            " switch_board  b",
            " scroll_down   ctrl+d",
            " scroll_up     ctrl+u",
            " help          ?",
            " quit          q, esc",
            " confirm       enter",
            " cancel        esc",
        ]
    );
}

#[test]
fn fr32_help_shows_current_keys_after_rebinding() {
    let mut app = help();
    app.keys = KeyMap::build(&[("open".into(), vec!["x".into()])]).0;
    let rows = rows(&render(&app, 80, 24));
    let open = rows.iter().find(|r| r.starts_with(" open ")).unwrap();
    assert_eq!(open, " open          x");
}

#[test]
fn fr32_help_lists_all_warnings() {
    let mut app = help();
    app.warnings = vec!["config: unknown key 'colour'".into()];
    app.board.warnings = vec!["W-13: unknown column 'Doign'".into()];
    let rows = rows(&render(&app, 120, 40));
    let at = rows.iter().position(|r| r == " warnings:").unwrap();
    assert_eq!(
        rows[at + 1..at + 3],
        [
            " config: unknown key 'colour'",
            " W-13: unknown column 'Doign'"
        ]
    );
    assert_eq!(rows[at + 3], "");
}

#[test]
fn ts7_1_any_key_in_message_returns() {
    let mut app = message();
    assert!(press(&mut app, &["q"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn ts7_2_message_shows_each_line() {
    let rows = rows(&render(&message(), 80, 24));
    assert_eq!(rows[..2], [" pane closed", " /w"]);
}

#[test]
fn ts7_2_help_and_message_snapshots() {
    let mut with_warning = help();
    with_warning.warnings = vec!["config: unknown key 'colour'".into()];
    for (name, app) in [("help", with_warning), ("message", message())] {
        for (w, h) in [(80, 24), (120, 40)] {
            let buf = render(&app, w, h);
            assert_no_colour(&buf);
            insta::assert_snapshot!(format!("{name}_{w}x{h}"), rows(&buf).join("\n"));
        }
    }
}
