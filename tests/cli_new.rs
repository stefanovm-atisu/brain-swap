//! TECHSPEC 6.3 `new`, 6.7 Card, 4.3 invalid `board.md`, AS-9 first run.

mod common;

use brain_swap::core::card_file::parse;
use brain_swap::core::model::CardId;
use common::cli::Sandbox;
use jiff::tz::TimeZone;
use std::fs;
use std::process::Output;

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s
}

fn run(s: &Sandbox, args: &[&str], code: i32) -> Output {
    s.cmd(args).assert().code(code).get_output().clone()
}

fn work(s: &Sandbox) -> std::path::PathBuf {
    s.root().join("boards/work")
}

#[test]
fn fr13_new_creates_card_from_feature_skeleton() {
    let s = sandbox();
    let out = run(&s, &["new", "migrate", "invoices", "to", "v13"], 0);
    assert_eq!(
        text(&out.stdout),
        "created W-1: migrate invoices to v13 (Todo, work)\n"
    );
    insta::assert_snapshot!(fs::read_to_string(work(&s).join("W-1.md")).unwrap());
    assert_eq!(
        fs::read_to_string(work(&s).join("board.md")).unwrap(),
        "---\nnext: 2\n---\n"
    );
}

#[test]
fn ts6_3_new_joins_title_words() {
    let s = sandbox();
    let out = run(&s, &["new", "  fix", "rounding"], 0);
    assert_eq!(
        text(&out.stdout),
        "created W-1: fix rounding (Todo, work)\n"
    );
}

#[test]
fn ts6_3_new_without_title_exits_1() {
    let s = sandbox();
    let out = run(&s, &["new"], 1);
    assert_eq!(text(&out.stderr), "brain-swap: no title\n");
    insta::assert_snapshot!(text(&out.stderr));
    let out = run(&s, &["--json", "new"], 1);
    assert_eq!(text(&out.stderr), "");
    assert!(text(&out.stdout).contains("\"code\":\"invalid_input\""));
    insta::assert_snapshot!(text(&out.stdout));
    assert!(!work(&s).join("W-1.md").exists());
}

#[test]
fn ts6_7_new_json_returns_card() {
    let s = sandbox();
    let out = run(&s, &["--json", "new", "x"], 0);
    let stdout = s.redact(&text(&out.stdout));
    assert_eq!(
        stdout,
        "{\"v\":1,\"card\":{\"id\":\"W-1\",\"board\":\"work\",\"title\":\"x\",\"column\":\"Todo\",\"open\":true,\"path\":\"<sandbox>/boards/work/W-1.md\",\"latest_note\":null}}\n"
    );
    insta::assert_snapshot!(stdout);
}

#[test]
fn as9_new_on_first_run_writes_config_and_card() {
    let s = Sandbox::new();
    let out = run(&s, &["new", "first", "card"], 0);
    let config = s.config_file();
    assert_eq!(
        text(&out.stderr).lines().next(),
        Some(format!("created config {}", config.display()).as_str())
    );
    let toml = fs::read_to_string(&config).unwrap();
    let line = toml.lines().find(|l| l.starts_with("work = \"")).unwrap();
    let dir = line.split('"').nth(1).unwrap();
    assert!(std::path::Path::new(dir).join("W-1.md").is_file(), "{dir}");

    let s = Sandbox::new();
    let out = run(&s, &["--json", "new", "first", "card"], 0);
    assert_eq!(text(&out.stdout).lines().count(), 1);
    assert!(text(&out.stdout).starts_with("{\"v\":1,\"card\":"));
    assert_eq!(
        text(&out.stderr).lines().next(),
        Some(format!("warning: created config {}", s.config_file().display()).as_str())
    );
}

#[test]
fn ts4_3_new_with_invalid_board_md_exits_1_unreadable() {
    let s = sandbox();
    fs::create_dir_all(work(&s)).unwrap();
    let md = work(&s).join("board.md");
    fs::write(&md, "---\nletter W\n---\n").unwrap();
    let out = run(&s, &["new", "x"], 1);
    let err = text(&out.stderr);
    assert!(err.starts_with("brain-swap: board.md:2: "), "{err}");
    assert_eq!(err.lines().count(), 1, "{err}");
    insta::assert_snapshot!(err);
    let out = run(&s, &["--json", "new", "x"], 1);
    assert!(text(&out.stdout).contains("\"code\":\"unreadable\""));
    assert!(!work(&s).join("W-1.md").exists());
    assert_eq!(fs::read(&md).unwrap(), b"---\nletter W\n---\n");
}

#[test]
fn ts6_2_new_on_board_path_that_is_a_file_exits_1_io() {
    let s = sandbox();
    fs::create_dir_all(s.root().join("boards")).unwrap();
    fs::write(work(&s), "").unwrap();
    let out = run(&s, &["new", "x"], 1);
    insta::assert_snapshot!(s.redact(&text(&out.stderr)));
    let out = run(&s, &["--json", "new", "x"], 1);
    let stdout = s.redact(&text(&out.stdout));
    assert!(stdout.contains("\"code\":\"io\""), "{stdout}");
    insta::assert_snapshot!(stdout);
}

#[test]
fn ts5_4_new_warns_about_invalid_user_template() {
    let s = sandbox();
    let dir = s.root().join("config/brain-swap/templates");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("feature.md"), "---\nname Feature\n---\n## Why\n").unwrap();
    for args in [&["new", "x"][..], &["--json", "new", "y"]] {
        let out = run(&s, args, 0);
        let err = text(&out.stderr);
        assert!(err.contains("warning: template feature.md"), "{err}");
    }
}

#[test]
fn ts6_2_unknown_template_exits_3_listing_names() {
    let s = sandbox();
    let out = run(&s, &["new", "--template", "epic", "x"], 3);
    assert_eq!(
        text(&out.stderr),
        "brain-swap: unknown template 'epic' (Feature, Bug, Research, Chore)\n"
    );
    insta::assert_snapshot!(text(&out.stderr));
    assert!(!work(&s).join("W-1.md").exists());
    let out = run(&s, &["--json", "new", "--template", "epic", "x"], 3);
    assert!(text(&out.stdout).contains("\"code\":\"not_found\""));
    insta::assert_snapshot!(text(&out.stdout));
}

#[test]
fn ts6_2_unknown_column_exits_3_listing_columns() {
    let s = sandbox();
    let out = run(&s, &["new", "--column", "Doign", "x"], 3);
    assert_eq!(
        text(&out.stderr),
        "brain-swap: unknown column 'Doign' on work (Todo, Doing, Done)\n"
    );
    insta::assert_snapshot!(text(&out.stderr));
    let out = run(&s, &["--json", "new", "--column", "Doign", "x"], 3);
    assert!(text(&out.stdout).contains("\"code\":\"not_found\""));
    insta::assert_snapshot!(text(&out.stdout));
    assert!(!work(&s).join("W-1.md").exists());
}

#[test]
fn ts6_2_unknown_board_exits_3_listing_boards() {
    let s = sandbox();
    let out = run(&s, &["--board", "work2", "new", "x"], 3);
    assert_eq!(
        text(&out.stderr),
        "brain-swap: unknown board 'work2' (work, home, personal)\n"
    );
    insta::assert_snapshot!(text(&out.stderr));
    let out = run(&s, &["--json", "--board", "work2", "new", "x"], 3);
    assert!(text(&out.stdout).contains("\"code\":\"not_found\""));
    insta::assert_snapshot!(text(&out.stdout));
}

#[test]
fn i3_new_template_and_column_flags() {
    let s = sandbox();
    let out = run(
        &s,
        &["new", "--template", "bug", "--column", "Doing", "x"],
        0,
    );
    assert_eq!(text(&out.stdout), "created W-1: x (Doing, work)\n");
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    for want in [
        "column: Doing\n",
        "template: Bug\n",
        "## Symptom\n",
        "## Expected\n",
        "## Repro\n",
        "## Context\n",
    ] {
        assert!(card.contains(want), "{want}: {card}");
    }
}

#[test]
fn ts6_2_names_match_ignoring_case() {
    let s = sandbox();
    let args = [
        "--board",
        "WORK",
        "new",
        "--template",
        "BUG",
        "--column",
        "doing",
        "x",
    ];
    let out = run(&s, &args, 0);
    assert_eq!(text(&out.stdout), "created W-1: x (Doing, work)\n");
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert!(card.contains("column: Doing\n"), "{card}");
    assert!(card.contains("template: Bug\n"), "{card}");
}

#[test]
fn ts6_3_new_on_named_board_uses_its_letter() {
    let s = sandbox();
    let out = run(&s, &["new", "x", "--board", "home"], 0);
    assert_eq!(text(&out.stdout), "created H-1: x (Todo, home)\n");
    assert!(s.root().join("boards/home/H-1.md").is_file());
}

fn run_in(s: &Sandbox, args: &[&str], stdin: &str, code: i32) -> Output {
    s.cmd(args)
        .write_stdin(stdin)
        .assert()
        .code(code)
        .get_output()
        .clone()
}

const BODY: &str = "# migrate invoices\n\n## Goal\nswitch reads\n";

#[test]
fn fr13_new_body_from_stdin_replaces_skeleton() {
    let s = sandbox();
    let out = run_in(&s, &["new", "--body", "-"], BODY, 0);
    assert_eq!(
        text(&out.stdout),
        "created W-1: migrate invoices (Todo, work)\n"
    );
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert!(card.contains("# migrate invoices\n\n## Goal\nswitch reads\n"));
    assert!(!card.contains("## Acceptance") && !card.contains("## Context"));
    assert_eq!(card.matches("## Timeline").count(), 1);
    assert!(card.ends_with("## Timeline\n"));
    insta::assert_snapshot!(card);
}

#[test]
fn ts6_3_new_positional_title_wins_and_heading_removed() {
    let s = sandbox();
    let out = run_in(&s, &["new", "--body", "-", "v13", "migration"], BODY, 0);
    assert_eq!(
        text(&out.stdout),
        "created W-1: v13 migration (Todo, work)\n"
    );
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert!(!card.contains("# migrate invoices"), "{card}");
    assert!(
        card.contains("# v13 migration\n\n## Goal\nswitch reads\n"),
        "{card}"
    );
    insta::assert_snapshot!(card);
}

#[test]
fn ts6_3_new_body_title_is_first_nonblank_line() {
    let s = sandbox();
    let out = run_in(&s, &["new", "--body", "-"], "\n\n# t\nrest\n", 0);
    assert_eq!(text(&out.stdout), "created W-1: t (Todo, work)\n");
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert!(
        card.ends_with("---\n# t\n\nrest\n\n## Timeline\n"),
        "{card}"
    );
}

#[test]
fn ts6_3_new_body_later_heading_is_not_title() {
    let s = sandbox();
    let out = run_in(&s, &["new", "--body", "-"], "intro\n# t\n", 1);
    assert_eq!(text(&out.stderr), "brain-swap: no title\n");
    assert!(!work(&s).join("W-1.md").exists());
}

#[test]
fn r4_new_body_drops_timeline_heading_with_warning() {
    let s = sandbox();
    let stdin = "# t\n## Timeline\n### 2026-10-08T10:00:00+03:00\n- Doing: x\n";
    let out = run_in(&s, &["new", "--body", "-"], stdin, 0);
    assert!(
        text(&out.stderr).contains("warning: ## Timeline in body dropped\n"),
        "{}",
        text(&out.stderr)
    );
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert_eq!(card.lines().filter(|l| *l == "## Timeline").count(), 1);
    assert_eq!(card.lines().filter(|l| l.starts_with("# ")).count(), 1);
    let env = common::env::core_env(&s.root().join("home"), "2026-10-08T11:52:00+03:00");
    let board = brain_swap::core::board::load("work", &work(&s), &env).unwrap();
    assert_eq!(board.cards.len(), 1);
    assert!(board.cards[0].notes.is_empty());
}

#[test]
fn ts6_3_new_body_empty_stdin_keeps_positional_title() {
    let s = sandbox();
    run_in(&s, &["new", "--body", "-", "x"], "", 0);
    let card = fs::read_to_string(work(&s).join("W-1.md")).unwrap();
    assert!(card.ends_with("---\n# x\n\n## Timeline\n"), "{card}");
    assert!(!card.contains("## Goal"), "{card}");
}

#[test]
fn ts6_3_new_body_flag_accepts_only_dash() {
    let s = sandbox();
    let out = run(&s, &["--json", "new", "--body", "notes.md", "x"], 2);
    assert!(text(&out.stdout).contains("\"code\":\"usage\""));
    assert!(!work(&s).join("W-1.md").exists());
}

#[test]
fn r7_cli_new_without_fixed_clock_succeeds() {
    let mut s = Sandbox::new();
    s.write_config();
    s.copy_board("work_4_9", "work");
    let sc = s.scenario_mut();
    *sc = sc.clone().unset("BRAIN_SWAP_NOW");
    let out = run(&s, &["new", "x"], 0);
    assert_eq!(text(&out.stdout), "created W-13: x (Todo, work)\n");
    let path = work(&s).join("W-13.md");
    let id = CardId::parse("W-13").unwrap();
    let card = parse(id, &path, &fs::read(&path).unwrap(), "Todo", &TimeZone::UTC).card;
    assert_eq!(card.title, "x");
}
