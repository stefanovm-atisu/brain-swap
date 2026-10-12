//! TECHSPEC 4.9, 7.2 and 7.3: the board's header, columns and cards (FR-20, NFR-10).

mod common;

use brain_swap::core::model::{Card, CardId};
use brain_swap::tui::app::{App, Input, Selection, update};
use brain_swap::tui::view_board::{card_lines, panel_lines};
use common::tui::{Ev, TuiFixture, assert_no_colour, render, rows, write_card};
use ratatui::style::Modifier;
use std::time::Duration;

fn app(fixture: &str) -> App {
    TuiFixture::new(Some(fixture)).app()
}

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn card(app: &App, s: &str) -> Card {
    app.board.card(id(s)).unwrap().clone()
}

fn w12_selected() -> App {
    let mut app = app("work_4_9");
    app.sel = Selection {
        column: 1,
        card: Some(id("W-12")),
    };
    app
}

/// W-12 with its latest note's Doing and Next replaced.
fn second_line(doing: &str, next: &str) -> String {
    let app = app("work_4_9");
    let mut c = card(&app, "W-12");
    let n = c.notes.last_mut().unwrap();
    (n.doing, n.next) = (doing.into(), next.into());
    card_lines(&c, &app.now).1
}

#[test]
fn fr20_card_lines_show_id_title_age_and_next() {
    let app = app("work_4_9");
    assert_eq!(
        card_lines(&card(&app, "W-12"), &app.now),
        (
            "W-12 migrate invoices to v13".to_string(),
            "  39 min  open the MR, ask the DBA for the v13 deploy list".to_string()
        )
    );
}

#[test]
fn fr20_card_without_notes_shows_no_note() {
    let app = app("work_4_9");
    assert_eq!(card_lines(&card(&app, "W-11"), &app.now).1, "  no note");
}

#[test]
fn fr20_preview_falls_back_to_doing() {
    assert_eq!(second_line("first\nsecond", ""), "  39 min  first");
}

#[test]
fn fr20_preview_is_first_line_of_next() {
    assert!(second_line("doing", "a\nb").ends_with("  a"));
}

#[test]
fn fr20_empty_next_and_doing_show_only_age() {
    assert_eq!(second_line("", ""), "  39 min");
}

#[test]
fn ts7_2_header_shows_board_path_and_help_key() {
    let rows = rows(&render(&app("work_4_9"), 80, 24));
    assert!(
        rows[0].starts_with(" work  ~/.local/share/brain-swap/work"),
        "{}",
        rows[0]
    );
    assert!(rows[0].ends_with("? help"), "{}", rows[0]);
}

#[test]
fn ts7_3_column_headers_show_counts_and_focus() {
    let row = &rows(&render(&w12_selected(), 80, 24))[1];
    let at: Vec<usize> = ["Todo (1)", "> Doing (1)", "Done (1)"]
        .iter()
        .map(|h| row.find(h).unwrap_or_else(|| panic!("{h} in {row}")))
        .collect();
    assert!(at.is_sorted(), "{row}");
}

#[test]
fn ts7_3_selection_is_reverse_video_and_focus_bold() {
    let buf = render(&w12_selected(), 80, 24);
    let rows = rows(&buf);
    let (x0, next) = (
        rows[2].find("W-12").unwrap() as u16,
        rows[2].find("W-9").unwrap() as u16,
    );
    for y in 0..24 {
        for x in 0..80 {
            let inside = (2..=3).contains(&y) && (x0..next - 2).contains(&x);
            let reversed = buf[(x, y)].modifier.contains(Modifier::REVERSED);
            assert_eq!(reversed, inside, "cell ({x}, {y})");
        }
    }
    let h = rows[1].find("> Doing (1)").unwrap() as u16;
    for x in h..h + "> Doing (1)".len() as u16 {
        assert!(
            buf[(x, 1)].modifier.contains(Modifier::BOLD),
            "cell ({x}, 1)"
        );
    }
}

#[test]
fn ts7_3_text_is_cut_at_column_width() {
    let rows = rows(&render(&app("work_4_9"), 80, 24));
    let next = rows[2].find("W-12").unwrap();
    let todo = rows[2][..next].trim();
    let rest = todo.strip_prefix("W-11 ").unwrap();
    assert!(!rest.is_empty(), "{todo}");
    assert!(
        "invoice PDF shows the wrong VAT".starts_with(rest),
        "{todo}"
    );
    assert_ne!(rest, "invoice PDF shows the wrong VAT", "{todo}");
    assert!(rows.iter().all(|r| r.chars().count() <= 80));
}

#[test]
fn ts4_7_extra_column_renders_last() {
    let rows = rows(&render(&app("as7"), 120, 40));
    assert!(rows[1].ends_with("Doign (unknown) (1)"), "{}", rows[1]);
}

#[test]
fn nfr10_board_uses_no_colour() {
    let buf = render(&w12_selected(), 80, 24);
    assert!(
        rows(&buf)[1].contains("> Doing (1)"),
        "the board of test 7 is drawn"
    );
    assert_no_colour(&buf);
}

#[test]
fn fr20_preview_skips_blank_first_line_of_next() {
    assert_eq!(
        second_line("doing", "\nopen the MR"),
        "  39 min  open the MR"
    );
}

#[test]
fn fr20_blank_next_falls_back_to_doing() {
    assert_eq!(second_line("first", "\n  \n"), "  39 min  first");
}

/// The row under the one holding `W-12` in `lines`.
fn w12_second_row(lines: &[String]) -> &str {
    let i = lines.iter().position(|r| r.contains("W-12")).unwrap();
    &lines[i + 1]
}

#[test]
fn fr21_card_ages_are_floored() {
    let fx = TuiFixture::new(Some("work_4_9"));
    let stamps = [
        "2026-10-08T11:51:01+03:00",
        "2026-10-08T11:51:00+03:00",
        "2026-10-08T10:52:01+03:00",
        "2026-10-07T11:53:00+03:00",
        "2026-10-06T17:05:00+03:00",
        "2026-10-08T11:57:00+03:00",
        "someday",
    ];
    for (i, at) in stamps.iter().enumerate() {
        let id = format!("W-{}", i + 1);
        write_card(&fx.board_dir, &id, "Todo", stamps[0], &[(at, "go")]);
    }
    let app = fx.app();
    let ages: Vec<String> = (1..=7)
        .map(|i| card_lines(&card(&app, &format!("W-{i}")), &app.now).1)
        .collect();
    let want = [
        "  now", "  1 min", "  59 min", "  23 h", "  1 d", "  now", "  ?",
    ];
    for (got, want) in ages.iter().zip(want) {
        assert!(got.starts_with(&format!("{want}  ")), "{got:?} vs {want:?}");
    }
}

#[test]
fn fr21_tick_refreshes_card_age() {
    let mut app = app("work_4_9");
    assert!(w12_second_row(&rows(&render(&app, 80, 24))).contains("39 min"));
    let later = app.now.checked_add(Duration::from_secs(60)).unwrap();
    update(&mut app, Input::Tick(later));
    assert!(w12_second_row(&rows(&render(&app, 80, 24))).contains("40 min"));
}

#[test]
fn fr21_idle_runtime_refreshes_ages_within_30_seconds() {
    let mut fx = TuiFixture::new(Some("work_4_9"));
    fx.env.now = fx.env.now.checked_add(Duration::from_secs(30)).unwrap();
    let (mut rt, _guard, events) = fx.runtime(80, 24);
    rt.draw().unwrap();
    let first = rows(rt.terminal.backend().buffer());
    assert!(w12_second_row(&first).contains("39 min"));
    events.push((0..120).map(|_| Ev::Timeout));
    events.push([Ev::Key("q")]);
    rt.run();
    let last = rows(rt.terminal.backend().buffer());
    assert!(w12_second_row(&last).contains("40 min"));
}

#[test]
fn fr22_panel_shows_latest_note_with_pane_and_cwd() {
    let app = app("work_4_9");
    assert_eq!(
        panel_lines(Some(&card(&app, "W-12")), &app.now),
        [
            "W-12 latest: 39 min ago, pane w4V:p9, /home/smilen/Work/ati.billing",
            "Doing:      migration test passes on staging",
            "Next:       open the MR, ask the DBA for the v13 deploy list",
            "Watch out:  nothing new",
        ]
    );
}

#[test]
fn fr22_panel_marks_auto() {
    let app = app("work_4_9");
    let mut c = card(&app, "W-12");
    let n = c.notes.last_mut().unwrap();
    n.auto = true;
    n.at = app.now.checked_sub(Duration::from_secs(12 * 60)).ok();
    n.place.herdr = None;
    assert_eq!(
        panel_lines(Some(&c), &app.now)[0],
        "W-12 latest: 12 min ago, auto, /home/smilen/Work/ati.billing"
    );
}

#[test]
fn fr22_panel_without_pane_shows_cwd_only() {
    let app = app("work_4_9");
    assert_eq!(
        panel_lines(Some(&card(&app, "W-9")), &app.now)[0],
        "W-9 latest: 1 d ago, /home/smilen/Work/private/brain-swap"
    );
}

#[test]
fn fr22_panel_without_notes_says_no_note_yet() {
    let app = app("work_4_9");
    assert_eq!(
        panel_lines(Some(&card(&app, "W-11")), &app.now),
        ["W-11 no note yet"]
    );
}

#[test]
fn fr22_panel_shows_first_line_of_each_part() {
    let app = app("work_4_9");
    let mut c = card(&app, "W-12");
    c.notes.last_mut().unwrap().doing = "first\nsecond".into();
    assert_eq!(panel_lines(Some(&c), &app.now)[1], "Doing:      first");
}

/// A row of only `-` after the margin.
fn is_separator(row: &str) -> bool {
    let r = row.trim();
    !r.is_empty() && r.chars().all(|c| c == '-')
}

#[test]
fn fr22_panel_with_no_selection_shows_only_separator() {
    let fx = TuiFixture::new(Some("work_4_9"));
    std::fs::remove_file(fx.board_dir.join("W-11.md")).unwrap();
    let mut app = fx.app();
    app.sel = Selection {
        column: 0,
        card: None,
    };
    let rows = rows(&render(&app, 80, 24));
    assert!(rows.iter().any(|r| is_separator(r)), "{rows:#?}");
    assert!(!rows.iter().any(|r| r.contains("latest:")), "{rows:#?}");
}

#[test]
fn fr22_panel_hidden_below_20_rows() {
    let app = w12_selected();
    let low = rows(&render(&app, 80, 19));
    assert!(!low.iter().any(|r| is_separator(r)), "{low:#?}");
    assert!(!low.iter().any(|r| r.contains("latest:")), "{low:#?}");
    let high = rows(&render(&app, 80, 20));
    assert!(high.iter().any(|r| is_separator(r)), "{high:#?}");
    assert!(high.iter().any(|r| r.contains("latest:")), "{high:#?}");
}

#[test]
fn ts4_9_board_snapshot_80x24() {
    let buf = render(&w12_selected(), 80, 24);
    let rows = rows(&buf);
    let text = rows.join("\n");
    for want in [
        " work  ~/.local/share/brain-swap/work",
        "> Doing (1)",
        "no note",
        // Equal column widths (A-E4-09) cut two characters before the drawing's `ask`.
        "39 min  open the MR, a",
        "1 d  nothing left",
        " W-12 latest: 39 min ago, pane w4V:p9, /home/smilen/Work/ati.billing",
        " Doing:      migration test passes on staging",
        " Next:       open the MR, ask the DBA for the v13 deploy list",
        " Watch out:  nothing new",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    assert!(!text.contains("39 min  open the MR, as"), "{text}");
    assert!(rows.iter().any(|r| is_separator(r)), "{text}");
    assert_no_colour(&buf);
    insta::assert_snapshot!(text);
}

#[test]
fn ts4_9_board_snapshot_120x40() {
    let buf = render(&w12_selected(), 120, 40);
    assert_no_colour(&buf);
    insta::assert_snapshot!(rows(&buf).join("\n"));
}

#[test]
fn nfr10_panel_uses_no_colour() {
    for (w, h) in [(80, 24), (120, 40)] {
        let buf = render(&w12_selected(), w, h);
        assert!(
            rows(&buf).iter().any(|r| r.contains("W-12 latest:")),
            "the panel of tests 8 and 9 is drawn"
        );
        assert_no_colour(&buf);
    }
}
