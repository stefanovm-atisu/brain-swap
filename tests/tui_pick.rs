//! TECHSPEC 5.2, 5.4, 7.1 and 7.2: the template picker (FR-30, NFR-10).

mod common;

use brain_swap::core::config::load;
use brain_swap::tui::app::{App, Mode};
use brain_swap::tui::{TuiStart, prepare};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};
use ratatui::style::Modifier;
use std::fs;

fn work_4_9() -> App {
    TuiFixture::new(Some("work_4_9")).app()
}

fn picker() -> App {
    let mut app = work_4_9();
    press(&mut app, &["n"]);
    app
}

/// The `work_4_9` app through `tui::prepare`, with `file` holding `bytes` in the templates folder.
fn prepared_with(file: &str, bytes: &[u8]) -> App {
    let mut fx = TuiFixture::new(Some("work_4_9"));
    let dir = fx.env.templates_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(file), bytes).unwrap();
    let start = TuiStart {
        config: load(&fx.env),
        board: None,
    };
    prepare(&mut fx.env, start).unwrap().app
}

fn spike(key: &str) -> App {
    prepared_with(
        "spike.md",
        format!("---\nname: Spike\n{key}---\n## Goal\n").as_bytes(),
    )
}

fn title_input(template: &str) -> Mode {
    Mode::TitleInput {
        template: template.into(),
        buf: String::new(),
    }
}

#[test]
fn fr30_new_key_opens_template_pick() {
    let mut app = work_4_9();
    assert!(press(&mut app, &["n"]).is_empty());
    assert_eq!(app.mode, Mode::TemplatePick { sel: 0 });
}

#[test]
fn fr30_hotkey_opens_title_input_for_template() {
    let mut app = picker();
    assert!(press(&mut app, &["b"]).is_empty());
    assert_eq!(app.mode, title_input("Bug"));
}

#[test]
fn ts5_4_keyless_template_chosen_with_down_and_confirm() {
    let mut app = spike("");
    press(&mut app, &["n", "j", "j", "j", "j", "enter"]);
    assert_eq!(app.mode, title_input("Spike"));
}

#[test]
fn ts5_2_template_pick_up_down_clamp() {
    let mut app = picker();
    press(&mut app, &["k"]);
    assert_eq!(app.mode, Mode::TemplatePick { sel: 0 });
    press(&mut app, &["j", "j", "j", "j", "j"]);
    assert_eq!(app.mode, Mode::TemplatePick { sel: 3 });
}

#[test]
fn ts5_2_cancel_in_template_pick_returns_to_board() {
    let mut app = picker();
    assert!(press(&mut app, &["esc"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr30_user_template_hotkey_works() {
    let mut app = spike("key: s\n");
    press(&mut app, &["n"]);
    let frame = rows(&render(&app, 80, 24));
    let listed: Vec<&String> = frame[..23].iter().filter(|r| !r.is_empty()).collect();
    assert_eq!(listed.last().unwrap().as_str(), " s  Spike", "{frame:?}");
    press(&mut app, &["s"]);
    assert_eq!(app.mode, title_input("Spike"));
}

#[test]
fn ts5_2_colliding_hotkey_dropped_with_warning() {
    let mut app = spike("key: j\n");
    press(&mut app, &["n", "j"]);
    assert_eq!(app.mode, Mode::TemplatePick { sel: 1 });
    let w = "template Spike: hotkey 'j' collides with down, dropped";
    assert!(app.warnings.iter().any(|x| x == w), "{:?}", app.warnings);
}

#[test]
fn ts5_4_template_load_warning_reaches_help() {
    let app = prepared_with("bad.md", b"---\nname: Bad\n---\n\xff\n");
    assert!(
        app.warnings
            .iter()
            .any(|w| w.starts_with("template bad.md: ")),
        "{:?}",
        app.warnings
    );
}

#[test]
fn ts5_2_board_keys_do_nothing_in_template_pick() {
    let mut app = picker();
    assert!(press(&mut app, &["q", "H", "o"]).is_empty());
    assert_eq!(app.mode, Mode::TemplatePick { sel: 0 });
}

#[test]
fn fr30_template_pick_snapshots() {
    let mut app = work_4_9();
    app.sel.column = 0;
    app.sel.card = app.board.cards_in(0).first().map(|c| c.id);
    press(&mut app, &["n"]);
    for (w, h) in [(80, 24), (120, 40)] {
        let buf = render(&app, w, h);
        assert_no_colour(&buf);
        assert!(buf[(0, 1)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(0, 2)].modifier.contains(Modifier::REVERSED));
        insta::assert_snapshot!(format!("template_pick_{w}x{h}"), rows(&buf).join("\n"));
    }
}
