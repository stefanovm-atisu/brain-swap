//! TECHSPEC 7.1 and 7.2: the frame, the status line and `Failed` outcomes (NFR-10).

mod common;

use brain_swap::tui::app::{App, Input, Outcome, update};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};

fn work_4_9() -> App {
    TuiFixture::new(Some("work_4_9")).app()
}

fn hello() -> App {
    let mut app = work_4_9();
    app.status = Some("hello".into());
    app
}

fn failed(text: &str) -> Input {
    Input::Done(Outcome::Failed(text.into()))
}

#[test]
fn ts7_2_status_line_is_last_row() {
    assert_eq!(rows(&render(&hello(), 80, 24))[23], " hello");
}

#[test]
fn ts7_1_failed_outcome_sets_status_line() {
    let mut app = work_4_9();
    let (mode, sel) = (app.mode.clone(), app.sel.clone());
    let effects = update(&mut app, failed("board busy, try again"));
    assert_eq!(app.status.as_deref(), Some("board busy, try again"));
    assert_eq!((app.mode, app.sel), (mode, sel));
    assert!(effects.is_empty());
}

#[test]
fn ts7_1_failed_texts_are_shown_verbatim() {
    for text in ["W-12.md: verify failed", "editor: exit status: 1"] {
        let mut app = work_4_9();
        update(&mut app, failed(text));
        for (w, h) in [(80, 24), (120, 40)] {
            let rows = rows(&render(&app, w, h));
            assert_eq!(rows.last().unwrap(), &format!(" {text}"), "{w}x{h}");
        }
    }
}

#[test]
fn ts7_1_key_press_clears_status() {
    let mut app = hello();
    assert!(press(&mut app, &["z"]).is_empty());
    assert_eq!(app.status, None);
}

#[test]
fn nfr10_status_line_uses_no_colour() {
    assert_no_colour(&render(&hello(), 80, 24));
}
