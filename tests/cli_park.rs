//! TECHSPEC 6.2 exit codes for `park`, 6.3 `park --card` from stdin, 6.7 `park`, 10 lock.

mod common;

use brain_swap::core::card_file::parse;
use brain_swap::core::env::Env;
use brain_swap::core::store::lock_board;
use common::cli::Sandbox;
use jiff::tz::TimeZone;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Output;
use std::time::{Duration, Instant};

const NOTE: &str = "Doing: a\nNext: b\nWatch out: c\n";

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s.copy_board("work", "work");
    s
}

/// Runs `args` with `stdin` from `<root>/<cwd>`, which it creates, and checks the exit code.
fn park(s: &Sandbox, args: &[&str], stdin: &str, cwd: &str, code: i32) -> Output {
    let dir = s.root().join(cwd);
    fs::create_dir_all(&dir).unwrap();
    s.cmd(args)
        .current_dir(dir)
        .write_stdin(stdin)
        .assert()
        .code(code)
        .get_output()
        .clone()
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
fn fr16_park_appends_note_only_to_timeline() {
    let s = sandbox();
    let mut before = files(&s);
    let out = park(&s, &["park", "--card", "W-12"], NOTE, "ati.billing", 0);
    assert_eq!(
        text(&out.stdout),
        "parked to W-12: migrate invoices to v13\n"
    );
    let mut after = files(&s);
    let old = text(&before.remove("W-12.md").unwrap());
    let new = text(&after.remove("W-12.md").unwrap());
    let root = s.root().display();
    let note = format!(
        "\n### 2026-10-08T08:52:00+00:00\n- Doing: a\n- Next: b\n- Watch out: c\n<!-- where: cwd={root}/ati.billing -->\n"
    );
    assert_eq!(new, old + &note);
    insta::assert_snapshot!(s.redact(&new));
    assert_eq!(before, after);
}

#[test]
fn as6_park_from_stdin_writes_no_auto() {
    let s = sandbox();
    park(&s, &["park", "--card", "W-12"], NOTE, "ati.billing", 0);
    let file = fs::read_to_string(card(&s, "W-12")).unwrap();
    let heading = file.lines().rfind(|l| l.starts_with("### "));
    assert_eq!(heading, Some("### 2026-10-08T08:52:00+00:00"));
    assert!(file.lines().any(|l| l == "column: Doing"), "{file}");
}

#[test]
fn fr15_park_outside_herdr_records_cwd_only() {
    let s = sandbox();
    park(&s, &["park", "--card", "W-11"], "Doing: a", "x", 0);
    let file = s.redact(&fs::read_to_string(card(&s, "W-11")).unwrap());
    let place = file.lines().rfind(|l| l.starts_with("<!-- where:"));
    assert_eq!(place, Some("<!-- where: cwd=<sandbox>/x -->"));
}

#[test]
fn ts6_1_park_card_id_case_insensitive() {
    let s = sandbox();
    let out = park(&s, &["park", "--card", "w-12"], NOTE, "ati.billing", 0);
    assert_eq!(
        text(&out.stdout),
        "parked to W-12: migrate invoices to v13\n"
    );
}

#[test]
fn ts6_7_park_json_returns_card() {
    let s = sandbox();
    let args = ["--json", "park", "--card", "W-12"];
    let out = park(&s, &args, NOTE, "ati.billing", 0);
    let stdout = s.redact(&text(&out.stdout));
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let n = &v["card"]["latest_note"];
    assert_eq!(n["doing"], "a");
    assert_eq!(n["next"], "b");
    assert_eq!(n["watch_out"], "c");
    assert_eq!(n["auto"], false);
    assert_eq!(n["age_min"], 0);
    assert!(n["by_this_session"].is_null(), "{stdout}");
    assert!(n["place"]["herdr"].is_null(), "{stdout}");
    insta::assert_snapshot!(stdout);
}

#[test]
fn ts6_2_park_unknown_card_exits_3() {
    let s = sandbox();
    let before = files(&s);
    let out = park(&s, &["park", "--card", "W-99"], "Doing: a", "x", 3);
    let err = text(&out.stderr);
    assert_eq!(err, "brain-swap: W-99 not found\n");
    insta::assert_snapshot!("unknown_card_text", err);
    let args = ["--json", "park", "--card", "W-99"];
    let out = park(&s, &args, "Doing: a", "x", 3);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"not_found\""), "{stdout}");
    insta::assert_snapshot!("unknown_card_json", stdout);
    assert_eq!(files(&s), before);
}

#[test]
fn ts6_2_park_unreadable_card_exits_1() {
    let s = sandbox();
    let w12 = card(&s, "W-12");
    let conflict = fs::read_to_string(&w12).unwrap().replacen(
        "# migrate invoices to v13\n",
        "# migrate invoices to v13\n<<<<<<< HEAD\n",
        1,
    );
    fs::write(&w12, &conflict).unwrap();
    let w11 = card(&s, "W-11");
    let mut bad = fs::read(&w11).unwrap();
    bad.push(0xFF);
    fs::write(&w11, &bad).unwrap();
    for (id, path, bytes) in [("W-12", &w12, conflict.into_bytes()), ("W-11", &w11, bad)] {
        let out = park(&s, &["park", "--card", id], "Doing: a", "x", 1);
        insta::assert_snapshot!(format!("unreadable_text_{id}"), text(&out.stderr));
        let out = park(&s, &["--json", "park", "--card", id], "Doing: a", "x", 1);
        let stdout = text(&out.stdout);
        assert!(stdout.contains("\"code\":\"unreadable\""), "{stdout}");
        insta::assert_snapshot!(format!("unreadable_json_{id}"), stdout);
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn ts6_2_park_busy_exits_5() {
    let mut s = sandbox();
    let env = Env::from_vars(
        s.scenario_mut().vars(),
        s.root().to_path_buf(),
        std::process::id(),
        TimeZone::UTC,
        jiff::Zoned::now(),
    )
    .unwrap();
    let before = fs::read(card(&s, "W-12")).unwrap();
    // Dropped at the end of the test, also when an assertion panics.
    let _lock = lock_board(&env, &s.root().join("boards/work")).unwrap();
    let start = Instant::now();
    let out = park(&s, &["park", "--card", "W-12"], "Doing: a", "x", 5);
    assert!(start.elapsed() >= Duration::from_millis(1900));
    insta::assert_snapshot!("busy_text", text(&out.stderr));
    let start = Instant::now();
    let out = park(
        &s,
        &["--json", "park", "--card", "W-12"],
        "Doing: a",
        "x",
        5,
    );
    assert!(start.elapsed() >= Duration::from_millis(1900));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"busy\""), "{stdout}");
    insta::assert_snapshot!("busy_json", stdout);
    assert_eq!(fs::read(card(&s, "W-12")).unwrap(), before);
}

#[test]
fn fr18_park_empty_note_exits_1() {
    let s = sandbox();
    let before = fs::read(card(&s, "W-12")).unwrap();
    for (i, stdin) in ["Doing:\nNext:  \n", ""].into_iter().enumerate() {
        let out = park(&s, &["park", "--card", "W-12"], stdin, "x", 1);
        let err = text(&out.stderr);
        assert_eq!(err, "brain-swap: empty note\n");
        insta::assert_snapshot!(format!("empty_note_text_{i}"), err);
        let out = park(&s, &["--json", "park", "--card", "W-12"], stdin, "x", 1);
        let stdout = text(&out.stdout);
        assert!(stdout.contains("\"code\":\"invalid_input\""), "{stdout}");
        insta::assert_snapshot!(format!("empty_note_json_{i}"), stdout);
        assert_eq!(fs::read(card(&s, "W-12")).unwrap(), before);
    }
}

#[test]
fn ts6_5_continuation_written_indented() {
    let s = sandbox();
    park(&s, &["park", "--card", "W-12"], "Next: a\nb\n", "x", 0);
    let file = fs::read_to_string(card(&s, "W-12")).unwrap();
    assert!(file.contains("\n- Next: a\n  b\n"), "{file}");
}

#[test]
fn as6_park_from_terminal_prompts_and_writes_no_auto() {
    let mut s = sandbox();
    let bin = env!("CARGO_BIN_EXE_brain-swap");
    let script = std::path::Path::new("/usr/bin/script");
    // Linux only: macOS `script` takes other flags (A-E2-35).
    let version = common::spawn::command(s.scenario_mut(), script)
        .arg("--version")
        .output();
    if !version.is_ok_and(|v| text(&v.stdout).contains("util-linux")) {
        eprintln!("skipped: /usr/bin/script is not util-linux");
        return;
    }
    let out = common::spawn::command(s.scenario_mut(), script)
        .args(["-qec", &format!("{bin} park --card W-12"), "/dev/null"])
        .current_dir(s.root().join("elsewhere"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            c.stdin.take().unwrap().write_all(b"a\nb\nc\n")?;
            c.wait_with_output()
        })
        .unwrap();
    let all = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{all}");
    let at = |needle: &str| {
        all.find(needle)
            .unwrap_or_else(|| panic!("{needle}: {all}"))
    };
    assert!(at("Doing: ") < at("Next: "), "{all}");
    assert!(at("Next: ") < at("Watch out: "), "{all}");
    assert!(at("Watch out: ") < at("parked to W-12: migrate invoices to v13"));
    let file = fs::read_to_string(card(&s, "W-12")).unwrap();
    let (_, note) = file.rsplit_once("\n### ").unwrap();
    let mut lines = note.lines();
    assert_eq!(lines.next(), Some("2026-10-08T08:52:00+00:00"));
    let parts: Vec<_> = lines.take(3).collect();
    assert_eq!(parts, ["- Doing: a", "- Next: b", "- Watch out: c"]);
}

/// Reloads W-12, checks I6 on its latest note (`at`, `place.cwd`, a non-empty part) and
/// returns the note's place line.
fn i6_place_line(s: &Sandbox) -> String {
    let path = card(s, "W-12");
    let id = brain_swap::core::model::CardId::parse("W-12").unwrap();
    let parsed = parse(id, &path, &fs::read(&path).unwrap(), "Todo", &TimeZone::UTC);
    let n = parsed.card.latest_note().unwrap();
    assert!(n.at.is_some() && n.place.cwd.is_some() && !n.doing.is_empty());
    let file = s.redact(&fs::read_to_string(&path).unwrap());
    let place = file.lines().rfind(|l| l.starts_with("<!-- where:"));
    place.unwrap().to_string()
}

#[test]
fn ts6_3_park_auto_flag_marks_heading() {
    let s = sandbox();
    park(
        &s,
        &["park", "--card", "W-12", "--auto"],
        "Doing: a",
        "elsewhere",
        0,
    );
    i6_place_line(&s);
    let file = fs::read_to_string(card(&s, "W-12")).unwrap();
    let heading = file.lines().rfind(|l| l.starts_with("### "));
    assert_eq!(heading, Some("### 2026-10-08T08:52:00+00:00 auto"));
}

#[test]
fn fr15_park_inside_herdr_records_pane_tab_workspace() {
    let mut s = sandbox();
    let scenario = s.scenario_mut();
    *scenario = scenario
        .clone()
        .herdr("w4V:p9", "w4V:t1", "w4V")
        .fake_scenario("pane-session");
    let args = ["park", "--card", "W-12", "--session", "s1"];
    park(&s, &args, "Doing: a", "elsewhere", 0);
    assert_eq!(
        i6_place_line(&s),
        "<!-- where: cwd=<sandbox>/elsewhere pane=w4V:p9 tab=w4V:t1 workspace=w4V session=s1 -->"
    );
    // TECHSPEC 16 SP-4: no moved-pane check, so park calls no herdr.
    assert!(common::fake_herdr::calls(&s.root().join("home")).is_empty());
}

#[test]
fn fr15_park_records_env_session() {
    let mut s = sandbox();
    let scenario = s.scenario_mut();
    *scenario = scenario.clone().session("s2");
    park(&s, &["park", "--card", "W-12"], "Doing: a", "elsewhere", 0);
    assert_eq!(
        i6_place_line(&s),
        "<!-- where: cwd=<sandbox>/elsewhere session=s2 -->"
    );
}
