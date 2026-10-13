//! TECHSPEC 6.3 and 6.7 `install herdr`, 9.4: the key binding snippet (FR-64).

mod common;

use brain_swap::adapters::herdr::key_snippet;
use common::cli::Sandbox;
use common::spawn::command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn canonical() -> String {
    fs::canonicalize(env!("CARGO_BIN_EXE_brain-swap"))
        .unwrap()
        .display()
        .to_string()
}

fn stdout(s: &Sandbox, args: &[&str]) -> String {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    String::from_utf8(out.stdout).unwrap()
}

/// Every file under `dir`, sorted.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(files(&p));
        } else {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn fr64_install_herdr_prints_the_snippet() {
    let s = Sandbox::new();
    let text = stdout(&s, &["install", "herdr"]).replace(&canonical(), "<brain-swap>");
    insta::assert_snapshot!("install_herdr", s.redact(&text));
}

#[test]
fn fr64_snippet_names_the_canonical_absolute_path() {
    let mut s = Sandbox::new();
    let link = s.root().join("bin/bs");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_brain-swap"), &link).unwrap();
    let out = command(s.scenario_mut(), &link)
        .args(["install", "herdr"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let line = text.lines().find(|l| l.starts_with("command = ")).unwrap();
    assert!(line.contains(&canonical()), "{line}");
    assert!(!line.contains(&link.display().to_string()), "{line}");
}

#[test]
fn fr64_install_herdr_never_edits_herdr_config() {
    let s = Sandbox::new();
    let herdr = s.root().join("config/herdr/config.toml");
    fs::create_dir_all(herdr.parent().unwrap()).unwrap();
    fs::write(&herdr, "[keys]\n").unwrap();
    let before = files(s.root());
    stdout(&s, &["install", "herdr"]);
    assert_eq!(fs::read(&herdr).unwrap(), b"[keys]\n");
    assert_eq!(files(s.root()), before);
}

#[test]
fn ts6_7_install_herdr_json_returns_the_snippet() {
    let s = Sandbox::new();
    let text = stdout(&s, &["--json", "install", "herdr"]);
    assert_eq!(text.lines().count(), 1, "{text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["v"], 1);
    assert_eq!(v["snippet"], key_snippet(&canonical()));
    assert!(!text.contains("reload-config"), "{text}");
}

#[test]
fn ts6_3_install_herdr_ignores_a_broken_config() {
    let s = Sandbox::new();
    let config = s.config_file();
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "default_board = \n").unwrap();
    stdout(&s, &["install", "herdr"]);
    assert_eq!(fs::read(&config).unwrap(), b"default_board = \n");
}
