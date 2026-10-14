//! TECHSPEC 13 and 12.2 item 5: the README's install, pack, herdr key and upgrade
//! sections (FR-64, FR-67, FR-68).

mod common;

use brain_swap::cli::args::Cli;
use clap::Parser;
use common::cli::Sandbox;
use common::skill_md::{SESSION_A, commands, shell_words, substitute};

const README: &str = include_str!("../README.md");

/// The text from the line `heading` to the next `## ` heading.
pub fn readme_section(text: &str, heading: &str) -> Option<String> {
    let rest = text.split(&format!("\n{heading}\n")).nth(1)?;
    Some(rest.split("\n## ").next().unwrap().to_string())
}

fn section(heading: &str) -> String {
    readme_section(README, heading).unwrap_or_else(|| panic!("README has no `{heading}`"))
}

/// Index of the first fenced line starting with each of the three install steps.
fn steps(section: &str) -> Vec<Option<usize>> {
    let fenced: Vec<&str> = section
        .split("```")
        .skip(1)
        .step_by(2)
        .flat_map(|b| b.lines().skip(1))
        .map(str::trim)
        .collect();
    [
        "cargo install",
        "brain-swap install claude",
        "brain-swap install herdr",
    ]
    .iter()
    .map(|p| fenced.iter().position(|l| l.starts_with(p)))
    .collect()
}

fn assert_three_steps(section: &str) {
    let found = steps(section);
    assert!(found.iter().all(Option::is_some), "{found:?}");
    assert!(found[0] < found[1] && found[1] < found[2], "{found:?}");
}

/// The value of the first `key = "..."` line of `text`.
fn key(text: &str) -> String {
    let line = text.lines().find(|l| l.starts_with("key = ")).unwrap();
    line["key = ".len()..].trim_matches('"').to_string()
}

#[test]
fn fr68_readme_install_lists_three_steps_in_order() {
    let install = section("## Install");
    assert_three_steps(&install);
    assert!(install.contains("herdr server reload-config"));
}

#[test]
fn ts13_readme_upgrade_repeats_the_three_steps() {
    assert_three_steps(&section("## Upgrade"));
}

#[test]
fn ts13_readme_brain_swap_commands_parse() {
    let cmds = commands(README);
    assert!(!cmds.is_empty());
    for c in cmds {
        Cli::try_parse_from(shell_words(&substitute(&c, SESSION_A)))
            .unwrap_or_else(|e| panic!("{c}: {e}"));
    }
}

#[test]
fn fr64_readme_herdr_key_matches_install_herdr() {
    let s = Sandbox::new();
    let out = s
        .cmd(&["install", "herdr"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let printed = key(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(printed, key(&section("## herdr key binding")));
}

#[test]
fn fr67_readme_pack_section_names_commands_and_hints() {
    let pack = section("## Claude Code pack");
    for word in [
        "/bs-card",
        "/bs-park",
        "/bs-back",
        "/bs-link",
        "--link",
        "--force",
        "--remove",
        "/reload-skills",
        "Bash(/home/you/.cargo/bin/brain-swap:*)",
    ] {
        assert!(pack.contains(word), "pack section lacks {word}");
    }
}

fn usage() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/usage.md"))
        .expect("docs/usage.md")
}

#[test]
fn ts12_2_usage_guide_names_every_pack_command() {
    let guide = usage();
    for word in [
        "/bs-card",
        "/bs-park",
        "/bs-back",
        "/bs-link",
        "/bs-card feature: ",
        "/bs-park next: ",
    ] {
        assert!(guide.contains(word), "usage guide lacks {word}");
    }
}

#[test]
fn ts12_2_usage_guide_commands_parse() {
    let cmds = commands(&usage());
    assert!(!cmds.is_empty());
    for c in cmds {
        Cli::try_parse_from(shell_words(&substitute(&c, SESSION_A)))
            .unwrap_or_else(|e| panic!("{c}: {e}"));
    }
}

#[test]
fn ts12_2_readme_cheat_sheet_links_usage_guide() {
    let sheet = section("## Cheat sheet");
    for word in [
        "/bs-card",
        "/bs-park",
        "/bs-back",
        "/bs-link",
        "docs/usage.md",
    ] {
        assert!(sheet.contains(word), "cheat sheet lacks {word}");
    }
    let lines = sheet.lines().filter(|l| !l.trim().is_empty()).count();
    assert!(lines <= 25, "cheat sheet has {lines} non-empty lines");
}
