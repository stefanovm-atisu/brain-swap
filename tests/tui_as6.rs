//! PRD AS-6 end to end: TUI and CLI only, no Claude (FR-25, FR-59), no terminal, no real herdr.

mod common;

use brain_swap::adapters::runner::{Answer, FakeRunner};
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{Effect, Input, Outcome, update};
use brain_swap::tui::runtime::run_jump;
use common::spawn::{Scenario, brain_swap};
use common::tui::{TuiFixture, key};
use std::fs;

const SHELL: &str = include_str!("fixtures/herdr/pane_shell.json");

#[test]
fn as6_tui_card_park_from_shell_and_enter_focuses_the_tab() {
    let fx = TuiFixture::new(Some("as6"));
    let w12 = CardId {
        letter: 'W',
        number: 12,
    };
    let (mut rt, _, _) = fx.runtime(80, 24);
    let title = "migrate invoices to v13";
    let mut keys = vec!["n".to_string(), "f".to_string()];
    keys.extend(title.chars().map(|c| match c {
        ' ' => "space".to_string(),
        c => c.to_string(),
    }));
    keys.push("enter".into());
    for k in &keys {
        assert_eq!(rt.dispatch(Input::Key(key(k))), None);
    }
    let card = fx.board().card(w12).cloned().expect("W-12.md");
    assert_eq!((card.column.as_str(), card.title.as_str()), ("Todo", title));
    assert_eq!(rt.app().sel.card, Some(w12));

    assert_eq!(rt.dispatch(Input::Key(key("L"))), None);
    assert_eq!(fx.board().card(w12).unwrap().column, "Doing");
    assert_eq!(rt.app().sel.card, Some(w12));

    let s = Scenario::new(fx.home.parent().unwrap())
        .herdr("w4V:p3", "w4V:t2", "w4V")
        .fake_scenario("as6-shell");
    let out = brain_swap(&s)
        .args(["park", "--card", "W-12"])
        .current_dir(&fx.home)
        .write_stdin("Doing: a\nNext: b\nWatch out: c\n")
        .assert()
        .code(0)
        .get_output()
        .clone();
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("parked to W-12: {title}\n")
    );
    let text = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    let heading = text.lines().rfind(|l| l.starts_with("### ")).unwrap();
    assert!(!heading.contains("auto"), "{heading}");
    let place = text
        .lines()
        .rfind(|l| l.starts_with("<!-- where:"))
        .unwrap();
    assert!(
        place.contains("pane=w4V:p3 tab=w4V:t2 workspace=w4V"),
        "{place}"
    );
    assert!(!place.contains("session="), "{place}");

    let mut app = rt.app().clone();
    update(&mut app, Input::Reloaded(fx.board()));
    assert_eq!(
        update(&mut app, Input::Key(key("enter"))),
        [Effect::Jump(w12, 0)]
    );
    let fake = FakeRunner::new()
        .on("pane get w4V:p3", Answer::ok(SHELL))
        .on("workspace focus w4V", Answer::ok(""))
        .on("tab focus w4V:t2", Answer::ok(""));
    let mut env = fx.env.clone();
    env.herdr_env = Some("1".into());
    let board = fx.board();
    let outcome = run_jump(&env, board.card(w12).unwrap(), 0, &fake);
    assert_eq!(outcome, Outcome::Jumped);
    assert_eq!(
        fake.calls(),
        ["pane get w4V:p3", "workspace focus w4V", "tab focus w4V:t2"]
    );
    assert_eq!(update(&mut app, Input::Done(outcome)), [Effect::Quit]);
}
