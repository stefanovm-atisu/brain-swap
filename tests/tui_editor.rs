//! TECHSPEC 7.1 and 7.6: `e` runs the editor on a copy of the card and merges the save (FR-31, R6).

mod common;

use brain_swap::core::card_file::{check_writable, render_note};
use brain_swap::core::config;
use brain_swap::core::model::{CardId, NewNote};
use brain_swap::core::store::{lock_board, park};
use brain_swap::tui::Prepared;
use brain_swap::tui::app::{App, Effect, Input, Mode, Outcome, Selection, update};
use brain_swap::tui::editor::editor_argv;
use brain_swap::tui::runtime::Runtime;
use common::env::core_env;
use common::spawn::Scenario;
use common::tui::{
    FakeGuard, FakeLauncher, NOW, TuiFixture, fake_runtime_with, key, press, rows, scripted_editor,
};
use ratatui::backend::{Backend, TestBackend};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const W12: CardId = CardId {
    letter: 'W',
    number: 12,
};

fn work_4_9() -> TuiFixture {
    TuiFixture::new(Some("work_4_9"))
}

/// `W-12.md` with the title `migrate invoices to v14`.
fn edited(fx: &TuiFixture) -> String {
    fs::read_to_string(fx.board_dir.join("W-12.md"))
        .unwrap()
        .replace("# migrate invoices to v13", "# migrate invoices to v14")
}

/// The `work_4_9` app with W-12 (Doing) selected.
fn app(fx: &TuiFixture) -> App {
    let mut app = fx.app();
    app.sel = Selection {
        column: 1,
        card: Some(W12),
    };
    app
}

/// A runtime over `app` with `launcher`.
fn runtime(fx: &TuiFixture, launcher: &FakeLauncher) -> Runtime<TestBackend> {
    runtime_logged(fx, launcher, FakeGuard::default())
}

/// Like `runtime`, with `guard`, whose log the launcher may share.
fn runtime_logged(
    fx: &TuiFixture,
    launcher: &FakeLauncher,
    guard: FakeGuard,
) -> Runtime<TestBackend> {
    let prepared = Prepared {
        app: app(fx),
        config: Some(config::load(&fx.env).unwrap().config),
        templates: Vec::new(),
    };
    fake_runtime_with(&fx.env, prepared, 80, 24, Box::new(launcher.clone()), guard).0
}

/// Runs `e` on W-12 with a launcher that writes `edited` to the copy and exits 0.
fn edit_title(fx: &TuiFixture) -> (Runtime<TestBackend>, FakeLauncher) {
    let e = edited(fx);
    let launcher = FakeLauncher::new(Some(0), move |copy| fs::write(copy, &e).unwrap());
    let mut rt = runtime(fx, &launcher);
    assert_eq!(rt.dispatch(Input::Key(key("e"))), None);
    (rt, launcher)
}

#[test]
fn fr31_edit_key_emits_edit_for_selected_card() {
    let mut app = app(&work_4_9());
    assert_eq!(press(&mut app, &["e"]), [Effect::Edit(W12)]);
    app.mode = Mode::Detail {
        card: W12,
        note: 1,
        scroll: 0,
    };
    assert_eq!(press(&mut app, &["e"]), [Effect::Edit(W12)]);
    app.mode = Mode::Board;
    app.sel.card = None;
    assert_eq!(press(&mut app, &["e"]), []);
}

#[test]
fn ts7_6_card_bytes_are_copied_to_edit_folder() {
    let fx = work_4_9();
    let launcher = FakeLauncher::new(Some(0), |_| {});
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    let calls = launcher.calls.borrow();
    assert_eq!(calls.len(), 1);
    let (argv, bytes) = &calls[0];
    let copy = fx.home.join(".local/state/brain-swap/edit/work-W-12.md");
    assert_eq!(argv.last().unwrap(), copy.to_str().unwrap());
    assert_eq!(*bytes, fs::read(fx.board_dir.join("W-12.md")).unwrap());
}

#[test]
fn fr31_saved_edit_is_written_to_card() {
    let fx = work_4_9();
    let e = edited(&fx);
    let (rt, _) = edit_title(&fx);
    assert_eq!(fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap(), e);
    let app = rt.app();
    assert_eq!(
        app.board.card(W12).unwrap().title,
        "migrate invoices to v14"
    );
    assert_eq!(app.sel.card, Some(W12));
    assert_eq!(app.status, None);
    assert_eq!(app.mode, Mode::Board);
    // `Edited` selects the card even when the selection is elsewhere (W-11, the start-up one).
    let mut app = fx.app();
    assert_eq!(app.sel.card.map(|c| c.number), Some(11));
    assert_eq!(update(&mut app, Input::Done(Outcome::Edited(W12))), []);
    assert_eq!(
        app.sel,
        Selection {
            column: 1,
            card: Some(W12)
        }
    );
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn ts7_6_copy_deleted_after_successful_merge() {
    let fx = work_4_9();
    let (_, launcher) = edit_title(&fx);
    assert_eq!(launcher.calls.borrow().len(), 1);
    let copy = fx.home.join(".local/state/brain-swap/edit/work-W-12.md");
    assert!(!copy.exists());
}

#[test]
fn r6_note_parked_in_process_during_edit_is_kept() {
    let fx = work_4_9();
    let e = edited(&fx);
    let note = NewNote {
        at: fx.env.now.clone(),
        auto: false,
        doing: "parked mid-edit".into(),
        next: "rerun the migration test".into(),
        watch_out: String::new(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: None,
        session: None,
    };
    let block = render_note(&note, "\n").0;
    let (env, board, written) = (fx.env.clone(), fx.board(), e.clone());
    let launcher = FakeLauncher::new(Some(0), move |copy| {
        park(&env, &board, W12, &note).unwrap();
        fs::write(copy, &written).unwrap();
    });
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(
        fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap(),
        format!("{e}{block}")
    );
}

/// `editor_argv` with copy `/s/c.md` and the given `editor`, `$VISUAL` and `$EDITOR`.
fn argv(config_editor: &str, visual: Option<&str>, editor: Option<&str>) -> Vec<String> {
    let mut env = core_env(Path::new("/h"), NOW);
    env.visual = visual.map(String::from);
    env.editor = editor.map(String::from);
    editor_argv(config_editor, &env, Path::new("/s/c.md"))
}

#[test]
fn fr31_config_editor_wins() {
    assert_eq!(
        argv("nvim", Some("code"), Some("nano")),
        ["nvim", "/s/c.md"]
    );
}

#[test]
fn fr31_visual_used_when_config_empty() {
    assert_eq!(
        argv("", Some("code --wait"), None),
        ["code", "--wait", "/s/c.md"]
    );
}

#[test]
fn fr31_editor_variable_used_when_visual_unset() {
    assert_eq!(argv("", None, Some("nano")), ["nano", "/s/c.md"]);
}

#[test]
fn fr31_vi_when_nothing_set() {
    assert_eq!(argv("", None, None), ["vi", "/s/c.md"]);
}

#[test]
fn ts5_2_blank_values_fall_through() {
    assert_eq!(argv("  ", Some(""), Some("nano")), ["nano", "/s/c.md"]);
}

#[test]
fn ts5_2_editor_string_split_without_quoting() {
    assert_eq!(
        argv("\"my editor\" -f\t-x", None, None),
        ["\"my", "editor\"", "-f", "-x", "/s/c.md"]
    );
}

/// The kept or written copy of W-12 on board `work`.
fn copy(fx: &TuiFixture) -> std::path::PathBuf {
    fx.home.join(".local/state/brain-swap/edit/work-W-12.md")
}

/// Runs `e` on W-12 with a launcher that writes `edited` to the copy and exits 1.
fn edit_exit_1(fx: &TuiFixture) -> Runtime<TestBackend> {
    let e = edited(fx);
    let launcher = FakeLauncher::new(Some(1), move |copy| fs::write(copy, &e).unwrap());
    let mut rt = runtime(fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    rt
}

#[test]
fn ts7_6_nonzero_exit_writes_nothing_and_keeps_copy() {
    let fx = work_4_9();
    let before = fs::read(fx.board_dir.join("W-12.md")).unwrap();
    let rt = edit_exit_1(&fx);
    assert_eq!(fs::read(fx.board_dir.join("W-12.md")).unwrap(), before);
    assert_eq!(fs::read_to_string(copy(&fx)).unwrap(), edited(&fx));
    assert_eq!(rt.app().status.as_deref(), Some("editor: exit status: 1"));
}

#[test]
fn ts7_6_kept_copy_is_overwritten_by_next_edit() {
    let fx = work_4_9();
    edit_exit_1(&fx);
    let launcher = FakeLauncher::new(Some(1), |_| {});
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    let calls = launcher.calls.borrow();
    assert_eq!(calls[0].1, fs::read(fx.board_dir.join("W-12.md")).unwrap());
}

#[test]
fn ts7_6_unchanged_copy_writes_nothing() {
    let fx = work_4_9();
    let card = fx.board_dir.join("W-12.md");
    let (bytes, mtime) = (
        fs::read(&card).unwrap(),
        fs::metadata(&card).unwrap().modified().unwrap(),
    );
    let launcher = FakeLauncher::new(Some(0), |_| {});
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(fs::read(&card).unwrap(), bytes);
    assert_eq!(fs::metadata(&card).unwrap().modified().unwrap(), mtime);
    assert!(!copy(&fx).exists());
    assert_eq!(rt.app().status, None);
}

#[test]
fn ts7_6_launch_error_sets_status_and_keeps_copy() {
    let fx = work_4_9();
    let before = fs::read(fx.board_dir.join("W-12.md")).unwrap();
    let launcher = FakeLauncher::new(None, |_| {});
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    let status = rt.app().status.clone().unwrap();
    assert!(status.starts_with("editor: "), "{status}");
    assert_eq!(fs::read(fx.board_dir.join("W-12.md")).unwrap(), before);
    assert!(copy(&fx).exists());
}

#[test]
fn ts7_6_failed_merge_keeps_copy() {
    let fx = work_4_9();
    let e = edited(&fx).replace(
        "# migrate invoices to v14",
        "# migrate invoices to v14\n<<<<<<< HEAD",
    );
    let note = NewNote {
        at: fx.env.now.clone(),
        auto: false,
        doing: "parked mid-edit".into(),
        next: "rerun the migration test".into(),
        watch_out: String::new(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: None,
        session: None,
    };
    let parked = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let (env, board, written, seen) = (fx.env.clone(), fx.board(), e.clone(), parked.clone());
    let launcher = FakeLauncher::new(Some(0), move |copy| {
        park(&env, &board, W12, &note).unwrap();
        *seen.borrow_mut() = fs::read(board.path.join("W-12.md")).unwrap();
        fs::write(copy, &written).unwrap();
    });
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("e")));
    let unreadable = check_writable("W-12.md", e.as_bytes())
        .unwrap_err()
        .to_string();
    assert_eq!(rt.app().status.as_deref(), Some(unreadable.as_str()));
    assert_eq!(
        fs::read(fx.board_dir.join("W-12.md")).unwrap(),
        *parked.borrow()
    );
    assert_eq!(fs::read_to_string(copy(&fx)).unwrap(), e);
}

#[test]
fn ts7_6_missing_card_sets_status_without_launch() {
    let fx = work_4_9();
    let launcher = FakeLauncher::new(Some(0), |_| {});
    let mut rt = runtime(&fx, &launcher);
    fs::remove_file(fx.board_dir.join("W-12.md")).unwrap();
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(rt.app().status.as_deref(), Some("W-12 not found"));
    assert!(launcher.calls.borrow().is_empty());
}

/// A launcher exiting with `code` that logs `launch` into `guard`'s log.
fn logging(guard: &FakeGuard, code: Option<i32>) -> FakeLauncher {
    let log = guard.0.clone();
    FakeLauncher::new(code, move |_| log.borrow_mut().push("launch"))
}

#[test]
fn ts7_5_terminal_left_before_editor_and_reentered_after() {
    let fx = work_4_9();
    let guard = FakeGuard::default();
    let mut rt = runtime_logged(&fx, &logging(&guard, Some(0)), guard.clone());
    rt.draw().unwrap();
    let frame = rows(rt.terminal.backend().buffer());
    // The editor leaves the real screen blank; the next frame must repaint all of it.
    rt.terminal.backend_mut().clear().unwrap();
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(guard.0.borrow().as_slice(), ["leave", "launch", "enter"]);
    rt.draw().unwrap();
    assert_eq!(rows(rt.terminal.backend().buffer()), frame);
}

#[test]
fn ts7_5_terminal_reentered_after_failed_editor() {
    let fx = work_4_9();
    let guard = FakeGuard::default();
    runtime_logged(&fx, &logging(&guard, Some(1)), guard.clone()).dispatch(Input::Key(key("e")));
    assert_eq!(guard.0.borrow().as_slice(), ["leave", "launch", "enter"]);
    let guard = FakeGuard::default();
    let launcher = FakeLauncher::new(None, |_| {});
    runtime_logged(&fx, &launcher, guard.clone()).dispatch(Input::Key(key("e")));
    assert_eq!(guard.0.borrow().as_slice(), ["leave", "enter"]);
}

#[test]
fn ts10_no_board_lock_held_while_editor_runs() {
    let fx = work_4_9();
    let (env, dir) = (fx.env.clone(), fx.board_dir.clone());
    let launcher = FakeLauncher::new(Some(0), move |_| {
        let start = Instant::now();
        let lock = lock_board(&env, &dir).unwrap();
        assert!(start.elapsed() < Duration::from_millis(100));
        drop(lock);
    });
    runtime(&fx, &launcher).dispatch(Input::Key(key("e")));
    assert_eq!(launcher.calls.borrow().len(), 1);
}

#[test]
fn fr31_edit_from_detail_returns_to_detail() {
    let fx = work_4_9();
    let text = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    let (old, new) = (
        text.find("### 2026-10-08T10:31:05").unwrap(),
        text.find("### 2026-10-08T11:12:40").unwrap(),
    );
    let e = format!("{}{}", &text[..old], &text[new..]);
    let launcher = FakeLauncher::new(Some(0), move |copy| fs::write(copy, &e).unwrap());
    let mut rt = runtime(&fx, &launcher);
    rt.dispatch(Input::Key(key("o")));
    assert!(matches!(
        rt.app().mode,
        Mode::Detail {
            card: W12,
            note: 1,
            ..
        }
    ));
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(rt.app().board.card(W12).unwrap().notes.len(), 1);
    assert!(matches!(
        rt.app().mode,
        Mode::Detail {
            card: W12,
            note: 0,
            ..
        }
    ));
}

/// Parks on W-12 through the real CLI, the note on a quoted heredoc.
const PARK: &str = "@BS@ park --card W-12 --session test <<'EOF'
Doing: parked mid-edit
Next: check the merge
Watch out: nothing
EOF";

/// Runs `e` on W-12 under the scripted editor with `steps`, saving `edited` and exiting 0.
fn scripted(fx: &TuiFixture, steps: &[&str], edited: &str) -> Runtime<TestBackend> {
    let tmp = fx.home.parent().unwrap();
    let path = tmp.join("edited.md");
    fs::write(&path, edited).unwrap();
    let editor = scripted_editor(&Scenario::new(tmp), tmp, steps, &path, 0);
    let mut rt = fx.set_editor(&editor);
    rt.dispatch(Input::Key(key("l")));
    assert_eq!(rt.app().sel.card, Some(W12));
    rt.dispatch(Input::Key(key("e")));
    rt
}

#[test]
fn fr31_note_parked_mid_edit_survives() {
    let fx = work_4_9();
    let e = edited(&fx);
    let rt = scripted(&fx, &[PARK], &e);
    let text = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    let block = text
        .strip_prefix(e.as_str())
        .expect("bytes outside the note");
    assert!(block.starts_with("\n### "), "{block}");
    assert_eq!(block.matches("### ").count(), 1);
    assert!(
        block.contains("- Doing: parked mid-edit\n- Next: check the merge\n- Watch out: nothing\n")
    );
    assert!(!copy(&fx).exists());
    let card = rt.app().board.card(W12).unwrap();
    assert_eq!(card.title, "migrate invoices to v14");
    assert_eq!(card.notes.len(), 3);
    assert_eq!(card.notes[2].doing, "parked mid-edit");
    assert_eq!(rt.app().sel.card, Some(W12));
}

#[test]
fn r6_file_column_change_mid_edit_is_kept() {
    let fx = work_4_9();
    let e = fs::read_to_string(fx.board_dir.join("W-12.md"))
        .unwrap()
        .replace("the SQL only.", "the SQL and the deploy list.");
    scripted(&fx, &["@BS@ move W-12 Done"], &e);
    let text = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    assert_eq!(text, e.replace("column: Doing", "column: Done"));
}

#[test]
fn r6_user_column_change_wins_over_file() {
    let fx = work_4_9();
    let e = fs::read_to_string(fx.board_dir.join("W-12.md"))
        .unwrap()
        .replace("column: Doing", "column: Todo");
    scripted(&fx, &["@BS@ move W-12 Done"], &e);
    let text = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    assert!(text.starts_with("---\ncolumn: Todo\n"), "{text}");
}

#[test]
fn ts12_1_scripted_editor_is_hermetic() {
    let fx = work_4_9();
    let tmp = fx.home.parent().unwrap();
    let out = tmp.join("env.txt");
    let e = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    scripted(&fx, &[&format!("env > {}", out.display())], &e);
    // `/bin/sh` exports its own `PWD` (dash) or also `SHLVL`, `_` and `OLDPWD` (bash).
    let shell = ["PWD", "OLDPWD", "SHLVL", "_"];
    let got: Vec<String> = fs::read_to_string(&out)
        .unwrap()
        .lines()
        .filter(|l| !shell.iter().any(|v| l.starts_with(&format!("{v}="))))
        .map(String::from)
        .collect();
    let want: Vec<String> = Scenario::new(tmp)
        .vars()
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    let mut sorted = got.clone();
    sorted.sort();
    assert_eq!(sorted, want);
    let leaked = |l: &&String| {
        l.starts_with("CLAUDE_") || (l.starts_with("HERDR_") && !l.starts_with("HERDR_BIN_PATH="))
    };
    assert_eq!(got.iter().filter(leaked).count(), 0, "{got:?}");
}
