//! TECHSPEC 6.3 `new`, 6.7 Card, 4.3 invalid `board.md`, AS-9 first run.

mod common;

use common::cli::Sandbox;
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
