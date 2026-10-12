//! TECHSPEC 5.2, 5.4, 7.1, 7.2 and 7.4: the template and board pickers (FR-24, FR-30, NFR-10).

mod common;

use brain_swap::core::config::load;
use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Effect, Input, Mode};
use brain_swap::tui::runtime::Runtime;
use brain_swap::tui::{TuiStart, prepare};
use common::tui::{TuiFixture, assert_no_colour, key, press, render, rows, write_card};
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use std::fs;
use std::os::unix::fs::PermissionsExt;

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

/// The `work_4_9` app through `tui::prepare` with the default config, on `board`.
fn prepared_on(board: Option<&str>) -> App {
    let mut fx = TuiFixture::new(Some("work_4_9"));
    let start = TuiStart {
        config: load(&fx.env),
        board: board.map(String::from),
    };
    prepare(&mut fx.env, start).unwrap().app
}

fn board_picker() -> App {
    let mut app = prepared_on(None);
    press(&mut app, &["b"]);
    app
}

fn at(time: &str) -> String {
    format!("2026-10-08T{time}+03:00")
}

/// The fixture with H-1 (Todo, created 09:00) and H-2 (Doing, a note at 11:00) in the home board.
fn with_home(fx: &TuiFixture) {
    let d = fx.env.default_board_dir("home");
    write_card(&d, "H-1", "Todo", &at("09:00:00"), &[]);
    write_card(
        &d,
        "H-2",
        "Doing",
        &at("08:00:00"),
        &[(&at("11:00:00"), "n")],
    );
}

fn dispatch_keys(rt: &mut Runtime<TestBackend>, keys: &[&str]) {
    for k in keys {
        rt.dispatch(Input::Key(key(k)));
    }
}

/// The number of `reload` lines in `env.log`.
fn reload_lines(fx: &TuiFixture) -> usize {
    fs::read_to_string(fx.env.log.as_ref().unwrap())
        .unwrap_or_default()
        .lines()
        .filter(|l| l.split(' ').nth(1) == Some("reload"))
        .count()
}

#[test]
fn fr24_switch_board_lists_configured_boards() {
    let app = board_picker();
    assert_eq!(app.mode, Mode::BoardPick { sel: 0 });
    let frame = rows(&render(&app, 80, 24));
    let names: Vec<&str> = frame[1..4]
        .iter()
        .map(|r| r[1..].split_whitespace().next().unwrap_or(""))
        .collect();
    assert_eq!(names, ["work", "home", "personal"], "{frame:?}");
}

#[test]
fn fr24_confirm_emits_load_board() {
    let mut app = board_picker();
    assert_eq!(
        press(&mut app, &["j", "enter"]),
        [Effect::LoadBoard("home".into())]
    );
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn ts5_2_cancel_in_board_pick_returns() {
    let mut app = board_picker();
    assert!(press(&mut app, &["esc"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr24_pick_starts_on_current_board() {
    let mut app = prepared_on(Some("home"));
    press(&mut app, &["b"]);
    assert_eq!(app.mode, Mode::BoardPick { sel: 1 });
}

#[test]
fn fr24_switched_board_opens_with_initial_selection() {
    let fx = TuiFixture::new(Some("work_4_9"));
    with_home(&fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    dispatch_keys(&mut rt, &["b", "j", "enter"]);
    assert_eq!(rt.app().board.name, "home");
    assert_eq!(rt.app().mode, Mode::Board);
    assert_eq!(rt.app().sel.column, 1);
    let h2 = CardId {
        letter: 'H',
        number: 2,
    };
    assert_eq!(rt.app().sel.card, Some(h2));
}

#[test]
fn ts7_4_polling_follows_switched_board() {
    let mut fx = TuiFixture::new(Some("work_4_9"));
    fx.env.log = Some(fx.home.join("bs.log"));
    with_home(&fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    dispatch_keys(&mut rt, &["b", "j", "enter"]);
    let home = fx.env.default_board_dir("home");
    write_card(&home, "H-3", "Todo", &at("09:30:00"), &[]);
    rt.tick();
    let h3 = CardId {
        letter: 'H',
        number: 3,
    };
    assert!(rt.app().board.card(h3).is_some());
    let before = reload_lines(&fx);
    let w12 = fx.board_dir.join("W-12.md");
    let text = fs::read_to_string(&w12).unwrap();
    fs::write(&w12, text + "\nmore\n").unwrap();
    rt.tick();
    assert_eq!(reload_lines(&fx), before);
}

#[test]
fn fr24_switch_failure_keeps_current_board() {
    let fx = TuiFixture::new(Some("work_4_9"));
    with_home(&fx);
    let home = fx.env.default_board_dir("home");
    let (mut rt, _, _) = fx.runtime(80, 24);
    fs::set_permissions(&home, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&home).is_ok() {
        // Root ignores permissions; nothing to check.
        fs::set_permissions(&home, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    dispatch_keys(&mut rt, &["b", "j", "enter"]);
    fs::set_permissions(&home, fs::Permissions::from_mode(0o755)).unwrap();
    let status = rt.app().status.clone().unwrap_or_default();
    assert!(status.starts_with("board home: "), "{status}");
    assert_eq!(rt.app().board.name, "work");
}

#[test]
fn fr24_board_pick_snapshots() {
    let fx = TuiFixture::new(Some("work_4_9"));
    let mut env = fx.env.clone();
    let start = TuiStart {
        config: load(&env),
        board: None,
    };
    let mut app = prepare(&mut env, start).unwrap().app;
    press(&mut app, &["b"]);
    let root = fx.home.parent().unwrap().display().to_string();
    for (w, h) in [(80, 24), (120, 40)] {
        let buf = render(&app, w, h);
        assert_no_colour(&buf);
        assert!(buf[(0, 1)].modifier.contains(Modifier::REVERSED));
        assert!(!buf[(0, 2)].modifier.contains(Modifier::REVERSED));
        let text = rows(&buf).join("\n").replace(&root, "<root>");
        insta::assert_snapshot!(format!("board_pick_{w}x{h}"), text);
    }
}

/// The `work_4_9` app in TitleInput for Feature with `buf`.
fn titled(buf: &str) -> App {
    let mut app = work_4_9();
    app.mode = Mode::TitleInput {
        template: "Feature".into(),
        buf: buf.into(),
    };
    app
}

fn buf_of(app: &App) -> &str {
    match &app.mode {
        Mode::TitleInput { buf, .. } => buf,
        m => panic!("not in TitleInput: {m:?}"),
    }
}

fn create(title: &str, column: &str) -> Effect {
    Effect::Create {
        template: "Feature".into(),
        title: title.into(),
        column: column.into(),
    }
}

#[test]
fn fr30_j_and_k_type_into_title() {
    let mut app = work_4_9();
    press(&mut app, &["n", "f", "j", "k"]);
    assert_eq!(
        app.mode,
        Mode::TitleInput {
            template: "Feature".into(),
            buf: "jk".into()
        }
    );
}

#[test]
fn ts5_2_title_input_takes_every_printable_character() {
    let mut app = titled("");
    assert!(press(&mut app, &["q", "?", "H", "space", "é"]).is_empty());
    assert_eq!(buf_of(&app), "q?H é");
}

#[test]
fn ts5_2_backspace_deletes_last_character() {
    let mut app = titled("abc");
    press(&mut app, &["backspace"]);
    assert_eq!(buf_of(&app), "ab");
    let mut app = titled("");
    press(&mut app, &["backspace"]);
    assert_eq!(buf_of(&app), "");
}

#[test]
fn ts5_2_confirm_ignored_while_title_blank() {
    for buf in ["", "   "] {
        let mut app = titled(buf);
        assert!(press(&mut app, &["enter"]).is_empty());
        assert_eq!(buf_of(&app), buf);
    }
}

#[test]
fn as6_creation_emits_one_create_in_focused_column() {
    let mut app = work_4_9();
    app.sel.column = 0;
    let keys = [
        "n", "f", "f", "i", "x", "space", "l", "o", "g", "i", "n", "enter",
    ];
    assert_eq!(press(&mut app, &keys), [create("fix login", "Todo")]);
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr30_title_is_trimmed_and_column_is_focused_one() {
    let mut app = titled("  spike  ");
    app.sel.column = 1;
    assert_eq!(press(&mut app, &["enter"]), [create("spike", "Doing")]);
}

#[test]
fn ts5_2_cancel_in_title_input_returns_to_board() {
    let mut app = titled("abc");
    assert!(press(&mut app, &["esc"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn ts5_2_printable_confirm_binding_types_in_title() {
    let mut app = titled("a");
    app.sel.column = 0;
    app.keys = KeyMap::build(&[("confirm".into(), vec!["enter".into(), "y".into()])]).0;
    assert!(press(&mut app, &["y"]).is_empty());
    assert!(buf_of(&app).ends_with('y'));
    assert_eq!(press(&mut app, &["enter"]), [create("ay", "Todo")]);
}

#[test]
fn ts5_2_ctrl_and_alt_keys_are_not_inserted() {
    let mut app = titled("ab");
    assert!(press(&mut app, &["ctrl+d", "alt+x"]).is_empty());
    assert_eq!(buf_of(&app), "ab");
}

#[test]
fn fr30_create_from_extra_column_uses_first_column() {
    let mut app = TuiFixture::new(Some("as7")).app();
    app.sel.column = app
        .board
        .column_list()
        .iter()
        .position(|c| c.name == "Doign (unknown)")
        .unwrap();
    let effects = press(&mut app, &["n", "f", "x", "enter"]);
    assert_eq!(effects, [create("x", "Todo")]);
}

#[test]
fn fr30_title_input_snapshots() {
    let mut app = titled("fix login");
    app.sel.column = 0;
    for (w, h) in [(80, 24), (120, 40)] {
        let buf = render(&app, w, h);
        assert_no_colour(&buf);
        insta::assert_snapshot!(format!("title_input_{w}x{h}"), rows(&buf).join("\n"));
    }
}
