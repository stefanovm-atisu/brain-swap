//! TECHSPEC 8.1, 8.5, 12.1 Pack: static checks of the skill files; nothing is spawned.

mod common;

use brain_swap::cli::args::{Cli, Command};
use clap::Parser;
use common::skill_md::*;
use std::collections::BTreeMap;

#[test]
fn ts8_1_bs_link_frontmatter_names_and_version() {
    let fm = frontmatter(&read_skill("bs-link"));
    assert_eq!(fm["name"], Value::Text("bs-link".into()));
    assert!(matches!(&fm["description"], Value::Text(d) if !d.is_empty()));
    assert_eq!(
        fm["argument-hint"],
        Value::Text("<card ID, e.g. W-12>".into())
    );
    assert_eq!(
        fm["metadata"],
        Value::Map(BTreeMap::from([
            ("pack".into(), "brain-swap".into()),
            ("version".into(), env!("CARGO_PKG_VERSION").into()),
        ]))
    );
}

#[test]
fn fr46_bs_link_user_only_and_cli_only() {
    let fm = frontmatter(&read_skill("bs-link"));
    assert_eq!(fm["disable-model-invocation"], Value::Text("true".into()));
    assert_eq!(
        fm["allowed-tools"],
        Value::Text("Bash(brain-swap:*)".into())
    );
}

#[test]
fn ts8_5_bs_link_has_no_inline_command() {
    assert_eq!(
        inline_commands(&read_skill("bs-link")),
        Vec::<String>::new()
    );
}

#[test]
fn ts12_1_bs_link_commands_parse() {
    let mut cmds = Vec::new();
    for c in commands(&read_skill("bs-link")) {
        if c.starts_with("brain-swap ls ") {
            cmds.push(c.clone());
            cmds.push(format!("{c} --board home"));
        } else {
            cmds.push(c);
        }
    }
    let found: Vec<String> = cmds
        .iter()
        .map(|c| {
            let cli = Cli::try_parse_from(shell_words(&substitute(c, SESSION_A)))
                .unwrap_or_else(|e| panic!("{c}: {e}"));
            match (cli.command, cli.board.as_deref()) {
                (Some(Command::Link { .. }), None) => "link",
                (Some(Command::Ls { .. }), None) => "ls",
                (Some(Command::Ls { .. }), Some("home")) => "ls --board home",
                (Some(Command::Move { .. }), None) => "move",
                (other, board) => panic!("{c}: unexpected {other:?} {board:?}"),
            }
            .to_string()
        })
        .collect();
    assert_eq!(found, ["link", "ls", "ls --board home", "move"]);
}

#[test]
fn ts8_1_bs_link_session_passed_quoted() {
    let with_session: Vec<String> = commands(&read_skill("bs-link"))
        .into_iter()
        .filter(|c| c.contains("--session"))
        .collect();
    assert!(!with_session.is_empty());
    for c in with_session {
        let words: Vec<&str> = c.split_whitespace().collect();
        let i = words.iter().position(|w| *w == "--session").unwrap();
        assert_eq!(words.get(i + 1), Some(&"\"${CLAUDE_SESSION_ID}\""), "{c}");
    }
}

#[test]
fn fr47_bs_link_ends_with_move_line() {
    let text = read_skill("bs-link");
    let last = text.lines().rev().find(|l| !l.trim().is_empty());
    assert_eq!(
        last,
        Some(
            "If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line."
        )
    );
}
