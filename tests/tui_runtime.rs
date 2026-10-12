//! TECHSPEC 7.1 and 7.5: the runtime loop and the terminal guard (FR-45, 12.1).

mod common;

use brain_swap::tui::Prepared;
use brain_swap::tui::app::{App, Input, Outcome};
use common::tui::{Ev, TuiFixture, fake_runtime};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

fn work_4_9() -> TuiFixture {
    TuiFixture::new(Some("work_4_9"))
}

#[test]
fn ts7_5_terminal_entered_and_restored_on_quit() {
    let (mut rt, guard, events) = work_4_9().runtime(80, 24);
    events.push([Ev::Key("q")]);
    assert_eq!(rt.run(), 0);
    assert_eq!(*guard.0.borrow(), ["enter", "leave"]);
}

#[test]
fn fr45_config_error_any_key_exits_1() {
    let fx = work_4_9();
    let prepared = Prepared {
        app: App::config_error("config broken".into(), fx.env.now.clone()),
        config: None,
        templates: Vec::new(),
    };
    let (mut rt, guard, events) = fake_runtime(&fx.env, prepared, 80, 24);
    events.push([Ev::Key("x")]);
    assert_eq!(rt.run(), 1);
    assert_eq!(*guard.0.borrow(), ["enter", "leave"]);
}

#[test]
fn ts7_5_terminal_restored_on_panic() {
    let (mut rt, guard, events) = work_4_9().runtime(80, 24);
    events.push([Ev::Timeout, Ev::Panic]);
    assert!(catch_unwind(AssertUnwindSafe(|| rt.run())).is_err());
    assert_eq!(guard.0.borrow().last(), Some(&"leave"));
}

#[test]
fn ts7_5_terminal_restored_on_event_error() {
    let (mut rt, guard, events) = work_4_9().runtime(80, 24);
    events.push([Ev::Error]);
    assert_eq!(rt.run(), 1);
    assert_eq!(*guard.0.borrow(), ["enter", "leave"]);
}

#[test]
fn ts7_1_input_polled_every_250_ms() {
    let (mut rt, _guard, events) = work_4_9().runtime(80, 24);
    events.push([Ev::Timeout, Ev::Timeout, Ev::Key("q")]);
    rt.run();
    assert_eq!(*events.timeouts.borrow(), [Duration::from_millis(250); 3]);
}

#[test]
fn ts7_1_screen_redrawn_after_input() {
    let (mut rt, _guard, _events) = work_4_9().runtime(80, 24);
    rt.dispatch(Input::Done(Outcome::Failed("x".into())));
    rt.draw().unwrap();
    let rows = common::tui::rows(rt.terminal.backend().buffer());
    assert_eq!(rows[23], " x");
}
