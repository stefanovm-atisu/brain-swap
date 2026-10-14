//! TECHSPEC 6.2 exit codes for `park`, 6.3 `park --card` from stdin, 6.7 `park`, 10 lock.

mod common;

use brain_swap::core::card_file::parse;
use brain_swap::core::env::Env;
use brain_swap::core::session::{read_pane_hint, read_reference, reference_path};
use brain_swap::core::store::lock_board;
use brain_swap::core::time::format_stamp;
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
    // TECHSPEC 9.1: no moved-pane check, so park calls no herdr.
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

/// Parks `Doing: x` on W-12 with `args` appended, exit 0; returns stderr and the place line.
fn park_session(s: &Sandbox, args: &[&str]) -> (String, String) {
    let mut all = vec!["park", "--card", "W-12"];
    all.extend(args);
    let out = park(s, &all, "Doing: x", "elsewhere", 0);
    (text(&out.stderr), i6_place_line(s))
}

/// The fake herdr pane `w4V:p9` answering per `scenario`.
fn in_herdr(s: &mut Sandbox, scenario: &str) {
    let sc = s.scenario_mut();
    *sc = sc
        .clone()
        .herdr("w4V:p9", "w4V:t1", "w4V")
        .fake_scenario(scenario);
}

fn herdr_calls(s: &Sandbox) -> Vec<String> {
    common::fake_herdr::calls(&s.root().join("home"))
}

#[test]
fn fr41_flag_session_wins_over_env() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc.clone().session("b");
    let (err, place) = park_session(&s, &["--session", "a"]);
    assert!(place.ends_with(" session=a -->"), "{place}");
    assert_eq!(err, "");
}

#[test]
fn fr41_missing_flag_uses_env_without_warning() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc.clone().session("b");
    let (err, place) = park_session(&s, &[]);
    assert!(place.ends_with(" session=b -->"), "{place}");
    assert_eq!(err, "");
}

#[test]
fn fr41_empty_flag_warns_and_falls_back() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc.clone().session("b");
    let (err, place) = park_session(&s, &["--session", ""]);
    assert_eq!(err, "warning: session id not substituted\n");
    assert!(place.ends_with(" session=b -->"), "{place}");
}

#[test]
fn fr41_placeholder_flag_warns_and_degrades_to_none() {
    let s = sandbox();
    let (err, place) = park_session(&s, &["--session", "${CLAUDE_SESSION_ID}"]);
    assert_eq!(err, "warning: session id not substituted\n");
    assert!(!place.contains("session="), "{place}");
}

#[test]
fn fr41_invalid_flag_warns_invalid_session_id() {
    let s = sandbox();
    let (err, place) = park_session(&s, &["--session", "a b"]);
    assert_eq!(err, "warning: invalid session id\n");
    assert!(!place.contains("session="), "{place}");
}

#[test]
fn fr41_inside_herdr_falls_back_to_pane_session() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-session");
    let (_, place) = park_session(&s, &["--session", ""]);
    assert!(place.ends_with(" session=herdr-s1 -->"), "{place}");
    // TECHSPEC 9.1: no moved-pane check, so only step 3 calls herdr.
    assert_eq!(herdr_calls(&s), ["pane get w4V:p9"]);
}

#[test]
fn fr41_valid_env_session_never_calls_herdr() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-session");
    let sc = s.scenario_mut();
    *sc = sc.clone().session("b");
    park_session(&s, &[]);
    assert!(herdr_calls(&s).is_empty());
}

#[test]
fn fr41_pane_without_agent_session_resolves_none() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-no-agent");
    let (_, place) = park_session(&s, &["--session", ""]);
    assert!(!place.contains("session="), "{place}");
}

#[test]
fn ts6_1_brain_swap_log_receives_herdr_call() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-session");
    let log = s.root().join("bs.log");
    let sc = s.scenario_mut();
    *sc = sc.clone().log(&log);
    park_session(&s, &["--session", ""]);
    let calls = herdr_calls(&s);
    assert!(!calls.is_empty());
    let text = fs::read_to_string(&log).unwrap_or_default();
    let lines: Vec<_> = text.lines().filter(|l| l.contains(" herdr ")).collect();
    assert_eq!(lines.len(), calls.len(), "{text}");
    for (line, call) in lines.iter().zip(&calls) {
        assert!(line.contains(&format!(" herdr {call} exit=")), "{line}");
    }
}

#[test]
fn ts11_cli_ignores_log_inside_board_folder() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-session");
    let log = s.root().join("boards/work/bs.log");
    let sc = s.scenario_mut();
    *sc = sc.clone().log(&log);
    let (err, _) = park_session(&s, &["--session", ""]);
    assert!(!herdr_calls(&s).is_empty());
    assert!(
        err.contains("warning: BRAIN_SWAP_LOG inside board work, ignored"),
        "{err}"
    );
    assert!(!log.exists());
}

/// The `Env` the sandbox's processes see, for reading references and hints through E1-F7.
fn sandbox_env(s: &mut Sandbox) -> Env {
    let vars = s.scenario_mut().vars();
    Env::from_vars(
        vars,
        s.root().to_path_buf(),
        std::process::id(),
        TimeZone::UTC,
        jiff::Zoned::now(),
    )
    .unwrap()
}

/// Every file name in `dir`, none when it is missing.
fn names(dir: &std::path::Path) -> Vec<String> {
    let entries = fs::read_dir(dir).into_iter().flatten().flatten();
    entries
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// The sandbox without the `work` fixture, so `new` creates W-1.
fn empty_sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s
}

/// Test 1's step: `park --card W-12 --session s1` outside herdr from `<sandbox>/elsewhere`.
fn park_sets_reference(s: &Sandbox) {
    let args = ["park", "--card", "W-12", "--session", "s1"];
    park(s, &args, "Doing: x", "elsewhere", 0);
}

#[test]
fn fr57_park_card_sets_reference() {
    let mut s = sandbox();
    park_sets_reference(&s);
    let env = sandbox_env(&mut s);
    let r = read_reference(&env, "s1").expect("reference for s1");
    assert_eq!(r.card.to_string(), "W-12");
    assert_eq!(r.board, "work");
    let cwd = r.cwd.map(|c| s.redact(&c.display().to_string()));
    assert_eq!(cwd.as_deref(), Some("<sandbox>/elsewhere"));
    assert_eq!(format_stamp(&r.set_at), "2026-10-08T08:52:00+00:00");
    assert_eq!(r.pane, None);
}

#[test]
fn fr57_park_by_reference_needs_no_card() {
    let mut s = sandbox();
    let fixture = fs::read_to_string(card(&s, "W-12")).unwrap();
    park_sets_reference(&s);
    let env = sandbox_env(&mut s);
    let path = reference_path(&env, "s1").unwrap();
    let reference = fs::read(&path).unwrap();
    let out = park(&s, &["park", "--session", "s1"], "Doing: y", "elsewhere", 0);
    assert_eq!(
        text(&out.stdout),
        "parked to W-12: migrate invoices to v13\n"
    );
    let notes = |t: &str| t.lines().filter(|l| l.starts_with("### ")).count();
    let after = fs::read_to_string(card(&s, "W-12")).unwrap();
    assert_eq!(notes(&after), notes(&fixture) + 2);
    assert_eq!(fs::read(&path).unwrap(), reference);
}

#[test]
fn ts6_2_park_without_card_or_reference_exits_4() {
    let s = sandbox();
    let before = files(&s);
    let out = park(&s, &["park", "--session", "s9"], "Doing: x", "x", 4);
    let err = text(&out.stderr);
    assert_eq!(err, "brain-swap: no reference: pass --card <ID>\n");
    insta::assert_snapshot!("no_reference_text", s.redact(&err));
    let args = ["--json", "park", "--session", "s9"];
    let out = park(&s, &args, "Doing: x", "x", 4);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\"code\":\"no_reference\""), "{stdout}");
    insta::assert_snapshot!("no_reference_json", s.redact(&stdout));
    assert_eq!(files(&s), before);
}

#[test]
fn ts6_2_park_without_any_session_exits_4() {
    let s = sandbox();
    park(&s, &["park"], "Doing: x", "x", 4);
}

#[test]
fn i9_reference_to_deleted_card_exits_4() {
    let s = sandbox();
    park_sets_reference(&s);
    fs::remove_file(card(&s, "W-12")).unwrap();
    park(&s, &["park", "--session", "s1"], "Doing: x", "x", 4);
}

#[test]
fn ts12_1_new_empty_session_in_fake_herdr_sets_reference() {
    let mut s = empty_sandbox();
    in_herdr(&mut s, "pane-session");
    let out = s.cmd(&["new", "--session", "", "x"]).assert().code(0);
    let err = text(&out.get_output().stderr);
    assert!(err.contains("warning: session id not substituted"), "{err}");
    let env = sandbox_env(&mut s);
    let r = read_reference(&env, "herdr-s1").expect("reference for herdr-s1");
    assert_eq!(r.card.to_string(), "W-1");
    assert_eq!(r.board, "work");
    assert_eq!(r.pane.as_deref(), Some("w4V:p9"));
}

#[test]
fn ts6_4_reference_inside_herdr_writes_pane_hint() {
    let mut s = sandbox();
    in_herdr(&mut s, "pane-session");
    park_sets_reference(&s);
    let env = sandbox_env(&mut s);
    let hint = read_pane_hint(&env, "w4V:p9").expect("hint for w4V:p9");
    assert_eq!(hint.card.to_string(), "W-12");
    assert_eq!(hint.session, "s1");
}

#[test]
fn ts6_3_new_with_session_sets_reference() {
    let mut s = empty_sandbox();
    s.cmd(&["new", "--session", "s1", "x"]).assert().code(0);
    let env = sandbox_env(&mut s);
    let r = read_reference(&env, "s1").expect("reference for s1");
    assert_eq!(r.card.to_string(), "W-1");
    assert_eq!(names(&env.panes_dir()), Vec::<String>::new());
}

#[test]
fn ts6_4_no_session_sets_no_reference() {
    let mut s = empty_sandbox();
    s.cmd(&["new", "x"]).assert().code(0);
    let env = sandbox_env(&mut s);
    assert_eq!(names(&env.sessions_dir()), Vec::<String>::new());
}

#[test]
fn ts6_3_failed_command_sets_no_reference() {
    let mut s = sandbox();
    let args = ["park", "--card", "W-99", "--session", "s1"];
    park(&s, &args, "Doing: x", "x", 3);
    let args = ["new", "--template", "epic", "--session", "s1", "x"];
    s.cmd(&args).assert().code(3);
    let env = sandbox_env(&mut s);
    assert!(read_reference(&env, "s1").is_none());
    assert!(!reference_path(&env, "s1").unwrap().exists());
}
