//! TECHSPEC 8.1, 8.5, 12.1 Pack: static checks of the skill files; nothing is spawned.

mod common;

use brain_swap::cli::args::{Cli, Command};
use clap::Parser;
use common::pack_env::{ContextState, PackEnv};
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

#[test]
fn ts8_1_bs_park_frontmatter_names_and_version() {
    let fm = frontmatter(&read_skill("bs-park"));
    assert_eq!(fm["name"], Value::Text("bs-park".into()));
    assert!(matches!(&fm["description"], Value::Text(d) if !d.is_empty()));
    assert_eq!(
        fm["argument-hint"],
        Value::Text("[your own words, e.g. next: rerun the test, watch: timeout]".into())
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
fn fr46_bs_park_user_only_and_cli_only() {
    let fm = frontmatter(&read_skill("bs-park"));
    assert_eq!(fm["disable-model-invocation"], Value::Text("true".into()));
    assert_eq!(
        fm["allowed-tools"],
        Value::Text("Bash(brain-swap:*)".into())
    );
}

const PARK_INLINE: &str = r#"brain-swap context --session "${CLAUDE_SESSION_ID}""#;

#[test]
fn ts8_1_bs_park_inline_command_is_context_only() {
    let inline = inline_commands(&read_skill("bs-park"));
    assert_eq!(inline, [PARK_INLINE]);
    assert!(!inline[0].contains("$ARGUMENTS"));
}

#[test]
fn ts12_1_bs_park_commands_parse() {
    let found: Vec<String> = commands(&read_skill("bs-park"))
        .iter()
        .map(|c| {
            let cli = Cli::try_parse_from(shell_words(&substitute(c, SESSION_A)))
                .unwrap_or_else(|e| panic!("{c}: {e}"));
            match (cli.command, cli.board.as_deref()) {
                (Some(Command::Context { .. }), None) => "context".to_string(),
                (
                    Some(Command::Ls {
                        open: true,
                        guess: true,
                    }),
                    Some("home"),
                ) => "ls --open --guess --board home".to_string(),
                (
                    Some(Command::Park {
                        card: Some(id),
                        session: Some(s),
                        auto: true,
                    }),
                    None,
                ) => {
                    format!("park --card {id} --session {s} --auto")
                }
                (Some(Command::Move { .. }), None) => "move".to_string(),
                (other, board) => panic!("{c}: unexpected {other:?} {board:?}"),
            }
        })
        .collect();
    assert_eq!(
        found,
        [
            "context".to_string(),
            "ls --open --guess --board home".to_string(),
            format!("park --card W-12 --session {SESSION_A} --auto"),
            "move".to_string(),
        ]
    );
}

#[test]
fn ts8_1_bs_park_session_passed_quoted_and_card_named() {
    let cmds = commands(&read_skill("bs-park"));
    assert!(cmds.iter().any(|c| c.starts_with("brain-swap park ")));
    for c in cmds {
        let words: Vec<&str> = c.split_whitespace().collect();
        if let Some(i) = words.iter().position(|w| *w == "--session") {
            assert_eq!(words.get(i + 1), Some(&"\"${CLAUDE_SESSION_ID}\""), "{c}");
        }
        if c.starts_with("brain-swap park ") {
            let i = words.iter().position(|w| *w == "--card").expect(&c);
            assert_eq!(words.get(i + 1), Some(&"<ID>"), "{c}");
        }
    }
}

#[test]
fn fr47_bs_park_ends_with_move_line() {
    let text = read_skill("bs-park");
    let last = text.lines().rev().find(|l| !l.trim().is_empty());
    assert_eq!(
        last,
        Some(
            "If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line."
        )
    );
}

/// The bs-park inline command run in `state`: exit code, stdout, stderr.
fn park_inline(state: ContextState) -> (i32, String, String) {
    let (env, session) = PackEnv::for_state(state);
    env.run_sh(&substitute(PARK_INLINE, session), "")
}

#[test]
fn ts12_1_bs_park_inline_context_exits_0_in_every_state() {
    use ContextState::*;
    for state in [
        NoConfig,
        BrokenConfig,
        MissingFolder,
        NoReference,
        DeletedCard,
        EmptySession,
        Unsubstituted,
        Placeholder,
        WithNote,
        WithoutNote,
    ] {
        let (code, out, err) = park_inline(state);
        assert_eq!(code, 0, "{state:?}: {err}");
        assert_eq!(
            out.lines().next(),
            Some("brain-swap context v1"),
            "{state:?}: {out}"
        );
    }
}

#[test]
fn ts8_3_bs_park_inline_context_error_line_has_hint() {
    let (_, out, _) = park_inline(ContextState::BrokenConfig);
    let lines: Vec<&str> = out.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.starts_with("error: config "))
        .unwrap_or_else(|| panic!("{out}"));
    let place = lines[i]
        .split("/config/brain-swap/config.toml:")
        .nth(1)
        .unwrap_or_else(|| panic!("{out}"));
    let mut parts = place.splitn(3, ':');
    assert_eq!(parts.next(), Some("2"), "{out}");
    assert!(
        parts.next().unwrap().parse::<u32>().is_ok(),
        "no column: {out}"
    );
    assert!(lines[i + 1].starts_with("hint:"), "{out}");
}

#[test]
fn fr56_bs_park_no_reference_context_prints_picker() {
    let (_, out, _) = park_inline(ContextState::NoReference);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"reference: none"), "{out}");
    let i = lines
        .iter()
        .position(|l| *l == "open cards on work, best guess first:")
        .unwrap_or_else(|| panic!("{out}"));
    for (line, start) in lines[i + 1..].iter().zip(["1. W-12", "2. W-7", "3. W-11"]) {
        assert!(line.starts_with(start), "{start}: {out}");
    }
    assert_eq!(lines.last(), Some(&"o. other board: home, personal"));
}

/// Step 4's heredoc of bs-park for `SESSION_A`, filled with the given parts.
fn park_heredoc(doing: &str, next: &str, watch_out: &str) -> String {
    let blocks = heredoc_commands(&read_skill("bs-park"));
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    fill_park_heredoc(&substitute(&blocks[0], SESSION_A), doing, next, watch_out)
}

/// The notes of W-12: each `### ` heading with its `- ` part lines.
fn w12_notes(env: &PackEnv) -> Vec<(String, Vec<String>)> {
    let (_, text, _) = env.run_sh("cat boards/work/W-12.md", "");
    let mut notes: Vec<(String, Vec<String>)> = Vec::new();
    for l in text.lines() {
        if l.starts_with("### ") {
            notes.push((l.to_string(), Vec::new()));
        } else if let (Some(n), true) = (notes.last_mut(), l.starts_with("- ")) {
            n.1.push(l.to_string());
        }
    }
    notes
}

#[test]
fn fr49_bs_park_heredoc_parks_to_referenced_card() {
    let (env, _) = PackEnv::for_state(ContextState::WithNote);
    let cmd = park_heredoc("switched repo to v13", "rerun migration test", "timeout");
    let (code, out, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "parked to W-12: migrate invoices to v13\n");
    let notes = w12_notes(&env);
    assert_eq!(notes.len(), 2, "{notes:?}");
    let (heading, parts) = notes.last().unwrap();
    assert!(heading.ends_with(" auto"), "{heading}");
    assert_eq!(
        parts,
        &[
            "- Doing: switched repo to v13",
            "- Next: rerun migration test",
            "- Watch out: timeout",
        ]
    );
    let (_, text, _) = env.run_sh("cat boards/work/W-12.md", "");
    assert!(text.lines().any(|l| l == "column: Doing"), "{text}");
}

#[test]
fn fr50_bs_park_heredoc_without_auto_writes_plain_note() {
    let (env, _) = PackEnv::for_state(ContextState::WithNote);
    let cmd = park_heredoc("switched repo to v13", "rerun migration test", "timeout")
        .replace(" --auto", "");
    let (code, _, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    let notes = w12_notes(&env);
    let (heading, _) = notes.last().unwrap();
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(!heading.ends_with(" auto"), "{heading}");
}

#[test]
fn ts8_6_bs_park_heredoc_keeps_text_literal() {
    let (env, _) = PackEnv::for_state(ContextState::WithNote);
    let doing = r#"cost is $HOME and `date` "quoted""#;
    let cmd = park_heredoc(doing, "rerun migration test", "timeout");
    let (code, _, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    let notes = w12_notes(&env);
    assert_eq!(notes.last().unwrap().1[0], format!("- Doing: {doing}"));
}

// bs-back (TECHSPEC 8.4)

#[test]
fn ts8_1_bs_back_frontmatter_names_and_version() {
    let fm = frontmatter(&read_skill("bs-back"));
    assert_eq!(fm["name"], Value::Text("bs-back".into()));
    assert!(matches!(&fm["description"], Value::Text(d) if !d.is_empty()));
    assert!(!fm.contains_key("argument-hint"));
    assert_eq!(
        fm["metadata"],
        Value::Map(BTreeMap::from([
            ("pack".into(), "brain-swap".into()),
            ("version".into(), env!("CARGO_PKG_VERSION").into()),
        ]))
    );
}

#[test]
fn fr46_bs_back_user_only_and_cli_only() {
    let fm = frontmatter(&read_skill("bs-back"));
    assert_eq!(fm["disable-model-invocation"], Value::Text("true".into()));
    assert_eq!(
        fm["allowed-tools"],
        Value::Text("Bash(brain-swap:*)".into())
    );
}

#[test]
fn ts8_1_bs_back_inline_command_is_context_only() {
    assert_eq!(inline_commands(&read_skill("bs-back")), [PARK_INLINE]);
}

#[test]
fn ts12_1_bs_back_commands_parse() {
    let found: Vec<String> = commands(&read_skill("bs-back"))
        .iter()
        .map(|c| {
            let cli = Cli::try_parse_from(shell_words(&substitute(c, SESSION_A)))
                .unwrap_or_else(|e| panic!("{c}: {e}"));
            match (cli.command, cli.board.as_deref()) {
                (Some(Command::Context { .. }), None) => "context".to_string(),
                (
                    Some(Command::Ls {
                        open: true,
                        guess: true,
                    }),
                    Some("home"),
                ) => "ls --open --guess --board home".to_string(),
                (
                    Some(Command::Link {
                        id,
                        session: Some(s),
                    }),
                    None,
                ) => format!("link {id} --session {s}"),
                (Some(Command::Show { id, all: false }), None) => format!("show {id}"),
                (
                    Some(Command::Park {
                        card: Some(id),
                        session: Some(s),
                        auto: true,
                    }),
                    None,
                ) => format!("park --card {id} --session {s} --auto"),
                (Some(Command::Move { .. }), None) => "move".to_string(),
                (other, board) => panic!("{c}: unexpected {other:?} {board:?}"),
            }
        })
        .collect();
    assert_eq!(
        found,
        [
            "context".to_string(),
            "ls --open --guess --board home".to_string(),
            format!("link W-12 --session {SESSION_A}"),
            "show W-12".to_string(),
            format!("park --card W-12 --session {SESSION_A} --auto"),
            "move".to_string(),
        ]
    );
}

#[test]
fn ts8_1_bs_back_session_quoted_card_named_move_line_last() {
    let text = read_skill("bs-back");
    let cmds = commands(&text);
    assert!(cmds.iter().any(|c| c.starts_with("brain-swap park ")));
    for c in cmds {
        let words: Vec<&str> = c.split_whitespace().collect();
        if let Some(i) = words.iter().position(|w| *w == "--session") {
            assert_eq!(words.get(i + 1), Some(&"\"${CLAUDE_SESSION_ID}\""), "{c}");
        }
        if c.starts_with("brain-swap park ") {
            let i = words.iter().position(|w| *w == "--card").expect(&c);
            assert_eq!(words.get(i + 1), Some(&"<ID>"), "{c}");
        }
    }
    let last = text.lines().rev().find(|l| !l.trim().is_empty());
    assert_eq!(
        last,
        Some(
            "If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line."
        )
    );
}

/// The bs-back inline command for `session`, run in `env`: exit code, stdout, stderr.
fn back_inline(env: &PackEnv, session: &str) -> (i32, String, String) {
    let inline = inline_commands(&read_skill("bs-back"));
    env.run_sh(&substitute(&inline[0], session), "")
}

/// The first bs-back command starting with `prefix`, substituted for `session`.
fn bs_back_command(prefix: &str, session: &str) -> String {
    let cmd = commands(&read_skill("bs-back"))
        .into_iter()
        .find(|c| c.starts_with(prefix))
        .unwrap();
    substitute(&cmd, session)
}

/// The `latest note:` line of `out`.
fn latest_note_line(out: &str) -> &str {
    out.lines()
        .find(|l| l.starts_with("latest note:"))
        .unwrap_or_else(|| panic!("{out}"))
}

#[test]
fn ts12_1_bs_back_inline_context_exits_0_in_every_state() {
    use ContextState::*;
    for state in [
        NoConfig,
        BrokenConfig,
        MissingFolder,
        NoReference,
        DeletedCard,
        EmptySession,
        Unsubstituted,
        Placeholder,
        WithNote,
        WithoutNote,
    ] {
        let (env, session) = PackEnv::for_state(state);
        let (code, out, err) = back_inline(&env, session);
        assert_eq!(code, 0, "{state:?}: {err}");
        assert_eq!(
            out.lines().next(),
            Some("brain-swap context v1"),
            "{state:?}: {out}"
        );
    }
}

#[test]
fn fr51_bs_back_context_carries_the_printed_fields() {
    let (env, session) = PackEnv::for_state(ContextState::WithNote);
    let (_, out, _) = back_inline(&env, session);
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines.contains(&"card: W-12 migrate invoices to v13"),
        "{out}"
    );
    let note = latest_note_line(&out);
    assert!(note.contains("12 min ago"), "{note}");
    assert!(note.ends_with("by this session"), "{note}");
    for l in [
        "Doing: invoice reads switched to v13",
        "Next: run the migration test on staging",
        "Watch out: v13 needs a 10 s timeout",
    ] {
        assert!(lines.contains(&l), "{l}: {out}");
    }
}

#[test]
fn fr52_bs_back_context_without_note_reads_none() {
    let (env, session) = PackEnv::for_state(ContextState::WithoutNote);
    let (_, out, _) = back_inline(&env, session);
    assert!(out.lines().any(|l| l == "latest note: none"), "{out}");
}

#[test]
fn fr53_bs_back_context_names_the_note_author() {
    let env = PackEnv::new().with_w12();
    env.link("W-12", SESSION_A);
    env.link("W-12", SESSION_B);
    env.link("W-7", SESSION_C);
    for (session, end) in [
        (SESSION_A, "by this session"),
        (SESSION_B, "by another session"),
        (SESSION_C, "no session"),
    ] {
        let (_, out, _) = back_inline(&env, session);
        let note = latest_note_line(&out);
        assert!(note.ends_with(end), "{session}: {note}");
    }
}

#[test]
fn fr56_bs_back_link_after_pick_sets_reference() {
    let env = PackEnv::new().with_w12();
    let (code, out, err) = env.run_sh(&bs_back_command("brain-swap link ", SESSION_C), "");
    assert_eq!(code, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[..2],
        [
            "linked to W-12: migrate invoices to v13",
            "card: W-12 migrate invoices to v13",
        ],
        "{out}"
    );
    let (_, ctx, _) = back_inline(&env, SESSION_C);
    assert!(ctx.lines().any(|l| l == "reference: W-12"), "{ctx}");
}

#[test]
fn ts8_4_bs_back_show_fallback_when_no_session_resolves() {
    let (env, session) = PackEnv::for_state(ContextState::Unsubstituted);
    let (code, _, err) = env.run_sh(&bs_back_command("brain-swap link ", session), "");
    assert_eq!(code, 1, "{err}");
    let (code, out, err) = env.run_sh(&bs_back_command("brain-swap show ", session), "");
    assert_eq!(code, 0, "{err}");
    assert!(
        out.starts_with("card: W-12 migrate invoices to v13\n"),
        "{out}"
    );
}

#[test]
fn fr52_bs_back_catch_up_heredoc_saves_auto_note() {
    let (env, _) = PackEnv::for_state(ContextState::WithNote);
    let blocks = heredoc_commands(&read_skill("bs-back"));
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    let cmd = fill_park_heredoc(
        &substitute(&blocks[0], SESSION_A),
        "switched repo to v13",
        "rerun migration test",
        "timeout",
    );
    let (code, out, err) = env.run_sh(&cmd, "");
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "parked to W-12: migrate invoices to v13\n");
    let notes = w12_notes(&env);
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes.last().unwrap().0.ends_with(" auto"), "{notes:?}");
}

#[test]
fn fr51_bs_back_names_the_just_now_form() {
    let text = read_skill("bs-back");
    assert!(text.contains("an age of `now` gives `(note just now)`"));
    assert!(text.contains("an age of `now` gives `no note since just now on <ID>`"));
}

#[test]
fn fr52_bs_back_first_line_is_verbatim() {
    let text = read_skill("bs-back");
    assert!(text.contains("Write the first line exactly in the given form, never paraphrased"));
}

#[test]
fn fr51_bs_back_prints_plain_lines() {
    let text = read_skill("bs-back");
    assert!(text.contains("as plain text and not in a code block"));
}

// ---- bs-card (E3-F1-T5) ----

#[test]
fn ts8_1_bs_card_frontmatter_names_and_version() {
    let fm = frontmatter(&read_skill("bs-card"));
    assert_eq!(fm["name"], Value::Text("bs-card".into()));
    assert!(matches!(&fm["description"], Value::Text(d) if !d.is_empty()));
    assert_eq!(
        fm["argument-hint"],
        Value::Text("[<template>:] <task in a few words> [on home|personal]".into())
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
fn fr46_bs_card_user_only_and_cli_only() {
    let fm = frontmatter(&read_skill("bs-card"));
    assert_eq!(fm["disable-model-invocation"], Value::Text("true".into()));
    assert_eq!(
        fm["allowed-tools"],
        Value::Text("Bash(brain-swap:*)".into())
    );
}

#[test]
fn ts8_1_bs_card_inline_command_is_templates_only() {
    let inline = inline_commands(&read_skill("bs-card"));
    assert_eq!(inline, ["brain-swap templates"]);
    assert!(!inline[0].contains("$ARGUMENTS"));
}

#[test]
fn ts12_1_bs_card_commands_parse() {
    let mut cmds = Vec::new();
    for c in commands(&read_skill("bs-card")) {
        if c.starts_with("brain-swap new ") {
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
            let board = cli
                .board
                .map(|b| format!(" --board {b}"))
                .unwrap_or_default();
            match cli.command {
                Some(Command::Templates) if board.is_empty() => "templates".to_string(),
                Some(Command::New {
                    template: Some(t),
                    column: None,
                    session: Some(s),
                    body: Some(b),
                    title,
                }) if title.is_empty() => {
                    format!("new --template {t} --session {s} --body {b}{board}")
                }
                Some(Command::Move { .. }) if board.is_empty() => "move".to_string(),
                other => panic!("{c}: unexpected {other:?}{board}"),
            }
        })
        .collect();
    let new = format!("new --template Feature --session {SESSION_A} --body -");
    assert_eq!(
        found,
        [
            "templates".to_string(),
            new.clone(),
            format!("{new} --board home"),
            "move".to_string(),
        ]
    );
}

#[test]
fn ts8_1_bs_card_session_quoted_and_move_line_last() {
    let text = read_skill("bs-card");
    let with_session: Vec<String> = commands(&text)
        .into_iter()
        .filter(|c| c.contains("--session"))
        .collect();
    assert!(!with_session.is_empty());
    for c in with_session {
        let words: Vec<&str> = c.split_whitespace().collect();
        let i = words.iter().position(|w| *w == "--session").unwrap();
        assert_eq!(words.get(i + 1), Some(&"\"${CLAUDE_SESSION_ID}\""), "{c}");
    }
    let last = text.lines().rev().find(|l| !l.trim().is_empty());
    assert_eq!(
        last,
        Some(
            "If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line."
        )
    );
}

#[test]
fn ts12_1_bs_card_inline_templates_exits_0_in_every_state() {
    use ContextState::*;
    for state in [
        NoConfig,
        BrokenConfig,
        MissingFolder,
        NoReference,
        DeletedCard,
        EmptySession,
        Unsubstituted,
        Placeholder,
        WithNote,
        WithoutNote,
    ] {
        let (env, session) = PackEnv::for_state(state);
        let inline = inline_commands(&read_skill("bs-card"));
        let (code, out, err) = env.run_sh(&substitute(&inline[0], session), "");
        assert_eq!(code, 0, "{state:?}: {err}");
        assert!(
            out.lines().any(|l| l == "template: Feature (key f)"),
            "{state:?}: {out}"
        );
    }
}

const CARD_TITLE: &str = "migrate invoices to v13";
const CARD_GOAL: &str = "Switch invoice reads and writes to the v13 procedures.";
const CARD_CONTEXT: &str = r#"run $PATH check and `make` "now""#;

/// The bs-card heredoc for `SESSION_A`, filled with `CARD_TITLE` and the sample Feature body.
fn card_heredoc() -> String {
    let blocks = heredoc_commands(&read_skill("bs-card"));
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    let body = format!("## Goal\n{CARD_GOAL}\n\n## Acceptance\n\n## Context\n{CARD_CONTEXT}");
    fill_card_heredoc(&substitute(&blocks[0], SESSION_A), CARD_TITLE, &body)
}

/// Runs the filled bs-card heredoc in `env`, expecting exit 0; returns stdout.
fn create_card(env: &PackEnv, cmd: &str) -> String {
    let (code, out, err) = env.run_sh(cmd, "");
    assert_eq!(code, 0, "{err}");
    out
}

#[test]
fn fr48_bs_card_heredoc_creates_card_and_links_session() {
    let env = PackEnv::new();
    let out = create_card(&env, &card_heredoc());
    assert_eq!(out, format!("created W-12: {CARD_TITLE} (Todo, work)\n"));
    let (_, text, _) = env.run_sh("cat boards/work/W-12.md", "");
    let lines: Vec<&str> = text.lines().collect();
    for l in [
        "column: Todo",
        "template: Feature",
        "# migrate invoices to v13",
    ] {
        assert!(lines.contains(&l), "{l}: {text}");
    }
    let goal = lines.iter().position(|l| *l == "## Goal").expect(&text);
    assert_eq!(
        lines[goal + 1..].iter().find(|l| !l.trim().is_empty()),
        Some(&CARD_GOAL),
        "{text}"
    );
    assert_eq!(
        lines.iter().filter(|l| **l == "## Timeline").count(),
        1,
        "{text}"
    );
    let (_, ctx, _) = env.run_sh(&format!("brain-swap context --session {SESSION_A}"), "");
    assert!(ctx.lines().any(|l| l == "reference: W-12"), "{ctx}");
}

#[test]
fn fr48_bs_card_named_board_creates_on_that_board() {
    let env = PackEnv::new();
    let (_, before, _) = env.run_sh("ls boards/work", "");
    let cmd = card_heredoc().replacen(" <<'BS_EOF'", " --board home <<'BS_EOF'", 1);
    let out = create_card(&env, &cmd);
    assert_eq!(out, format!("created H-4: {CARD_TITLE} (Todo, home)\n"));
    let (_, after, _) = env.run_sh("ls boards/work", "");
    assert_eq!(after, before);
}

#[test]
fn ts8_6_bs_card_heredoc_keeps_body_literal() {
    let env = PackEnv::new();
    create_card(&env, &card_heredoc());
    let (_, text, _) = env.run_sh("cat boards/work/W-12.md", "");
    let context = text.lines().position(|l| l == "## Context").expect(&text);
    assert_eq!(text.lines().nth(context + 1), Some(CARD_CONTEXT), "{text}");
}

#[test]
fn fr47_move_line_command_moves_the_card() {
    let env = PackEnv::new();
    create_card(&env, &card_heredoc());
    let (_, before, _) = env.run_sh("cat boards/work/W-12.md", "");
    let mv = commands(&read_skill("bs-card"))
        .into_iter()
        .find(|c| c.starts_with("brain-swap move "))
        .unwrap();
    let mv = substitute(&mv, SESSION_A);
    assert_eq!(mv, "brain-swap move W-12 Doing");
    let (code, out, err) = env.run_sh(&mv, "");
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "moved W-12 to Doing\n");
    let (_, after, _) = env.run_sh("cat boards/work/W-12.md", "");
    assert_eq!(
        after,
        before.replacen("column: Todo\n", "column: Doing\n", 1)
    );
    assert_ne!(after, before);
}

// ---- AS-1, AS-2, AS-3 command replay (E3-F1-T7) ----

const P9: (&str, &str, &str) = ("w4V:p9", "w4V:t1", "w4V");
const P11: (&str, &str, &str) = ("w4V:p11", "w4V:t1", "w4V");

/// `env` in pane `p` of the `pack_two_panes` fake herdr.
fn pane(env: PackEnv, p: (&str, &str, &str)) -> PackEnv {
    env.in_pane(p.0, p.1, p.2)
}

/// Bs-park's heredoc for `session`, parking to `id` with sample parts.
fn park_block(session: &str, id: &str) -> String {
    let blocks = heredoc_commands(&read_skill("bs-park"));
    let cmd = substitute(&blocks[0], session).replace("--card W-12", &format!("--card {id}"));
    fill_park_heredoc(
        &cmd,
        "switched repo to v13",
        "rerun migration test",
        "timeout",
    )
}

/// Bs-park's inline command for `session`: its stdout.
fn park_context(env: &PackEnv, session: &str) -> String {
    let (code, out, err) = env.run_sh(&substitute(PARK_INLINE, session), "");
    assert_eq!(code, 0, "{err}");
    out
}

/// Runs `cmd` expecting exit 0; returns stdout.
fn run_ok(env: &PackEnv, cmd: &str) -> String {
    let (code, out, err) = env.run_sh(cmd, "");
    assert_eq!(code, 0, "{cmd}: {err}");
    out
}

/// A temp env with W-12 where `SESSION_A` linked W-12 in pane `w4V:p9`.
fn a_linked_in_p9() -> PackEnv {
    let env = pane(PackEnv::new().with_w12(), P9);
    env.link("W-12", SESSION_A);
    env
}

#[test]
fn as1_park_records_the_pane_place() {
    let env = a_linked_in_p9();
    let out = run_ok(&env, &park_block(SESSION_A, "W-12"));
    assert_eq!(out, "parked to W-12: migrate invoices to v13\n");
    let cwd = run_ok(&env, "pwd -P");
    let text = run_ok(&env, "cat boards/work/W-12.md");
    let place = text
        .lines()
        .rfind(|l| l.starts_with("<!-- where:"))
        .unwrap();
    for part in [
        format!("cwd={}", cwd.trim()),
        "pane=w4V:p9".to_string(),
        "tab=w4V:t1".to_string(),
        "workspace=w4V".to_string(),
        format!("session={SESSION_A}"),
    ] {
        assert!(place.split(' ').any(|w| w == part), "{part}: {place}");
    }
    assert_ne!(
        place,
        text.lines().find(|l| l.starts_with("<!-- where:")).unwrap()
    );
}

#[test]
fn as1_link_in_a_second_session_keeps_the_first_reference() {
    let env = pane(a_linked_in_p9(), P11);
    let cmd = substitute(&bs_link_command("brain-swap link "), SESSION_B).replace("W-12", "W-7");
    let out = run_ok(&env, &cmd);
    assert_eq!(
        out.lines().next(),
        Some("linked to W-7: fix rounding in act export")
    );
    assert!(latest_note_line(&out).contains("ago"), "{out}");
    let ctx = park_context(&env, SESSION_A);
    assert!(ctx.lines().any(|l| l == "reference: W-12"), "{ctx}");
}

#[test]
fn as2_back_context_reports_a_40_min_old_note() {
    let env = PackEnv::new().with_w12();
    env.link("W-12", SESSION_A);
    let inline = inline_commands(&read_skill("bs-back"));
    let cmd = format!(
        "BRAIN_SWAP_NOW=2026-10-08T11:40:00+03:00 {}",
        substitute(&inline[0], SESSION_A)
    );
    let out = run_ok(&env, &cmd);
    let note = latest_note_line(&out);
    assert!(note.contains("40 min ago"), "{note}");
}

#[test]
fn as3_after_clear_the_picker_puts_linked_here_first() {
    let env = pane(a_linked_in_p9(), P11);
    run_ok(&env, &park_block(SESSION_B, "W-7"));
    let env = pane(env, P9);
    let out = park_context(&env, SESSION_C);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"reference: none"), "{out}");
    let i = lines
        .iter()
        .position(|l| *l == "open cards on work, best guess first:")
        .unwrap_or_else(|| panic!("{out}"));
    let first = lines[i + 1];
    assert!(
        first.starts_with("1. W-12 migrate invoices to v13 [Doing]"),
        "{out}"
    );
    assert!(first.contains(", linked here:"), "{out}");
    assert_eq!(lines.last(), Some(&"o. other board: home, personal"));
}

#[test]
fn as3_a_pick_restores_the_reference_for_the_next_park() {
    let env = pane(PackEnv::new().with_w12(), P9);
    let out = run_ok(&env, &park_block(SESSION_C, "W-12"));
    assert_eq!(out, "parked to W-12: migrate invoices to v13\n");
    let ctx = park_context(&env, SESSION_C);
    assert!(ctx.lines().any(|l| l == "reference: W-12"), "{ctx}");
}

#[test]
fn as3_other_board_pick_sets_a_home_reference() {
    let env = PackEnv::new().with_w12();
    let ls = commands(&read_skill("bs-park"))
        .into_iter()
        .find(|c| c.starts_with("brain-swap ls "))
        .unwrap();
    let out = run_ok(&env, &substitute(&ls, SESSION_C));
    assert!(
        out.lines()
            .any(|l| l == "1. H-3 renew car insurance [Todo] no note"),
        "{out}"
    );
    let out = run_ok(&env, &park_block(SESSION_C, "H-3"));
    assert_eq!(out, "parked to H-3: renew car insurance\n");
    let ctx = park_context(&env, SESSION_C);
    let lines: Vec<&str> = ctx.lines().collect();
    for l in ["reference: H-3", "board: home, column: Todo"] {
        assert!(lines.contains(&l), "{l}: {ctx}");
    }
}
