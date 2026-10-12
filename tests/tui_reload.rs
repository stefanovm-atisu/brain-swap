//! TECHSPEC 7.4: polling the board folder and reloading it (FR-23, NFR-02, 11).

mod common;

use brain_swap::core::model::CardId;
use brain_swap::tui::Prepared;
use brain_swap::tui::app::{Input, Selection, update};
use brain_swap::tui::runtime::Runtime;
use common::tui::{Ev, TuiFixture, fake_runtime, write_card};
use ratatui::backend::TestBackend;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

fn work_4_9() -> TuiFixture {
    TuiFixture::new(Some("work_4_9"))
}

fn sel(column: usize, card: Option<u32>) -> Selection {
    Selection {
        column,
        card: card.map(|number| CardId {
            letter: 'W',
            number,
        }),
    }
}

/// A runtime over the fixture's app with `s` selected.
fn runtime_at(fx: &TuiFixture, s: Selection) -> Runtime<TestBackend> {
    let mut app = fx.app();
    app.sel = s;
    let config = brain_swap::core::config::load(&fx.env).unwrap().config;
    let prepared = Prepared {
        app,
        config: Some(config),
    };
    fake_runtime(&fx.env, prepared, 80, 24).0
}

/// Points `env.log` at a file in the temp home.
fn logged(fx: &mut TuiFixture) {
    fx.env.log = Some(fx.home.join("bs.log"));
}

/// The texts of the log lines of kind `reload`.
fn reloads(fx: &TuiFixture) -> Vec<String> {
    fs::read_to_string(fx.env.log.as_ref().unwrap())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut parts = l.splitn(3, ' ');
            parts.next();
            (parts.next() == Some("reload")).then(|| parts.next().unwrap_or("").to_string())
        })
        .collect()
}

fn edit(fx: &TuiFixture, file: &str, from: &str, to: &str) {
    let p = fx.board_dir.join(file);
    let text = fs::read_to_string(&p).unwrap();
    assert!(text.contains(from));
    fs::write(&p, text.replace(from, to)).unwrap();
}

#[test]
fn ts7_1_tick_updates_now() {
    let mut app = work_4_9().app();
    let later = app.now.checked_add(Duration::from_secs(300)).unwrap();
    assert_eq!(update(&mut app, Input::Tick(later.clone())), []);
    assert_eq!(app.now, later);
}

#[test]
fn fr23_file_change_shows_after_one_tick() {
    let fx = work_4_9();
    let (mut rt, _, _) = fx.runtime(80, 24);
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(fx.board_dir.join("W-11.md"))
        .unwrap();
    f.write_all(
        b"\n### 2026-10-08T11:50:00+03:00\n- Doing: looked at it\n- Next: fix the rate\n- Watch out: nothing\n",
    )
    .unwrap();
    rt.tick();
    let card = rt.app().board.card(sel(0, Some(11)).card.unwrap()).unwrap();
    assert_eq!(card.notes.len(), 1);
}

#[test]
fn fr23_reload_keeps_selection_by_card_id() {
    let fx = work_4_9();
    let mut rt = runtime_at(&fx, sel(1, Some(12)));
    edit(
        &fx,
        "W-12.md",
        "# migrate invoices to v13",
        "# migrate invoices",
    );
    write_card(
        &fx.board_dir,
        "W-13",
        "Doing",
        "2026-10-08T11:45:00+03:00",
        &[("2026-10-08T11:50:00+03:00", "go on")],
    );
    rt.tick();
    assert_eq!(rt.app().sel, sel(1, Some(12)));
    assert_eq!(rt.app().selected_card().unwrap().title, "migrate invoices");
}

#[test]
fn fr23_card_moved_by_hand_stays_selected() {
    let fx = work_4_9();
    let mut rt = runtime_at(&fx, sel(1, Some(12)));
    edit(&fx, "W-12.md", "column: Doing", "column: Todo");
    rt.tick();
    assert_eq!(rt.app().sel, sel(0, Some(12)));
}

#[test]
fn ts7_4_vanished_card_selects_nearest_in_column() {
    let fx = TuiFixture::new(None);
    for (id, at) in [("W-3", "10:20"), ("W-2", "10:10"), ("W-1", "10:00")] {
        let at = format!("2026-10-08T{at}:00+03:00");
        write_card(
            &fx.board_dir,
            id,
            "Todo",
            "2026-10-08T09:00:00+03:00",
            &[(&at, "go on")],
        );
    }
    let mut rt = runtime_at(&fx, sel(0, Some(1)));
    fs::remove_file(fx.board_dir.join("W-1.md")).unwrap();
    rt.tick();
    assert_eq!(rt.app().sel, sel(0, Some(2)));
    fs::remove_file(fx.board_dir.join("W-2.md")).unwrap();
    fs::remove_file(fx.board_dir.join("W-3.md")).unwrap();
    rt.tick();
    assert_eq!(rt.app().sel, sel(0, None));
}

#[test]
fn ts7_4_unchanged_folder_does_not_reload() {
    let mut fx = work_4_9();
    logged(&mut fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    rt.tick();
    rt.tick();
    assert_eq!(rt.scan_count(), 2);
    assert_eq!(reloads(&fx), Vec::<String>::new());
}

#[test]
fn ts7_4_other_files_do_not_reload() {
    let mut fx = work_4_9();
    logged(&mut fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    fs::write(fx.board_dir.join("README.md"), "changed by hand\n").unwrap();
    fs::write(fx.board_dir.join(".W-12.md.bs-tmp-1"), "half written").unwrap();
    rt.tick();
    assert_eq!(reloads(&fx), Vec::<String>::new());
}

#[test]
fn ts7_4_board_md_change_reloads() {
    let fx = work_4_9();
    let (mut rt, _, _) = fx.runtime(80, 24);
    fs::write(
        fx.board_dir.join("board.md"),
        "---\nletter: W\ncolumns: Todo, Doing, Review, Done\nnext: 13\n---\n# work\n",
    )
    .unwrap();
    rt.tick();
    assert_eq!(rt.app().board.column_list()[2].name, "Review");
}

#[test]
fn ts7_4_same_length_change_with_new_mtime_reloads() {
    let mut fx = work_4_9();
    logged(&mut fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    let p = fx.board_dir.join("W-12.md");
    let mtime = fs::metadata(&p).unwrap().modified().unwrap();
    edit(&fx, "W-12.md", "v13", "v14");
    fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(mtime + Duration::from_secs(1))
        .unwrap();
    rt.tick();
    assert_eq!(reloads(&fx), ["work"]);
}

#[test]
fn ts11_reload_is_logged_once_per_change() {
    let mut fx = work_4_9();
    logged(&mut fx);
    let (mut rt, _, _) = fx.runtime(80, 24);
    edit(&fx, "W-12.md", "column: Doing", "column: Todo");
    rt.tick();
    rt.tick();
    assert_eq!(reloads(&fx), ["work"]);
}

#[test]
fn nfr02_idle_runtime_scans_once_per_second() {
    let mut fx = work_4_9();
    logged(&mut fx);
    let (mut rt, _, events) = fx.runtime(80, 24);
    events.push((0..8).map(|_| Ev::Timeout));
    events.push([Ev::Key("q")]);
    assert_eq!(rt.run(), 0);
    assert_eq!(rt.scan_count(), 2);
    assert_eq!(reloads(&fx), Vec::<String>::new());
}

#[test]
fn ts7_4_missing_folder_created_later_reloads() {
    let fx = TuiFixture::new(None);
    let (mut rt, _, _) = fx.runtime(80, 24);
    write_card(
        &fx.board_dir,
        "W-1",
        "Todo",
        "2026-10-08T09:00:00+03:00",
        &[],
    );
    rt.tick();
    assert!(rt.app().board.card(sel(0, Some(1)).card.unwrap()).is_some());
}

#[test]
fn ts7_4_reload_error_keeps_board() {
    let fx = work_4_9();
    let (mut rt, _, _) = fx.runtime(80, 24);
    let before = rt.app().board.clone();
    let dir = &fx.board_dir;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(dir).is_ok() {
        // Root ignores permissions; nothing to check.
        fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    rt.tick();
    fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(rt.app().board, before);
    let status = rt.app().status.clone().unwrap_or_default();
    assert!(status.starts_with("board work: "), "{status}");
}
