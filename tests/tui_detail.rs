//! TECHSPEC 5.2, 7.1 and 7.2: opening the detail view (FR-26, FR-27).

mod common;

use brain_swap::core::keys::KeyMap;
use brain_swap::core::model::CardId;
use brain_swap::tui::app::{App, Input, Mode, Selection, update};
use common::tui::{TuiFixture, assert_no_colour, press, render, rows};

const W12: CardId = CardId {
    letter: 'W',
    number: 12,
};

fn w12_detail() -> Mode {
    Mode::Detail {
        card: W12,
        note: 1,
        scroll: 0,
    }
}

/// The `work_4_9` app with W-12 (Doing) selected.
fn board_on_w12() -> App {
    let mut app = TuiFixture::new(Some("work_4_9")).app();
    app.sel = Selection {
        column: 1,
        card: Some(W12),
    };
    app
}

fn detail_on_w12() -> App {
    let mut app = board_on_w12();
    press(&mut app, &["o"]);
    app
}

fn detail_rows() -> Vec<String> {
    rows(&render(&detail_on_w12(), 80, 24))
}

#[test]
fn fr26_open_shows_newest_note() {
    let mut app = board_on_w12();
    assert!(press(&mut app, &["o"]).is_empty());
    assert_eq!(app.mode, w12_detail());
}

#[test]
fn fr26_open_without_selected_card_does_nothing() {
    let mut app = TuiFixture::new(None).app();
    assert!(press(&mut app, &["o"]).is_empty());
    assert_eq!(app.mode, Mode::Board);
}

#[test]
fn fr27_detail_header_shows_id_title_column_and_board() {
    let buf = render(&detail_on_w12(), 80, 24);
    assert_eq!(rows(&buf)[0], " W-12 migrate invoices to v13 (Doing, work)");
    assert_no_colour(&buf);
}

#[test]
fn fr26_note_line_shows_indicator_stamp_and_age() {
    assert_eq!(
        detail_rows()[1],
        " note 2/2, 2026-10-08T11:12:40+03:00, 39 min ago"
    );
}

#[test]
fn fr27_detail_shows_three_parts() {
    assert_eq!(
        detail_rows()[2..5],
        [
            " Doing:      migration test passes on staging",
            " Next:       open the MR, ask the DBA for the v13 deploy list",
            " Watch out:  nothing new",
        ]
    );
}

#[test]
fn fr27_where_line_shows_cwd_and_pane() {
    assert_eq!(
        detail_rows()[5],
        " Where: /home/smilen/Work/ati.billing (pane w4V:p9)"
    );
}

#[test]
fn ts5_2_quit_in_detail_returns_to_board() {
    for k in ["q", "esc"] {
        let mut app = detail_on_w12();
        assert_eq!(app.mode, w12_detail(), "{k}");
        assert!(press(&mut app, &[k]).is_empty(), "{k}");
        assert_eq!(app.mode, Mode::Board, "{k}");
        assert_eq!(app.sel.card, Some(W12), "{k}");
    }
}

#[test]
fn ts5_2_board_keys_do_nothing_in_detail() {
    let mut app = detail_on_w12();
    assert!(press(&mut app, &["n", "H", "L", "b", "l"]).is_empty());
    assert_eq!(app.mode, w12_detail());
}

fn note_row(app: &App) -> String {
    rows(&render(app, 80, 24))[1].clone()
}

/// The `detail` fixture with W-5 (Doing) selected, still on the board.
fn board_on_w5() -> App {
    let mut app = TuiFixture::new(Some("detail")).app();
    app.sel = Selection {
        column: 1,
        card: Some(CardId {
            letter: 'W',
            number: 5,
        }),
    };
    app
}

#[test]
fn as1_j_shows_older_note_and_k_newest_again() {
    let mut app = detail_on_w12();
    assert!(press(&mut app, &["j"]).is_empty());
    let rows = rows(&render(&app, 80, 24));
    assert_eq!(
        rows[1],
        " note 1/2, 2026-10-08T10:31:05+03:00, 1 h ago, auto"
    );
    assert_eq!(
        rows[2],
        " Doing:      switched InvoiceRepository to v13, unit tests green"
    );
    press(&mut app, &["k"]);
    assert!(note_row(&app).starts_with(" note 2/2,"));
}

#[test]
fn fr26_note_navigation_clamps() {
    let mut app = detail_on_w12();
    press(&mut app, &["k"]);
    assert!(note_row(&app).starts_with(" note 2/2,"));
    press(&mut app, &["j", "j"]);
    assert!(note_row(&app).starts_with(" note 1/2,"));
}

#[test]
fn fr26_indicator_counts_from_oldest() {
    let mut app = board_on_w5();
    for (k, want) in [
        ("o", " note 3/3,"),
        ("j", " note 2/3,"),
        ("j", " note 1/3,"),
    ] {
        press(&mut app, &[k]);
        assert!(note_row(&app).starts_with(want), "{k} {want}");
    }
}

#[test]
fn ts5_2_rebound_detail_keys() {
    let mut app = detail_on_w12();
    app.keys = KeyMap::build(&[("down".into(), vec!["x".into()])]).0;
    press(&mut app, &["x"]);
    assert!(note_row(&app).starts_with(" note 1/2,"));
    press(&mut app, &["k"]);
    assert!(press(&mut app, &["j"]).is_empty());
    assert!(note_row(&app).starts_with(" note 2/2,"));
}

#[test]
fn fr26_detail_note_snapshots() {
    let mut app = board_on_w5();
    press(&mut app, &["o"]);
    let mut frames = vec![("3_120x40".to_string(), render(&app, 120, 40))];
    for n in [3, 2, 1] {
        frames.push((format!("{n}_80x24"), render(&app, 80, 24)));
        press(&mut app, &["j"]);
    }
    for (name, buf) in frames {
        assert_no_colour(&buf);
        insta::assert_snapshot!(format!("detail_note_{name}"), rows(&buf).join("\n"));
    }
}

fn detail_on_w5() -> App {
    let mut app = board_on_w5();
    press(&mut app, &["o"]);
    app
}

fn scroll(app: &App) -> u16 {
    match app.mode {
        Mode::Detail { scroll, .. } => scroll,
        _ => panic!("not in Detail: {:?}", app.mode),
    }
}

/// The row right after the separator, the first body row.
fn first_body_row(app: &App) -> String {
    let rows = rows(&render(app, 80, 24));
    let sep = rows.iter().position(|r| r.starts_with("---")).unwrap();
    rows[sep + 1].clone()
}

#[test]
fn fr27_body_follows_separator() {
    let rows = rows(&render(&detail_on_w5(), 80, 24));
    assert_eq!(rows[6], "-".repeat(80));
    assert_eq!(rows[7..9], [" ## Question", " line 1"]);
}

#[test]
fn fr27_scroll_down_moves_body_by_step() {
    let mut app = detail_on_w5();
    assert!(press(&mut app, &["ctrl+d"]).is_empty());
    assert_eq!(scroll(&app), 10);
    assert_eq!(first_body_row(&app), " line 10");
}

#[test]
fn fr27_scroll_up_clamps_at_top() {
    let mut app = detail_on_w5();
    press(&mut app, &["ctrl+u"]);
    assert_eq!(scroll(&app), 0);
    press(&mut app, &["ctrl+d"]);
    assert_eq!(scroll(&app), 10);
    press(&mut app, &["ctrl+u"]);
    assert_eq!(scroll(&app), 0);
}

#[test]
fn fr27_scroll_down_clamps_at_last_line() {
    let mut app = detail_on_w5();
    press(&mut app, &["ctrl+d"; 6]);
    let body = app.selected_card().unwrap().body.trim_end().to_string();
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(usize::from(scroll(&app)), lines.len() - 1);
    assert_eq!(first_body_row(&app), format!(" {}", lines[lines.len() - 1]));
    assert_eq!(first_body_row(&app), " line 40");
}

#[test]
fn fr27_scroll_kept_when_changing_note() {
    let mut app = detail_on_w5();
    press(&mut app, &["ctrl+d", "j"]);
    assert_eq!(scroll(&app), 10);
    assert!(note_row(&app).starts_with(" note 2/3,"));
}

#[test]
fn ts7_2_hint_line_shows_current_keys() {
    let mut app = detail_on_w5();
    assert_eq!(
        rows(&render(&app, 80, 24))[22],
        " j older  k newer  ctrl+d down  ctrl+u up  enter jump  e edit  ? help  q back"
    );
    app.keys = KeyMap::build(&[("down".into(), vec!["x".into()])]).0;
    assert!(rows(&render(&app, 80, 24))[22].starts_with(" x older"));
}

#[test]
fn fr27_wide_body_line_cut_not_wrapped() {
    let fx = TuiFixture::new(Some("detail"));
    let wide = "界".repeat(60);
    let text = format!(
        "---\ncolumn: Doing\n---\n# wide\n{wide}\nsecond\n\n## Timeline\n\n### 2026-10-08T11:40:00+03:00\n- Doing: d\n- Next: n\n- Watch out: w\n"
    );
    std::fs::write(fx.board_dir.join("W-7.md"), text).unwrap();
    let mut app = fx.app();
    app.mode = Mode::Detail {
        card: CardId {
            letter: 'W',
            number: 7,
        },
        note: 0,
        scroll: 0,
    };
    let rows = rows(&render(&app, 80, 24));
    let at = rows.iter().position(|r| r.starts_with(" 界")).unwrap();
    assert_eq!(rows[at + 1], " second");
}

fn w(number: u32) -> CardId {
    CardId {
        letter: 'W',
        number,
    }
}

/// The `detail` fixture app in Detail on `W-<number>` at note index `note`.
fn detail_at(fx: &TuiFixture, number: u32, note: usize) -> App {
    let mut app = fx.app();
    app.mode = Mode::Detail {
        card: w(number),
        note,
        scroll: 0,
    };
    app
}

fn where_row(app: &App) -> String {
    let rows = rows(&render(app, 80, 24));
    rows.into_iter().find(|r| r.starts_with(" Where:")).unwrap()
}

#[test]
fn fr27_card_without_notes_shows_no_note_yet() {
    let mut app = TuiFixture::new(Some("detail")).app();
    app.sel = Selection {
        column: 0,
        card: Some(w(4)),
    };
    press(&mut app, &["o"]);
    let buf = render(&app, 80, 24);
    let rows = rows(&buf);
    assert!(rows.iter().any(|r| r == " no note yet"), "{rows:?}");
    assert!(!rows.iter().any(|r| r.starts_with(" note ")), "{rows:?}");
    assert_no_colour(&buf);
    insta::assert_snapshot!("detail_no_note_80x24", rows.join("\n"));
    let mode = app.mode.clone();
    assert!(press(&mut app, &["j", "k"]).is_empty());
    assert_eq!(app.mode, mode);
}

#[test]
fn fr27_note_without_place_shows_where_none() {
    let fx = TuiFixture::new(Some("detail"));
    assert_eq!(where_row(&detail_at(&fx, 5, 2)), " Where: none");
}

#[test]
fn fr27_note_with_cwd_only_shows_cwd() {
    let fx = TuiFixture::new(Some("detail"));
    let app = detail_at(&fx, 5, 1);
    assert_eq!(where_row(&app), " Where: /w/latency");
    let rows = rows(&render(&app, 80, 24));
    assert_eq!(
        rows[2..4],
        [
            " Doing:      two runs done",
            "             third run pending"
        ]
    );
}

#[test]
fn fr27_bad_stamp_shows_heading_text_and_question_mark() {
    let fx = TuiFixture::new(Some("detail"));
    let app = detail_at(&fx, 6, 0);
    assert_eq!(rows(&render(&app, 80, 24))[1], " note 1/1, someday, ? ago");
    assert_eq!(where_row(&app), " Where: /w/odd (pane w4V:p5)");
}

#[test]
fn fr32_help_from_detail_returns_to_detail() {
    let mut app = detail_on_w5();
    press(&mut app, &["ctrl+d", "j"]);
    let want = Mode::Detail {
        card: w(5),
        note: 1,
        scroll: 10,
    };
    assert_eq!(app.mode, want);
    assert!(press(&mut app, &["?"]).is_empty());
    assert_eq!(
        app.mode,
        Mode::Help {
            back: Box::new(want.clone())
        }
    );
    assert!(press(&mut app, &["x"]).is_empty());
    assert_eq!(app.mode, want);
}

#[test]
fn fr23_reload_in_detail_keeps_shown_note() {
    let fx = TuiFixture::new(Some("detail"));
    let mut app = detail_at(&fx, 5, 1);
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(fx.board_dir.join("W-5.md"))
        .unwrap();
    std::io::Write::write_all(
        &mut f,
        b"\n### 2026-10-08T11:50:00+03:00\n- Doing: d\n- Next: n\n- Watch out: w\n",
    )
    .unwrap();
    assert!(update(&mut app, Input::Reloaded(fx.board())).is_empty());
    assert!(matches!(app.mode, Mode::Detail { note: 1, .. }));
    assert!(note_row(&app).starts_with(" note 2/4,"));
}

#[test]
fn fr23_reload_with_card_gone_returns_to_board() {
    let fx = TuiFixture::new(Some("detail"));
    let detail = detail_at(&fx, 5, 1);
    let mut help = detail.clone();
    help.mode = Mode::Help {
        back: Box::new(detail.mode.clone()),
    };
    std::fs::remove_file(fx.board_dir.join("W-5.md")).unwrap();
    for mut app in [detail, help] {
        update(&mut app, Input::Reloaded(fx.board()));
        assert_eq!(app.mode, Mode::Board);
    }
}
