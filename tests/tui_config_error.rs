//! TECHSPEC 7.1 and 7.2: the ConfigError screen (FR-45, A-23).

mod common;

use brain_swap::core::time::{Stamp, parse_stamp};
use brain_swap::tui::app::{App, Effect, Input, Mode, update};
use common::tui::{NOW, TuiFixture, assert_no_colour, press, render, rows};
use std::path::PathBuf;

const TEXT: &str = "config /h/.config/brain-swap/config.toml:4:7: expected '='";

fn now() -> Stamp {
    parse_stamp(NOW, &jiff::tz::TimeZone::UTC).unwrap()
}

fn app() -> App {
    App::config_error(TEXT.into(), now())
}

#[test]
fn fr45_config_error_any_key_emits_only_quit() {
    for k in ["n", "L", "e", "enter", "b", "q"] {
        let mut app = app();
        assert_eq!(press(&mut app, &[k]), [Effect::Quit], "{k}");
    }
}

#[test]
fn fr45_config_error_screen_shows_error() {
    for (w, h) in [(80, 24), (40, 24)] {
        let rows = rows(&render(&app(), w, h));
        let shown: Vec<&str> = rows[..h as usize - 1]
            .iter()
            .map(|r| r.trim())
            .filter(|r| !r.is_empty())
            .collect();
        assert_eq!(shown.join(" "), TEXT, "{w}x{h}");
    }
}

#[test]
fn fr45_config_error_ignores_reload_and_tick() {
    let mut app = app();
    let board = TuiFixture::new(Some("work_4_9")).board();
    let later = now()
        .checked_add(jiff::SignedDuration::from_mins(1))
        .unwrap();
    assert!(update(&mut app, Input::Reloaded(board)).is_empty());
    assert!(update(&mut app, Input::Tick(later)).is_empty());
    assert_eq!(app.mode, Mode::ConfigError(TEXT.into()));
    assert!(app.board.cards.is_empty());
    assert_eq!(app.board.name, "");
    assert_eq!(app.board.path, PathBuf::new());
}

#[test]
fn fr45_config_error_snapshots() {
    for (w, h) in [(80, 24), (120, 40)] {
        let buf = render(&app(), w, h);
        assert_no_colour(&buf);
        insta::assert_snapshot!(format!("config_error_{w}x{h}"), rows(&buf).join("\n"));
    }
}
