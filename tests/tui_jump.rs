//! TECHSPEC 7.1 and 7.7: the jump key in Board and Detail (FR-25, FR-28).

mod common;

use brain_swap::adapters::herdr::{BUDGET, PaneStatus};
use brain_swap::adapters::runner::{Answer, FakeRunner, RunResult, Runner};
use brain_swap::core::env::Env;
use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::Prepared;
use brain_swap::tui::app::{App, Effect, Input, Mode, Outcome, Selection, update};
use brain_swap::tui::runtime::{Runtime, run_jump};
use common::tui::{
    Ev, FakeGuard, NoLauncher, TuiFixture, assert_no_colour, fake_runtime_with, key, press, render,
    rows,
};
use ratatui::backend::TestBackend;
use std::time::Duration;

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

const AGENT: &str = include_str!("fixtures/herdr/pane_agent.json");
const SHELL: &str = include_str!("fixtures/herdr/pane_shell.json");
const NOT_FOUND: &str = include_str!("fixtures/herdr/pane_not_found.stderr");
const WS_LIST: &str = include_str!("fixtures/herdr/workspace_list.json");
const PANES_W4V: &str = include_str!("fixtures/herdr/pane_list_w4V.json");
const PANES_W7Q: &str = include_str!("fixtures/herdr/pane_list_w7Q.json");

/// The fixture's env with `HERDR_ENV=1`.
fn in_herdr(fx: &TuiFixture) -> Env {
    let mut env = fx.env.clone();
    env.herdr_env = Some("1".into());
    env
}

/// `run_jump` on `work_4_9`'s card `card`, note `note`, inside herdr when `herdr`.
fn jump_on(fx: &TuiFixture, card: u32, note: usize, herdr: bool, fake: &FakeRunner) -> Outcome {
    let env = if herdr { in_herdr(fx) } else { fx.env.clone() };
    let board = fx.board();
    run_jump(&env, board.card(id(card)).unwrap(), note, fake)
}

/// `pane get w4V:p9` answering `pane_agent.json`.
fn agent_pane() -> FakeRunner {
    FakeRunner::new().on("pane get w4V:p9", Answer::ok(AGENT))
}

fn billing() -> Option<std::path::PathBuf> {
    Some(BILLING.into())
}

#[test]
fn fr59_live_agent_pane_is_jumped() {
    let fake = agent_pane().on("agent focus w4V:p9", Answer::ok(""));
    assert_eq!(jump_on(&work(), 12, 1, true, &fake), Outcome::Jumped);
    assert_eq!(fake.calls(), ["pane get w4V:p9", "agent focus w4V:p9"]);
}

#[test]
fn fr59_live_shell_pane_is_jumped_by_tab() {
    let shell = SHELL
        .replace("w4V:p3", "w4V:p9")
        .replace("w4V:t2", "w4V:t1");
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", Answer::ok(&shell))
        .on("workspace focus w4V", Answer::ok(""))
        .on("tab focus w4V:t1", Answer::ok(""));
    assert_eq!(jump_on(&work(), 12, 1, true, &fake), Outcome::Jumped);
    assert_eq!(
        fake.calls(),
        ["pane get w4V:p9", "workspace focus w4V", "tab focus w4V:t1"]
    );
}

#[test]
fn fr60_moved_pane_is_jumped() {
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", Answer::fail(1, NOT_FOUND))
        .on("workspace list", Answer::ok(WS_LIST))
        .on("pane list --workspace w4V", Answer::ok(PANES_W4V))
        .on("pane list --workspace w7Q", Answer::ok(PANES_W7Q))
        .on("agent focus w7Q:p2", Answer::ok(""));
    assert_eq!(jump_on(&work(), 12, 1, true, &fake), Outcome::Jumped);
    assert_eq!(fake.calls().last().unwrap(), "agent focus w7Q:p2");
}

#[test]
fn ts7_7_failed_focus_is_focus_failed_with_cwd() {
    let fake = agent_pane()
        .on("agent focus w4V:p9", Answer::fail(1, ""))
        .on("workspace focus w4V", Answer::fail(1, ""));
    assert_eq!(
        jump_on(&work(), 12, 1, true, &fake),
        Outcome::FocusFailed("w4V:p9".into(), billing())
    );
}

#[test]
fn ts7_7_timed_out_focus_is_not_responding() {
    let fake = agent_pane().on("agent focus w4V:p9", Answer::Hang);
    assert_eq!(
        jump_on(&work(), 12, 1, true, &fake),
        Outcome::NotJumped(PaneStatus::NotResponding, billing())
    );
}

#[test]
fn fr61_closed_pane_is_not_jumped_without_focus() {
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", Answer::fail(1, NOT_FOUND))
        .on(
            "workspace list",
            Answer::ok(r#"{"result":{"workspaces":[]}}"#),
        );
    assert_eq!(
        jump_on(&work(), 12, 1, true, &fake),
        Outcome::NotJumped(PaneStatus::Closed, billing())
    );
    assert!(!fake.calls().iter().any(|c| c.contains("focus")));
}

#[test]
fn fr62_outside_herdr_is_not_jumped_without_a_call() {
    let fake = FakeRunner::new();
    assert_eq!(
        jump_on(&work(), 12, 1, false, &fake),
        Outcome::NotJumped(PaneStatus::OutsideHerdr, billing())
    );
    assert!(fake.calls().is_empty());
}

#[test]
fn fr62_note_without_pane_is_not_jumped() {
    let cwd = "/home/smilen/Work/private/brain-swap";
    assert_eq!(
        jump_on(&work(), 9, 0, true, &FakeRunner::new()),
        Outcome::NotJumped(PaneStatus::NoPane, Some(cwd.into()))
    );
}

#[test]
fn as11_hang_is_not_jumped_within_3_seconds() {
    let slow = Answer::fail(1, NOT_FOUND).taking(Duration::from_millis(1500));
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", slow)
        .on("workspace list", Answer::Hang);
    assert_eq!(
        jump_on(&work(), 12, 1, true, &fake),
        Outcome::NotJumped(PaneStatus::NotResponding, billing())
    );
    // A hang takes its whole timeout; `pane get` took 1.5 s.
    let took = Duration::from_millis(1500) + fake.timeouts()[1];
    assert!(took <= BUDGET, "{took:?}");
}

#[test]
fn ts9_3_locate_and_focus_share_one_budget() {
    let slow = Answer::ok(AGENT).taking(Duration::from_millis(1500));
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", slow)
        .on("agent focus w4V:p9", Answer::ok(""));
    assert_eq!(jump_on(&work(), 12, 1, true, &fake), Outcome::Jumped);
    assert_eq!(fake.timeouts()[1], Duration::from_millis(1500));
}

#[test]
fn fr28_run_jump_uses_the_given_note() {
    let fx = work();
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jump/W-14.md");
    std::fs::copy(from, fx.board_dir.join("W-14.md")).unwrap();
    let fake = FakeRunner::new()
        .on("pane get w4V:p3", Answer::ok(SHELL))
        .on("workspace focus w4V", Answer::ok(""))
        .on("tab focus w4V:t2", Answer::ok(""));
    jump_on(&fx, 14, 0, true, &fake);
    assert_eq!(fake.calls()[0], "pane get w4V:p3");
}

/// A runtime on `work_4_9` inside herdr with W-12 selected and `fake` as its runner.
fn jump_runtime(
    fx: &TuiFixture,
    fake: FakeRunner,
) -> (
    brain_swap::tui::runtime::Runtime<ratatui::backend::TestBackend>,
    FakeGuard,
    common::tui::FakeEvents,
) {
    let prepared = Prepared {
        app: board_on(fx, 1, 12),
        config: Some(brain_swap::core::config::load(&fx.env).unwrap().config),
        templates: Vec::new(),
    };
    let guard = FakeGuard::default();
    let env = in_herdr(fx);
    fake_runtime_with(
        &env,
        prepared,
        80,
        24,
        Box::new(NoLauncher),
        guard,
        Box::new(fake),
    )
}

#[test]
fn fr59_jumped_outcome_ends_the_runtime_with_exit_0() {
    let fake = agent_pane().on("agent focus w4V:p9", Answer::ok(""));
    let (mut rt, guard, events) = jump_runtime(&work(), fake);
    events.push([Ev::Key("enter")]);
    assert_eq!(rt.run(), 0);
    let restores = guard.0.borrow().iter().filter(|e| **e == "leave").count();
    assert_eq!(restores, 1);
}

#[test]
fn fr63_tui_stays_open_after_not_responding() {
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::Hang);
    let (mut rt, _guard, _events) = jump_runtime(&work(), fake);
    assert_eq!(rt.dispatch(Input::Key(key("enter"))), None);
    rt.draw().unwrap();
    let frame = rows(rt.terminal.backend().buffer()).join("\n");
    assert!(frame.contains("herdr not responding"), "{frame}");
    assert!(frame.contains(BILLING), "{frame}");
    assert_eq!(rt.dispatch(Input::Key(key("x"))), None);
    assert_eq!(rt.dispatch(Input::Key(key("q"))), Some(0));
}

/// Delegates to `fake` and logs each call into `log`, interleaved with the guard's events.
struct Recording {
    fake: FakeRunner,
    log: FakeGuard,
}

impl Runner for Recording {
    fn run(&self, args: &[&str], timeout: Duration) -> RunResult {
        let call: &'static str = Box::leak(args.join(" ").into_boxed_str());
        self.log.0.borrow_mut().push(call);
        self.fake.run(args, timeout)
    }
}

/// `jump_runtime` whose guard log also records every herdr call.
fn recording_runtime(
    fake: FakeRunner,
) -> (Runtime<TestBackend>, FakeGuard, common::tui::FakeEvents) {
    let fx = work();
    let prepared = Prepared {
        app: board_on(&fx, 1, 12),
        config: Some(brain_swap::core::config::load(&fx.env).unwrap().config),
        templates: Vec::new(),
    };
    let guard = FakeGuard::default();
    let log = guard.clone();
    let runner = Box::new(Recording { fake, log });
    let env = in_herdr(&fx);
    fake_runtime_with(&env, prepared, 80, 24, Box::new(NoLauncher), guard, runner)
}

fn frame(rt: &mut Runtime<TestBackend>) -> String {
    rt.draw().unwrap();
    rows(rt.terminal.backend().buffer()).join("\n")
}

#[test]
fn ts7_7_focus_precedes_restore() {
    let fake = agent_pane().on("agent focus w4V:p9", Answer::ok(""));
    let (mut rt, guard, events) = recording_runtime(fake);
    events.push([Ev::Key("enter")]);
    assert_eq!(rt.run(), 0);
    assert_eq!(
        *guard.0.borrow(),
        ["enter", "pane get w4V:p9", "agent focus w4V:p9", "leave"]
    );
}

#[test]
fn ts7_7_failed_focus_never_restores() {
    let fake = agent_pane()
        .on("agent focus w4V:p9", Answer::fail(1, ""))
        .on("workspace focus w4V", Answer::fail(1, ""));
    let (mut rt, guard, _events) = recording_runtime(fake);
    assert_eq!(rt.dispatch(Input::Key(key("enter"))), None);
    assert!(
        !guard.0.borrow().contains(&"leave"),
        "{:?}",
        guard.0.borrow()
    );
    let frame = frame(&mut rt);
    assert!(frame.contains("focus failed: w4V:p9"), "{frame}");
}

#[test]
fn ts7_7_status_without_a_live_pane_never_restores() {
    let fake = FakeRunner::new()
        .on("pane get w4V:p9", Answer::fail(1, NOT_FOUND))
        .on(
            "workspace list",
            Answer::ok(r#"{"result":{"workspaces":[]}}"#),
        );
    let (mut rt, guard, _events) = recording_runtime(fake);
    assert_eq!(rt.dispatch(Input::Key(key("enter"))), None);
    assert!(
        !guard.0.borrow().contains(&"leave"),
        "{:?}",
        guard.0.borrow()
    );
    let frame = frame(&mut rt);
    assert!(frame.contains("pane closed"), "{frame}");
}
