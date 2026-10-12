//! NFR-10: keyboard accessibility of the defaults and the shell screens (TECHSPEC 5.2, 7.3).

mod common;

use brain_swap::core::keys::{Action, KeyMap};
use brain_swap::core::time::parse_stamp;
use brain_swap::tui::app::{App, Effect, Mode};
use common::tui::{NOW, TuiFixture, assert_no_colour, press, render, rows};

fn board() -> App {
    TuiFixture::new(Some("work_4_9")).app()
}

fn with_mode(mode: Mode) -> App {
    let mut app = board();
    app.mode = mode;
    app
}

fn help() -> App {
    with_mode(Mode::Help {
        back: Box::new(Mode::Board),
    })
}

fn message() -> App {
    with_mode(Mode::Message {
        text: "pane closed".into(),
        back: Box::new(Mode::Board),
    })
}

fn config_error() -> App {
    let now = parse_stamp(NOW, &jiff::tz::TimeZone::UTC).unwrap();
    App::config_error("config x.toml:1:1: expected '='".into(), now)
}

#[test]
fn nfr10_every_action_has_a_default_key() {
    let keys = KeyMap::defaults();
    assert_eq!(Action::all().count(), 17);
    for a in Action::all() {
        assert!(!keys.keys(a).is_empty(), "{}", a.config_name());
    }
}

#[test]
fn nfr10_defaults_need_no_modifier_beyond_shift_except_scrolling() {
    let keys = KeyMap::defaults();
    for a in Action::all() {
        let scroll = matches!(a, Action::ScrollDown | Action::ScrollUp);
        for b in keys.keys(a) {
            assert_eq!(b.ctrl, scroll, "{} {b}", a.config_name());
            assert!(!b.alt, "{} {b}", a.config_name());
        }
    }
}

#[test]
fn nfr10_shell_modes_have_a_default_way_out() {
    for (mut app, k, out) in [
        (help(), "x", None),
        (message(), "x", None),
        (config_error(), "x", Some(Effect::Quit)),
        (board(), "q", Some(Effect::Quit)),
    ] {
        let effects = press(&mut app, &[k]);
        match out {
            None => {
                assert!(effects.is_empty(), "{k}");
                assert_eq!(app.mode, Mode::Board, "{k}");
            }
            Some(e) => assert_eq!(effects, [e], "{k}"),
        }
    }
}

#[test]
fn nfr10_help_fits_80x24() {
    let rows = rows(&render(&help(), 80, 24));
    for a in Action::all() {
        let name = format!(" {:<14}", a.config_name());
        assert!(
            rows[..23].iter().any(|r| r.starts_with(&name)),
            "{}",
            a.config_name()
        );
    }
}

#[test]
fn nfr10_shell_screens_use_no_colour() {
    let mut status = board();
    status.status = Some("move failed: card 4 is locked".into());
    for app in [help(), message(), config_error(), status] {
        assert_no_colour(&render(&app, 80, 24));
    }
}
