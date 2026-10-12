//! TECHSPEC 4.7 extra columns, 5.2, 6.2 `io`, 6.3 `ls` and the picker line, 6.7 `ls`, 11.

mod common;

use common::cli::Sandbox;
use std::fs;
use std::process::Output;

const LS: &str = "cards on work:
1. W-11 invoice PDF shows the wrong VAT [Todo] no note
2. W-12 migrate invoices to v13 [Doing] 39 min: open the MR, ask the DBA for the v13
3. W-7 fix rounding in act export [Doing] 2 h: check the 0.005 case
4. W-9 bump rust-version to 1.89 [Done] 1 d: nothing left
";

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

#[test]
fn ts6_3_ls_lists_cards_in_column_order() {
    let s = sandbox();
    let out = run(&s, &["ls"], 0);
    let stdout = text(&out.stdout);
    assert_eq!(stdout, LS);
    insta::assert_snapshot!(s.redact(&stdout));
}

#[test]
fn ts6_3_ls_open_omits_done_column() {
    let s = sandbox();
    let out = run(&s, &["ls", "--open"], 0);
    let stdout = text(&out.stdout);
    let want: Vec<&str> = LS.lines().take(4).collect();
    assert_eq!(stdout, want.join("\n") + "\n");
    insta::assert_snapshot!(s.redact(&stdout));
}

#[test]
fn ts4_7_ls_extra_column_after_configured() {
    let s = sandbox();
    fs::write(
        s.root().join("boards/work/W-20.md"),
        "---\ncolumn: Doign\n---\n# spike on caching\n",
    )
    .unwrap();
    let out = run(&s, &["ls"], 0);
    let stdout = text(&out.stdout);
    assert_eq!(
        stdout.lines().last(),
        Some("5. W-20 spike on caching [Doign] no note")
    );
    let err = text(&out.stderr);
    assert!(
        err.lines()
            .any(|l| l == "warning: W-20: unknown column 'Doign'"),
        "{err}"
    );
}

#[test]
fn ts6_3_ls_board_flag_after_subcommand() {
    let s = sandbox();
    let out = run(&s, &["ls", "--board", "home"], 0);
    assert_eq!(text(&out.stdout), "cards on home:\n");
    assert!(!s.root().join("boards/home").exists());
}

#[test]
fn ts6_7_ls_json_board_and_cards() {
    let s = sandbox();
    let out = run(&s, &["--json", "ls"], 0);
    let stdout = s.redact(&text(&out.stdout));
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["board"], "work");
    let ids: Vec<&str> = v["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["W-11", "W-12", "W-7", "W-9"]);
    insta::assert_snapshot!(stdout);
}

#[test]
fn ts5_2_unknown_config_key_warns_on_stderr() {
    let s = sandbox();
    let path = s.config_file();
    let config = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("colour = \"red\"\n{config}")).unwrap();
    let out = run(&s, &["ls"], 0);
    let err = text(&out.stderr);
    let warnings: Vec<&str> = err.lines().filter(|l| l.starts_with("warning: ")).collect();
    assert_eq!(warnings.len(), 1, "{err}");
    assert!(warnings[0].contains("colour"), "{err}");
    insta::assert_snapshot!(s.redact(&err));
}

#[test]
fn ts6_2_ls_unreadable_board_folder_exits_1_io() {
    let s = Sandbox::new();
    s.write_config();
    fs::create_dir_all(s.root().join("boards")).unwrap();
    fs::write(s.root().join("boards/work"), "").unwrap();
    let out = run(&s, &["ls"], 1);
    insta::assert_snapshot!("unreadable_text", s.redact(&text(&out.stderr)));
    let out = run(&s, &["--json", "ls"], 1);
    let stdout = s.redact(&text(&out.stdout));
    assert!(stdout.contains("\"code\":\"io\""), "{stdout}");
    insta::assert_snapshot!("unreadable_json", stdout);
}

#[test]
fn ts11_log_inside_board_folder_ignored_on_cli() {
    let mut s = sandbox();
    let log = s.root().join("boards/work/bs.log");
    let scenario = s.scenario_mut();
    *scenario = scenario.clone().log(&log);
    let out = run(&s, &["ls"], 0);
    let err = text(&out.stderr);
    assert!(
        err.lines()
            .any(|l| l == "warning: BRAIN_SWAP_LOG inside board work, ignored"),
        "{err}"
    );
    assert!(!log.exists());
}
