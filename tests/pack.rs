//! TECHSPEC 8.1, 8.5, 12.1 Pack: static checks of the skill files; nothing is spawned.

mod common;

use brain_swap::cli::args::{Cli, Command};
use clap::Parser;
use common::pack_env::PackEnv;
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

/// The first bs-link command starting with `prefix` (step 1 is `link`, step 2 is `ls`).
fn bs_link_command(prefix: &str) -> String {
    commands(&read_skill("bs-link"))
        .into_iter()
        .find(|c| c.starts_with(prefix))
        .unwrap()
}

#[test]
fn fr54_bs_link_command_links_and_prints_latest_note() {
    let env = PackEnv::new().with_w12();
    let cmd = substitute(&bs_link_command("brain-swap link "), SESSION_A);
    let (code, out, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "linked to W-12: migrate invoices to v13");
    assert_eq!(lines[1], "card: W-12 migrate invoices to v13");
    assert!(lines.iter().any(|l| l.starts_with("latest note:")), "{out}");
    for l in [
        "Doing: invoice reads switched to v13",
        "Next: run the migration test on staging",
        "Watch out: v13 needs a 10 s timeout",
    ] {
        assert!(lines.contains(&l), "{l}: {out}");
    }
    let (_, ctx, _) = env.run_sh(&format!("brain-swap context --session {SESSION_A}"), "");
    assert!(ctx.lines().any(|l| l == "reference: W-12"), "{ctx}");
}

#[test]
fn fr54_bs_link_unknown_id_prints_one_error_line_and_changes_nothing() {
    let env = PackEnv::new().with_w12();
    let cmd = substitute(&bs_link_command("brain-swap link "), SESSION_A).replace("W-12", "W-99");
    let (code, out, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 3, "{err}");
    assert_eq!(out, "");
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(err.starts_with("brain-swap: "), "{err}");
    let (_, ctx, _) = env.run_sh(&format!("brain-swap context --session {SESSION_A}"), "");
    assert!(ctx.lines().any(|l| l == "reference: none"), "{ctx}");
}

#[test]
fn fr55_bs_link_without_id_lists_open_cards_by_best_guess() {
    let env = PackEnv::new().with_w12();
    let (code, out, err) = env.run_sh(&bs_link_command("brain-swap ls "), "");
    assert_eq!(code, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 5, "{out}");
    assert_eq!(lines[0], "open cards on work, best guess first:");
    for (line, start) in lines[1..4].iter().zip([
        "1. W-12 migrate invoices to v13 [Doing]",
        "2. W-7 fix rounding in act export [Doing]",
        "3. W-11 invoice PDF shows the wrong VAT [Todo] no note",
    ]) {
        assert!(line.starts_with(start), "{start}: {out}");
    }
    assert_eq!(lines[4], "o. other board: home, personal");
}

#[test]
fn fr55_bs_link_other_board_lists_that_board() {
    let env = PackEnv::new().with_w12();
    let cmd = format!("{} --board home", bs_link_command("brain-swap ls "));
    let (code, out, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "open cards on home, best guess first:");
    assert!(
        lines.contains(&"1. H-3 renew car insurance [Todo] no note"),
        "{out}"
    );
    assert!(
        lines.last().unwrap().starts_with("o. other board:"),
        "{out}"
    );
}
