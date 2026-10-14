//! TECHSPEC 8.8 and 6.7 `install claude`: the embedded skills copied into Claude Code's
//! skills folder under a temp `HOME` (FR-67).

mod common;

use brain_swap::adapters::claude::SKILLS;
use common::cli::Sandbox;
use std::fs;
use std::path::{Path, PathBuf};

const RELOAD: &str = "run /reload-skills in sessions already open";

fn skills(s: &Sandbox) -> PathBuf {
    s.root().join("home/.claude/skills")
}

/// Runs `args` expecting exit 0: stdout, stderr.
fn run(s: &Sandbox, args: &[&str]) -> (String, String) {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (text(out.stdout), text(out.stderr))
}

/// Every entry under `dir`, relative to it, sorted.
fn tree(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        out.push(p.strip_prefix(dir).unwrap().to_path_buf());
        if p.is_dir() {
            out.extend(
                tree(&p)
                    .into_iter()
                    .map(|q| p.strip_prefix(dir).unwrap().join(q)),
            );
        }
    }
    out.sort();
    out
}

#[test]
fn fr67_install_writes_the_copies() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let body = fs::read(skills(&s).join(skill.name).join("SKILL.md")).unwrap();
        assert_eq!(body, skill.body.as_bytes(), "{}", skill.name);
    }
    assert!(!s.root().join("data/brain-swap/pack").exists());
}

#[test]
fn fr67_install_writes_plain_folders() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let dir = skills(&s).join(skill.name);
        assert!(fs::symlink_metadata(&dir).unwrap().file_type().is_dir());
        assert_eq!(tree(&dir), [PathBuf::from("SKILL.md")]);
        let md = fs::symlink_metadata(dir.join("SKILL.md")).unwrap();
        assert!(md.file_type().is_file());
    }
}

#[test]
fn ts8_8_install_prints_reload_hint_when_it_created_the_skills_dir() {
    let s = Sandbox::new();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(out.lines().any(|l| l == RELOAD), "{out}");
}

#[test]
fn ts8_8_install_omits_reload_hint_when_the_skills_dir_existed() {
    let s = Sandbox::new();
    fs::create_dir_all(skills(&s)).unwrap();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(!out.contains("run /reload-skills"), "{out}");
}

#[test]
fn ts8_8_install_suggests_the_allow_rule() {
    let s = Sandbox::new();
    fs::create_dir_all(skills(&s)).unwrap();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(
        out.lines()
            .any(|l| l.starts_with("tip: ") && l.contains("Bash(brain-swap:*)")),
        "{out}"
    );
}

#[test]
fn ts8_8_install_text_output() {
    let s = Sandbox::new();
    let (out, _) = run(&s, &["install", "claude"]);
    let root = s.root().display().to_string();
    insta::assert_snapshot!(out.replace(&root, "<sandbox>"));
}

#[test]
fn ts6_7_install_json_lists_linked_paths() {
    let s = Sandbox::new();
    let (out, err) = run(&s, &["--json", "install", "claude"]);
    let root = s.root().display().to_string();
    assert_eq!(out.lines().count(), 1, "{out}");
    insta::assert_snapshot!(out.replace(&root, "<sandbox>"));
    assert!(err.lines().all(|l| l.starts_with("warning: ")), "{err}");
    assert!(
        err.lines().any(|l| l == format!("warning: {RELOAD}")),
        "{err}"
    );
    assert!(err.contains("Bash(brain-swap:*)"), "{err}");
}

#[test]
fn fr67_install_never_edits_claude_settings() {
    let s = Sandbox::new();
    let claude = s.root().join("home/.claude");
    let files = [
        ("settings.json", "{\"permissions\":{\"allow\":[]}}\n"),
        ("settings.local.json", "{\"hooks\":{}}\n"),
        ("skills/other/SKILL.md", "---\nname: other\n---\nother\n"),
    ];
    for (path, text) in files {
        let p = claude.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let before = tree(&claude);
    run(&s, &["install", "claude"]);
    for (path, text) in files {
        assert_eq!(
            fs::read_to_string(claude.join(path)).unwrap(),
            text,
            "{path}"
        );
    }
    let mut expected = before;
    for skill in &SKILLS {
        let dir = Path::new("skills").join(skill.name);
        expected.push(dir.join("SKILL.md"));
        expected.push(dir);
    }
    expected.sort();
    assert_eq!(tree(&claude), expected);
}

#[test]
fn ts8_8_install_leaves_no_temp_entries() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    let names = tree(&skills(&s));
    assert!(
        names
            .iter()
            .all(|p| !p.to_string_lossy().contains(".bs-tmp-")),
        "{names:?}"
    );
}
