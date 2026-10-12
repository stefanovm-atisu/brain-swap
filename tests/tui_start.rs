//! TECHSPEC 6.2, 6.3, 7.1, 10 and 11: the bare invocation's gate and start-up (FR-19, FR-44, FR-45).

mod common;

use brain_swap::core::config::{Loaded, load};
use brain_swap::core::error::Error;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Mode};
use brain_swap::tui::{Prepared, TuiStart, gate, prepare, run};
use common::tui::{TuiFixture, press, render, rows, write_card};
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn work_4_9() -> TuiFixture {
    TuiFixture::new(Some("work_4_9"))
}

fn loaded(fx: &TuiFixture) -> Loaded {
    load(&fx.env).unwrap()
}

fn prep(fx: &mut TuiFixture, loaded: Loaded, board: Option<&str>) -> Prepared {
    let start = TuiStart {
        config: Ok(loaded),
        board: board.map(String::from),
    };
    prepare(&mut fx.env, start).unwrap()
}

/// Replaces `from` with `to` in the fixture's config file.
fn edit_config(fx: &TuiFixture, from: &str, to: &str) {
    let path = fx.env.config_file();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{from}");
    fs::write(&path, text.replacen(from, to, 1)).unwrap();
}

/// A config file whose line 4 lacks its `=`.
fn break_config(fx: &TuiFixture) {
    fs::write(
        fx.env.config_file(),
        "default_board = \"work\"\neditor = \"\"\n[boards]\nwork \"~/w\"\n",
    )
    .unwrap();
}

/// The Help rows after ` warnings:`.
fn help_warnings(app: &mut App) -> Vec<String> {
    press(app, &["?"]);
    let rows = rows(&render(app, 80, 40));
    let at = rows.iter().position(|r| r == " warnings:").unwrap();
    rows[at + 1..]
        .iter()
        .take_while(|r| r.starts_with(' ') && !r.trim().is_empty())
        .map(|r| r.trim().to_string())
        .collect()
}

#[test]
fn fr19_opens_default_board() {
    let mut fx = work_4_9();
    let l = loaded(&fx);
    assert_eq!(prep(&mut fx, l, None).app.board.name, "work");
}

#[test]
fn fr19_opens_board_named_by_flag() {
    let mut fx = work_4_9();
    let dir = fx.home.join(".local/share/brain-swap/home");
    write_card(&dir, "H-1", "Todo", "2026-10-08T09:00:00+03:00", &[]);
    let l = loaded(&fx);
    let app = prep(&mut fx, l, Some("home")).app;
    assert_eq!(app.board.name, "home");
    assert_eq!(app.sel.card, CardId::parse("H-1"));
}

#[test]
fn fr45_broken_config_starts_in_config_error() {
    let mut fx = TuiFixture::new(None);
    let e = Error::Config {
        path: fx.env.config_file(),
        line: 4,
        col: 7,
        msg: "expected '='".into(),
    };
    let start = TuiStart {
        config: Err(e.clone()),
        board: None,
    };
    let p = prepare(&mut fx.env, start).unwrap();
    assert_eq!(p.app.mode, Mode::ConfigError(e.to_string()));
    assert!(p.config.is_none());
    let data = fx.home.join(".local/share/brain-swap");
    assert_eq!(fs::read_dir(data).map_or(0, |d| d.count()), 0);
}

#[test]
fn as9_first_run_notice_in_status_line() {
    let mut fx = TuiFixture::without_config(Some("work_4_9"));
    let l = loaded(&fx);
    let path = l.created.clone().unwrap();
    let app = prep(&mut fx, l, None).app;
    assert_eq!(
        app.status,
        Some(format!("created config {}", path.display()))
    );
}

#[test]
fn fr44_bad_binding_warns_and_keeps_default() {
    let mut fx = work_4_9();
    edit_config(&fx, "help = \"?\"", "help = \"foo\"");
    let l = loaded(&fx);
    let mut app = prep(&mut fx, l, None).app;
    assert_eq!(
        app.status.as_deref(),
        Some("key config: help: unknown key 'foo'")
    );
    press(&mut app, &["?"]);
    assert!(matches!(app.mode, Mode::Help { .. }), "{:?}", app.mode);
}

#[test]
fn fr44_conflicting_binding_reverts_and_warns() {
    let mut fx = work_4_9();
    edit_config(&fx, "help = \"?\"", "help = \"j\"");
    let l = loaded(&fx);
    let mut app = prep(&mut fx, l, None).app;
    let status = app.status.clone().unwrap_or_default();
    assert!(status.contains("help"), "{status}");
    press(&mut app, &["?"]);
    assert!(matches!(app.mode, Mode::Help { .. }), "{:?}", app.mode);
}

#[test]
fn fr44_help_lists_all_start_warnings() {
    let mut fx = work_4_9();
    edit_config(&fx, "help = \"?\"", "help = \"foo\"");
    let mut l = loaded(&fx);
    l.warnings = vec!["config: unknown key 'colour'".into()];
    let mut app = prep(&mut fx, l, None).app;
    assert_eq!(
        help_warnings(&mut app),
        [
            "config: unknown key 'colour'",
            "key config: help: unknown key 'foo'"
        ]
    );
}

#[test]
fn ts11_log_inside_board_folder_is_dropped_with_warning() {
    let mut fx = work_4_9();
    fx.env.log = Some(fx.board_dir.join("bs.log"));
    let l = loaded(&fx);
    let app = prep(&mut fx, l, None).app;
    assert!(
        app.warnings
            .contains(&"BRAIN_SWAP_LOG inside board work, ignored".to_string()),
        "{:?}",
        app.warnings
    );
    assert_eq!(fx.env.log, None);
}

#[test]
fn ts11_first_load_warning_in_status_line() {
    let mut fx = TuiFixture::new(Some("as7"));
    let l = loaded(&fx);
    let app = prep(&mut fx, l, None).app;
    assert_eq!(app.status.as_deref(), Some("W-13: unknown column 'Doign'"));
}

#[test]
fn ts11_notice_wins_over_warnings() {
    let mut fx = work_4_9();
    edit_config(&fx, "help = \"?\"", "help = \"foo\"");
    let mut l = loaded(&fx);
    let path = fx.env.config_file();
    l.created = Some(path.clone());
    let mut app = prep(&mut fx, l, None).app;
    assert_eq!(
        app.status,
        Some(format!("created config {}", path.display()))
    );
    assert_eq!(
        help_warnings(&mut app),
        ["key config: help: unknown key 'foo'"]
    );
}

#[test]
fn fr19_unlistable_board_folder_fails_before_screen() {
    let mut fx = work_4_9();
    let dir = fx.board_dir.clone();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&dir).is_ok() {
        // Root ignores permissions; nothing to check.
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let start = TuiStart {
        config: Ok(loaded(&fx)),
        board: None,
    };
    let r = prepare(&mut fx.env, start);
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    match r {
        Err(e @ Error::Io(_)) => assert_eq!(e.exit_code(), 1),
        Err(e) => panic!("not an io error: {e:?}"),
        Ok(_) => panic!("prepared"),
    }
}

#[test]
fn ts6_3_gate_passes_with_terminal() {
    let fx = work_4_9();
    let start = gate(&fx.env, Some("HOME"), true).unwrap();
    assert!(start.config.is_ok());
    assert_eq!(start.board.as_deref(), Some("home"));
    break_config(&fx);
    let start = gate(&fx.env, None, true).unwrap();
    assert!(
        matches!(start.config, Err(Error::Config { .. })),
        "{start:?}"
    );
}

#[test]
fn ts11_loader_warnings_reach_help() {
    let mut fx = work_4_9();
    edit_config(&fx, "default_board", "colour = \"red\"\ndefault_board");
    let start = gate(&fx.env, None, true).unwrap();
    let app = prepare(&mut fx.env, start).unwrap().app;
    assert_eq!(
        app.warnings.first().map(String::as_str),
        Some("config: unknown key 'colour'")
    );
}

#[test]
fn ts6_2_tui_unknown_board_exits_3_before_screen() {
    let fx = work_4_9();
    let e = gate(&fx.env, Some("work2"), false).unwrap_err();
    assert!(matches!(e, Error::NotFound(_)), "{e:?}");
    assert_eq!(e.exit_code(), 3);
    assert_eq!(
        e.to_string(),
        "unknown board 'work2' (work, home, personal)"
    );
    assert_eq!(run(fx.env.clone(), Some("work2".into()), false), 3);
}

#[test]
fn ts6_3_bare_without_terminal_exits_2() {
    let fx = work_4_9();
    let e = gate(&fx.env, None, false).unwrap_err();
    assert!(matches!(e, Error::Usage(_)), "{e:?}");
    assert_eq!(e.exit_code(), 2);
    assert_eq!(e.to_string(), "not a terminal");
    assert_eq!(run(fx.env.clone(), None, false), 2);
}

#[test]
fn ts6_3_broken_config_without_terminal_exits_2() {
    let fx = work_4_9();
    break_config(&fx);
    let e = gate(&fx.env, Some("work2"), false).unwrap_err();
    assert_eq!(e.exit_code(), 2);
}
