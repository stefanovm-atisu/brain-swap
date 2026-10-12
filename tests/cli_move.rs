//! TECHSPEC 6.1 card IDs, 6.2 exit codes for `move`, 6.3 `move`, 6.7 `move`.

mod common;

use common::cli::Sandbox;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
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

fn card(s: &Sandbox, id: &str) -> PathBuf {
    s.root().join(format!("boards/work/{id}.md"))
}

/// Every file of the board folder by name.
fn files(s: &Sandbox) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(s.root().join("boards/work"))
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (name, fs::read(p).unwrap())
        })
        .collect()
}

#[test]
fn fr39_move_changes_only_column_line() {
    let s = sandbox();
    let before = files(&s);
    let out = run(&s, &["move", "W-12", "Done"], 0);
    assert_eq!(text(&out.stdout), "moved W-12 to Done\n");
    let mut after = files(&s);
    let old = text(&before["W-12.md"]);
    let new = text(&after.remove("W-12.md").unwrap());
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    assert_eq!(a.len(), b.len());
    let diff: Vec<(&&str, &&str)> = a.iter().zip(&b).filter(|(x, y)| x != y).collect();
    assert_eq!(diff, vec![(&"column: Doing", &"column: Done")]);
    let mut before = before;
    before.remove("W-12.md");
    assert_eq!(before, after);
}

#[test]
fn ts6_3_move_matches_column_ignoring_case() {
    let s = sandbox();
    let out = run(&s, &["move", "W-11", "doing"], 0);
    assert_eq!(text(&out.stdout), "moved W-11 to Doing\n");
    let file = fs::read_to_string(card(&s, "W-11")).unwrap();
    assert!(file.lines().any(|l| l == "column: Doing"), "{file}");
}

#[test]
fn ts6_1_move_card_id_case_insensitive() {
    let s = sandbox();
    let out = run(&s, &["move", "w-12", "Done"], 0);
    assert_eq!(text(&out.stdout), "moved W-12 to Done\n");
}

#[test]
fn ts6_2_move_to_current_column_exits_0_writes_nothing() {
    let s = sandbox();
    let before = files(&s);
    let mtime = fs::metadata(card(&s, "W-12")).unwrap().modified().unwrap();
    let out = run(&s, &["move", "W-12", "Doing"], 0);
    assert_eq!(text(&out.stdout), "moved W-12 to Doing\n");
    assert_eq!(files(&s), before);
    let after = fs::metadata(card(&s, "W-12")).unwrap().modified().unwrap();
    assert_eq!(after, mtime);
}

#[test]
fn ts6_2_move_to_extra_column_exits_3() {
    let s = sandbox();
    fs::write(
        card(&s, "W-20"),
        "---\ncolumn: Doign\n---\n# spike on caching\n",
    )
    .unwrap();
    let before = fs::read(card(&s, "W-12")).unwrap();
    let out = run(&s, &["move", "W-12", "Doign"], 3);
    let err = text(&out.stderr);
    assert_eq!(
        err,
        "brain-swap: unknown column 'Doign' on work (Todo, Doing, Done)\n"
    );
    insta::assert_snapshot!("extra_column_text", err);
    let out = run(&s, &["--json", "move", "W-12", "Doign"], 3);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"not_found\""), "{stdout}");
    insta::assert_snapshot!("extra_column_json", stdout);
    assert_eq!(fs::read(card(&s, "W-12")).unwrap(), before);
}

#[test]
fn ts6_2_move_unknown_card_exits_3() {
    let s = sandbox();
    let out = run(&s, &["move", "W-99", "Done"], 3);
    let err = text(&out.stderr);
    assert_eq!(err, "brain-swap: W-99 not found\n");
    insta::assert_snapshot!("unknown_card_text", err);
    let out = run(&s, &["--json", "move", "W-99", "Done"], 3);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"not_found\""), "{stdout}");
    insta::assert_snapshot!("unknown_card_json", stdout);
}

#[test]
fn ts6_2_move_unreadable_card_exits_1() {
    let s = sandbox();
    let path = card(&s, "W-12");
    let file = fs::read_to_string(&path).unwrap().replacen(
        "# migrate invoices to v13\n",
        "# migrate invoices to v13\n<<<<<<< HEAD\n",
        1,
    );
    fs::write(&path, &file).unwrap();
    let out = run(&s, &["move", "W-12", "Done"], 1);
    insta::assert_snapshot!("unreadable_text", text(&out.stderr));
    let out = run(&s, &["--json", "move", "W-12", "Done"], 1);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"unreadable\""), "{stdout}");
    insta::assert_snapshot!("unreadable_json", stdout);
    assert_eq!(fs::read_to_string(&path).unwrap(), file);
}

#[test]
fn ts6_7_move_json_returns_card() {
    let s = sandbox();
    let out = run(&s, &["--json", "move", "W-11", "Doing"], 0);
    let stdout = s.redact(&text(&out.stdout));
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let c = &v["card"];
    assert_eq!(c["id"], "W-11");
    assert_eq!(c["column"], "Doing");
    assert_eq!(c["open"], true);
    assert!(c["latest_note"].is_null(), "{stdout}");
    insta::assert_snapshot!(stdout);
}

#[test]
fn ts6_2_move_to_current_lowercase_column_returns_canonical_card() {
    let s = sandbox();
    let path = card(&s, "W-12");
    let file = fs::read_to_string(&path)
        .unwrap()
        .replacen("column: Doing", "column: done", 1);
    fs::write(&path, &file).unwrap();
    let out = run(&s, &["--json", "move", "W-12", "Done"], 0);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["card"]["column"], "Done");
    assert_eq!(v["card"]["open"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), file);
}
