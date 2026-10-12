//! TECHSPEC 6.1 card IDs, 6.2 exit 3 for a card, 6.3 `show`.

mod common;

use common::cli::Sandbox;
use std::process::Output;

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    s
}

fn run(s: &Sandbox, args: &[&str], code: i32) -> Output {
    s.cmd(args).assert().code(code).get_output().clone()
}

fn show(s: &Sandbox, id: &str) -> String {
    s.redact(&text(&run(s, &["show", id], 0).stdout))
}

#[test]
fn ts6_3_show_prints_block_from_card_line() {
    let s = sandbox();
    let out = show(&s, "W-12");
    assert_eq!(
        out,
        "card: W-12 migrate invoices to v13\n\
         board: work, column: Doing\n\
         latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by session 55583127-a22a-49bc-a803-e777c377a595\n\
         Doing: migration test passes on staging\n\
         Next: open the MR, ask the DBA for the v13 deploy list\n\
         Watch out: nothing new\n\
         Where: <sandbox>/ati.billing (pane w4V:p9)\n"
    );
    insta::assert_snapshot!(out);
}

#[test]
fn ts6_3_show_card_without_notes() {
    let s = sandbox();
    let out = show(&s, "W-11");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "{out}");
    assert_eq!(lines[2], "latest note: none");
}

#[test]
fn ts6_3_show_note_without_pane_or_session() {
    let s = sandbox();
    let out = show(&s, "W-9");
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines.contains(&"latest note: 2026-10-06T17:05:12+03:00, 1 d ago, no session"),
        "{out}"
    );
    assert!(lines.contains(&"Where: <sandbox>/brain-swap"), "{out}");
}

#[test]
fn ts6_3_show_auto_note() {
    let s = sandbox();
    let out = show(&s, "W-7");
    assert!(
        out.lines().any(|l| l
            == "latest note: 2026-10-08T09:40:00+03:00, 2 h ago, auto, by session 7c0ffee0-0000-4000-8000-000000000007"),
        "{out}"
    );
}

#[test]
fn ts6_1_show_resolves_board_by_letter() {
    let s = sandbox();
    std::fs::create_dir_all(s.root().join("boards/home")).unwrap();
    let out = s.redact(&text(
        &run(&s, &["--board", "home", "show", "w-12"], 0).stdout,
    ));
    assert_eq!(out, show(&s, "W-12"));
}

#[test]
fn ts6_2_show_unknown_card_exits_3() {
    let s = sandbox();
    for id in ["W-99", "Z-1"] {
        let out = run(&s, &["show", id], 3);
        let err = text(&out.stderr);
        assert_eq!(err, format!("brain-swap: {id} not found\n"));
        insta::assert_snapshot!(format!("{id}_text"), err);
        let out = run(&s, &["--json", "show", id], 3);
        let stdout = text(&out.stdout);
        assert!(stdout.contains("\"code\":\"not_found\""), "{stdout}");
        insta::assert_snapshot!(format!("{id}_json"), stdout);
    }
}
