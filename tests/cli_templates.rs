//! TECHSPEC 6.1 and 6.3 `templates`: the binary entry and its first subcommand.

mod common;

use common::cli::Sandbox;
use std::fs;

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn sandbox() -> Sandbox {
    let s = Sandbox::new();
    s.write_config();
    s
}

fn spike(dir: &std::path::Path) {
    let dir = dir.join("brain-swap/templates");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spike.md"), "## Question\n").unwrap();
}

const HEADERS: [&str; 4] = [
    "template: Feature (key f)",
    "template: Bug (key b)",
    "template: Research (key r)",
    "template: Chore (key c)",
];

#[test]
fn fr11_templates_prints_builtins() {
    let s = Sandbox::new();
    let out = s.cmd(&["templates"]).assert().code(0).get_output().clone();
    insta::assert_snapshot!("builtins", s.redact(&stdout(&out)));
}

#[test]
fn fr11_templates_reads_user_folder_from_xdg_config_home() {
    let s = Sandbox::new();
    spike(&s.root().join("config"));
    let out = s.cmd(&["templates"]).assert().code(0).get_output().clone();
    let text = stdout(&out);
    assert!(text.contains("template: Spike\n## Question\n"), "{text}");
    for h in HEADERS {
        assert!(text.lines().any(|l| l == h), "{h}: {text}");
    }
}

#[test]
fn ts6_1_config_folder_falls_back_to_home() {
    let mut s = Sandbox::new();
    let sc = s.scenario_mut();
    *sc = sc.clone().unset("XDG_CONFIG_HOME");
    spike(&s.root().join("home/.config"));
    let out = s.cmd(&["templates"]).assert().code(0).get_output().clone();
    assert!(stdout(&out).contains("template: Spike"), "{}", stdout(&out));
}

#[test]
fn ts6_3_templates_never_parses_config() {
    let s = Sandbox::new();
    let path = s.config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "default_board = \n").unwrap();
    let out = s.cmd(&["templates"]).assert().code(0).get_output().clone();
    insta::assert_snapshot!("builtins", s.redact(&stdout(&out)));
    assert_eq!(fs::read(&path).unwrap(), b"default_board = \n");
}

#[test]
fn ts6_3_templates_writes_no_config() {
    let s = Sandbox::new();
    s.cmd(&["templates"]).assert().code(0);
    assert!(!s.config_file().exists());
}

#[test]
fn fr33_help_lists_every_subcommand() {
    let s = sandbox();
    let out = s.cmd(&["--help"]).assert().code(0).get_output().clone();
    let text = stdout(&out);
    for name in [
        "new",
        "park",
        "context",
        "show",
        "ls",
        "link",
        "move",
        "locate",
        "jump",
        "templates",
        "install",
    ] {
        assert!(
            text.lines()
                .any(|l| l.split_whitespace().next() == Some(name)),
            "{name}: {text}"
        );
    }
}

#[test]
fn ts6_1_invalid_brain_swap_now_exits_1() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc.clone().now("tomorrow");
    s.cmd(&["ls"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("brain-swap: BRAIN_SWAP_NOW: invalid stamp 'tomorrow'\n");
}

fn json_templates(s: &Sandbox, args: &[&str]) -> (String, Vec<serde_json::Value>) {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    let text = stdout(&out);
    assert_eq!(text.lines().count(), 1, "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let ts = v["templates"].as_array().unwrap().clone();
    (text, ts)
}

#[test]
fn fr35_templates_json_is_one_object_v_first() {
    let s = Sandbox::new();
    let (text, ts) = json_templates(&s, &["--json", "templates"]);
    assert!(text.starts_with("{\"v\":1,"), "{text}");
    assert_eq!(ts.len(), 4);
    for t in &ts {
        for k in ["name", "key", "skeleton"] {
            assert!(t.get(k).is_some(), "{k}: {t}");
        }
    }
    insta::assert_snapshot!(s.redact(&text));
}

#[test]
fn fr35_json_flag_after_subcommand() {
    let s = Sandbox::new();
    let before = s.cmd(&["--json", "templates"]).output().unwrap().stdout;
    let after = s.cmd(&["templates", "--json"]).output().unwrap().stdout;
    assert!(before.starts_with(b"{\"v\":1,"));
    assert_eq!(after, before);
}

#[test]
fn fr35_templates_json_null_key() {
    let s = Sandbox::new();
    spike(&s.root().join("config"));
    let (_, ts) = json_templates(&s, &["--json", "templates"]);
    let spike = ts.iter().find(|t| t["name"] == "Spike").unwrap();
    assert_eq!(spike["key"], serde_json::Value::Null);
}

#[test]
fn ts5_4_unreadable_template_folder_keeps_json_one_object() {
    let s = Sandbox::new();
    let dir = s.root().join("config/brain-swap");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("templates"), "not a folder").unwrap();
    let (_, ts) = json_templates(&s, &["--json", "templates"]);
    let names: Vec<_> = ts.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Feature", "Bug", "Research", "Chore"]);
    let out = s.cmd(&["--json", "templates"]).output().unwrap();
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.lines().any(|l| l.starts_with("warning: ")), "{err}");
}

#[test]
fn ts6_2_invalid_brain_swap_now_json_envelope() {
    let mut s = sandbox();
    let sc = s.scenario_mut();
    *sc = sc.clone().now("tomorrow");
    s.cmd(&["--json", "ls"]).assert().code(1).stderr("").stdout(
        "{\"v\":1,\"error\":{\"code\":\"invalid_input\",\"message\":\"BRAIN_SWAP_NOW: invalid stamp 'tomorrow'\"}}\n",
    );
}
