# E3 Claude pack

Milestone: M2. This epic ships the Claude Code pack: the four skills `/bs-card`, `/bs-park`, `/bs-back` and `/bs-link` as `SKILL.md` files under `pack/claude/skills/`, embedded in the binary, plus `brain-swap install claude`, which writes a versioned copy of them and symlinks it into Claude Code's skills folder. It follows M1 because every pack command only calls section 6 subcommands (`context`, `templates`, `new`, `park`, `link`, `show`, `ls`, `move`) and relies on the session resolution of E2-F3; it comes before the TUI because principle 1 puts the re-entry cue first, and TECHSPEC 15 lets it run in parallel with E4. The spikes of E0-F1 (SP-1, SP-3, SP-4, SP-6) gate it, and E0-F3 scores the shipped `/bs-back` afterwards. Spec: TECHSPEC 2.1, 2.2, 5.1, 6.1 to 6.7, 8 (8.1 to 8.8), 10, 12.1 (Pack, CLI, Manual), 12.2, 13, 14, 15; PRD 7.10 (FR-46 to FR-57), FR-67, AS-1, AS-2, AS-3.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E3-F1 | Skills | E2-F3, E0-F1 | 10 | L |
| E3-F2 | install claude | E3-F1 | 7 | M |

## E3-F1 Skills

- Depends on: E2-F3, E0-F1
- Covers: FR-46, FR-47, FR-48, FR-49, FR-50, FR-51, FR-52, FR-53, FR-54, FR-55, FR-56, FR-57
- Spec: TECHSPEC 2.1, 2.2, 6.3, 6.4, 6.6, 8.1 to 8.7, 12.1 (Pack, Manual), 12.2, 14, 15; PRD 7.10, AS-1, AS-2, AS-3
- Scope: Writes the four skill files of TECHSPEC 8.2 to 8.5 (as updated by E0-F1 if a spike took a fallback) to `pack/claude/skills/<name>/SKILL.md` and embeds them in the library with `include_str!` in `src/adapters/claude.rs`. `tests/pack.rs` checks each file's frontmatter and the 8.1 common rules, runs each inline command with `sh -c` in every `context` state, parses every `brain-swap ...` command of each file with `Cli::try_parse_from`, and runs each heredoc command with sample text against a temp board. A replay test chains the skills' own commands into the CLI side of AS-1, AS-2 and AS-3 inside a fake herdr pane. The Claude side of those scenarios (asking nothing, label parsing, staleness judgement, the picker dialogue) is verified by hand in herdr and Claude Code with the checkout's skills symlinked into `~/.claude/skills/`.
- Provides: `pack/claude/skills/{bs-card,bs-park,bs-back,bs-link}/SKILL.md`; `brain_swap::adapters::claude::Skill` (`pub name: &'static str`, `pub body: &'static str`); `brain_swap::adapters::claude::SKILLS: [Skill; 4]` (embedded, order bs-card, bs-park, bs-back, bs-link); `tests/common/skill_md.rs` (`read_skill`, `frontmatter`, `inline_commands`, `commands`, `heredoc_commands`, `fill_park_heredoc`, `fill_card_heredoc`, `substitute`, `shell_words`, `SESSION_A`, `SESSION_B`, `SESSION_C`); `tests/common/pack_env.rs` (`PackEnv`, `ContextState`, `PackEnv::run_sh`, `PackEnv::link`, `PackEnv::in_pane`); `tests/fixtures/pack/` (work and home boards); `tests/pack.rs`
- Requires: E2-F1 `brain_swap::cli::args::Cli` (public, for `Cli::try_parse_from`) and `templates`; E2-F2 `new`, `move`, `ls`, `show`; E2-F3 `context`, `park`, `link`, session resolution (6.4) and reference setting; E1-F1 `tests/common/spawn.rs` (hermetic spawn helper, `Scenario` with its `xdg` and `path` setters), `tests/layering.rs` and the docs test; E5-F1 fake herdr script (`HERDR_BIN_PATH`, `FAKE_HERDR_SCENARIO`); E0-F1 the TECHSPEC 16 answers to SP-1, SP-3, SP-4 and SP-6 and section 8 as updated by them
- Feature DoD:
  - [ ] `bs-link/SKILL.md` equals TECHSPEC 8.5 and passes its frontmatter, inline, session, move-line and parse checks (E3-F1-T1).
  - [ ] The bs-link commands run as the skill states: link with latest note, unknown ID, picker for work and for another board (E3-F1-T2).
  - [ ] `bs-park/SKILL.md` equals TECHSPEC 8.3; its inline `context` exits 0 with the header line in every 12.1 context state; its heredoc parks (E3-F1-T3).
  - [ ] `bs-back/SKILL.md` equals TECHSPEC 8.4; same checks plus the link, show and catch-up commands (E3-F1-T4).
  - [ ] `bs-card/SKILL.md` equals TECHSPEC 8.2; its inline `templates` exits 0 in every state; its heredoc creates and links (E3-F1-T5).
  - [ ] The binary embeds exactly the four skill files (E3-F1-T6).
  - [ ] The CLI side of AS-1, AS-2 and AS-3 replays through the skills' own commands (E3-F1-T7).
  - [ ] Manual AS-1 (steps 1, 3, 4, 8) recorded, with `pack/claude/skills/*` symlinked into `~/.claude/skills/` by hand for the run (E3-F1-T8).
  - [ ] Manual AS-2 recorded (E3-F1-T9).
  - [ ] Manual AS-3 recorded (E3-F1-T10).

### E3-F1-T1 bs-link skill and static pack checks

- Status: todo
- Depends on: E2-F1-T1, E0-F1
- Covers: FR-46, FR-47; TECHSPEC 8.1, 8.5, 12.1 (Pack: frontmatter checks, commands parse)
- Size: M
- Scope: Add `pack/claude/skills/bs-link/SKILL.md` with the exact text of TECHSPEC 8.5. Create `tests/common/skill_md.rs` with the static helpers: `read_skill(name)` reads `pack/claude/skills/<name>/SKILL.md` under `CARGO_MANIFEST_DIR`; `frontmatter(text)` parses the leading `---` block into keys and values (one pair of surrounding quotes stripped, indented lines collected as a nested map under the previous key, for `metadata`); `inline_commands(text)` returns the commands inside `` !`...` ``; `commands(text)` returns every inline command, every fenced line and every code span that starts with `brain-swap `, a heredoc line cut before `<<`; `substitute(cmd, session)` replaces `${CLAUDE_SESSION_ID}` with `session` and applies the placeholder table (`<ID>` to `W-12`, `<column>` to `Doing`, `<name>` to `home`, `<template>` to `Feature`); `shell_words(cmd)` splits on blanks honouring single and double quotes; `SESSION_A`, `SESSION_B`, `SESSION_C` are fixed valid IDs (`aaaaaaaa-1111-4111-8111-111111111111` and so on). Create `tests/pack.rs` with the tests below; none of them spawns a process.
- Not in scope: running any command (E3-F1-T2); the other three skills (E3-F1-T3 to T5); embedding (E3-F1-T6); the behaviour of `link` and `ls` themselves (E2-F2, E2-F3).
- Tests first: pack layer, static.
  1. `ts8_1_bs_link_frontmatter_names_and_version`: given `bs-link/SKILL.md`, when `frontmatter` parses it, then `name` is `bs-link`, `description` is non-empty, `argument-hint` is `<card ID, e.g. W-12>`, and `metadata` holds `pack: brain-swap` and `version` equal to `env!("CARGO_PKG_VERSION")`.
  2. `fr46_bs_link_user_only_and_cli_only`: given the same frontmatter, when it is read, then `disable-model-invocation` is `true` and `allowed-tools` is exactly `Bash(brain-swap:*)`.
  3. `ts8_5_bs_link_has_no_inline_command`: given the file, when `inline_commands` runs, then it returns an empty list, so `link` runs through the Bash tool and `$ARGUMENTS` never reaches a render-time shell.
  4. `ts12_1_bs_link_commands_parse`: given `commands` of the file plus step 2's `ls` command with `--board home` appended, each passed through `substitute(_, SESSION_A)` and `shell_words`, when each argv goes to `Cli::try_parse_from`, then every one parses and the subcommands found are exactly `link`, `ls`, `ls --board home` and `move`, in file order.
  5. `ts8_1_bs_link_session_passed_quoted`: given every unsubstituted command of the file containing `--session`, when its words are read, then the value after `--session` is written exactly as `"${CLAUDE_SESSION_ID}"` with the double quotes.
  6. `fr47_bs_link_ends_with_move_line`: given the file, when its last non-empty line is read, then it equals ``If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.``
- DoD:
  - [ ] The tests above pass; the first test was committed red before the file turned it green (12.2 item 1).
  - [ ] `bs-link/SKILL.md` is byte-identical to the TECHSPEC 8.5 block as updated by E0-F1, checked with `diff` at review.
  - [ ] Test 4 asserts the expected subcommand list, so an extraction bug that finds no command cannot pass vacuously.
  - [ ] The E1-F1 docs test passes with the new markdown file (no U+2014 or U+2013).

### E3-F1-T2 Runtime pack harness and bs-link command runs

- Status: todo
- Depends on: E3-F1-T1, E2-F3
- Covers: FR-54, FR-55; TECHSPEC 8.5, 12.1 (hermetic spawning)
- Size: M
- Scope: Create `tests/common/pack_env.rs`. `PackEnv::new()` makes a temp `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` (the three folders set through the spawn helper's `xdg(..)`, E1-F1-T9), writes `config.toml` mapping `work` and `home` to temp copies of `tests/fixtures/pack/work/` and `tests/fixtures/pack/home/` and `personal` to a folder that does not exist, and fixes `BRAIN_SWAP_NOW=2026-10-08T11:12:00+03:00`; `with_w12()` copies `tests/fixtures/pack/W-12.md` into work; `run_sh(cmd, stdin)` runs `sh -c <cmd>` through `tests/common/spawn.rs` with `PATH` set through the spawn helper's `path("<dir of CARGO_BIN_EXE_brain-swap>:/usr/bin:/bin")` (E1-F1-T9), and returns status, stdout and stderr; `link(id, session)` seeds a reference by running `brain-swap link <id> --session <session>`. Create the fixtures: `work/board.md` (frontmatter `next: 12`), `work/W-7.md` (Chore, Doing, `# fix rounding in act export`, one note at `2026-10-08T09:12:00+03:00` with Next `check the 0.005 case` and place `cwd=/home/u/Work/ati.acts` only), `work/W-11.md` (Bug, Todo, `# invoice PDF shows the wrong VAT`, empty timeline), `W-12.md` (Feature, Doing, `# migrate invoices to v13`, one note at `2026-10-08T11:00:00+03:00` with all three parts and place `cwd=/home/u/Work/ati.billing pane=w4V:p9 tab=w4V:t1 workspace=w4V session=<SESSION_A>`), `home/H-3.md` (Chore, Todo, `# renew car insurance`, empty timeline). Each test takes its command from the skill file with `commands` and `substitute`.
- Not in scope: the `context` states (E3-F1-T3); herdr places and pane hints (E3-F1-T7); the CLI tests of `link` and `ls` themselves (E2-F2, E2-F3).
- Tests first: pack layer, spawned through the hermetic helper.
  1. `fr54_bs_link_command_links_and_prints_latest_note`: given `PackEnv::new().with_w12()` and no reference for `SESSION_A`, when step 1's command, substituted with `W-12` and `SESSION_A`, runs through `run_sh`, then it exits 0, the first stdout line is `linked to W-12: migrate invoices to v13`, the second is `card: W-12 migrate invoices to v13`, the output holds a `latest note:` line and the fixture note's `Doing:`, `Next:` and `Watch out:` lines, and `brain-swap context --session <SESSION_A>` then prints `reference: W-12`.
  2. `fr54_bs_link_unknown_id_prints_one_error_line_and_changes_nothing`: given the same env, when step 1's command runs with `W-99`, then it exits 3, stdout is empty, stderr is exactly one line starting `brain-swap: `, and `brain-swap context --session <SESSION_A>` still prints `reference: none`.
  3. `fr55_bs_link_without_id_lists_open_cards_by_best_guess`: given the same env, when step 2's `brain-swap ls --open --guess` runs, then it exits 0, prints `open cards on work, best guess first:`, then lines starting `1. W-12 migrate invoices to v13 [Doing]`, `2. W-7 fix rounding in act export [Doing]` and `3. W-11 invoice PDF shows the wrong VAT [Todo] no note`, and ends with `o. other board: home, personal`.
  4. `fr55_bs_link_other_board_lists_that_board`: given the same env, when step 2's command with `--board home` added runs, then it prints `open cards on home, best guess first:` and `1. H-3 renew car insurance [Todo] no note`, and its last line starts with `o. other board:`.
- DoD:
  - [ ] The tests above pass; the first test was committed red (it did not compile without `PackEnv`) before the harness turned it green (12.2 item 1).
  - [ ] Every process the pack tests spawn goes through `tests/common/spawn.rs`; the suite gives the same result when run inside a herdr pane and a Claude Code session (no `HERDR_*` or `CLAUDE_*` value reaches a child).
  - [ ] Each test builds its own `PackEnv`, so tests run in any order and in parallel.

### E3-F1-T3 bs-park skill

- Status: todo
- Depends on: E3-F1-T2
- Covers: FR-46, FR-47, FR-49, FR-50, FR-56; TECHSPEC 8.1, 8.3, 8.6, 12.1 (Pack: inline command in every context state, heredoc runs)
- Size: M
- Scope: Add `pack/claude/skills/bs-park/SKILL.md` with the exact text of TECHSPEC 8.3. Extend `tests/common/skill_md.rs` with `heredoc_commands(text)` (each fenced block whose first line contains `<<'BS_EOF'`, returned whole: command line, body and `BS_EOF`) and `fill_park_heredoc(block, doing, next, watch_out)` (replaces the `...` after `Doing: `, `Next: ` and `Watch out: `). Extend `tests/common/pack_env.rs` with `ContextState` and `PackEnv::for_state(state)`, which builds the env and prepares an inline command for each of the ten 12.1 states: `NoConfig` (no config file), `BrokenConfig` (a `config.toml` whose second line is `[boards`), `MissingFolder` (work mapped to an absent folder), `NoReference`, `DeletedCard` (`SESSION_A` linked to W-12, then `W-12.md` deleted), `EmptySession` (substituted with the empty string), `Unsubstituted` (the literal `${CLAUDE_SESSION_ID}` left in place, so `sh` expands it to empty in the cleared environment), `Placeholder` (substituted with `\${CLAUDE_SESSION_ID}`, so the literal reaches the binary), `WithNote` (`SESSION_A` linked to W-12) and `WithoutNote` (`SESSION_A` linked to W-11).
- Not in scope: how Claude splits the user's words into parts and decides on `--auto` (FR-50 rules, verified by hand in E3-F1-T8); pane capture and the AS-3 recovery (E3-F1-T7); the `park` and `context` subcommands themselves (E2-F3).
- Tests first: pack layer; 1 to 6 static, 7 to 12 spawned.
  1. `ts8_1_bs_park_frontmatter_names_and_version`: given `bs-park/SKILL.md`, when `frontmatter` parses it, then `name` is `bs-park`, `description` is non-empty, `argument-hint` is `[your own words, e.g. next: rerun the test, watch: timeout]`, and `metadata` holds `pack: brain-swap` and `version` equal to `env!("CARGO_PKG_VERSION")`.
  2. `fr46_bs_park_user_only_and_cli_only`: given the same frontmatter, when it is read, then `disable-model-invocation` is `true` and `allowed-tools` is exactly `Bash(brain-swap:*)`.
  3. `ts8_1_bs_park_inline_command_is_context_only`: given the file, when `inline_commands` runs, then it returns exactly `brain-swap context --session "${CLAUDE_SESSION_ID}"`, which contains no `$ARGUMENTS`.
  4. `ts12_1_bs_park_commands_parse`: given `commands` of the file, each substituted for `SESSION_A`, when each argv goes to `Cli::try_parse_from`, then every one parses and the subcommands found are exactly `context`, `ls --open --guess --board home`, `park --card W-12 --session <SESSION_A> --auto` and `move`, in file order.
  5. `ts8_1_bs_park_session_passed_quoted_and_card_named`: given every unsubstituted command of the file, when its words are read, then each `--session` value is written as `"${CLAUDE_SESSION_ID}"` and each `brain-swap park` command carries `--card <ID>`.
  6. `fr47_bs_park_ends_with_move_line`: given the file, when its last non-empty line is read, then it equals the move line of E3-F1-T1 test 6.
  7. `ts12_1_bs_park_inline_context_exits_0_in_every_state`: given each of the ten `ContextState` values, when the inline command prepared for that state runs through `run_sh`, then it exits 0 and its first stdout line is `brain-swap context v1`, and a failure names the state.
  8. `ts8_3_bs_park_inline_context_error_line_has_hint`: given `BrokenConfig`, when the inline command runs, then stdout holds a line starting `error: config ` that names the config path with line and column, and the line after it starts with `hint:` (step 1 of the skill relies on both).
  9. `fr56_bs_park_no_reference_context_prints_picker`: given `NoReference` on `PackEnv::new().with_w12()`, when the inline command runs, then stdout holds `reference: none` and `open cards on work, best guess first:`, the picker lines start `1. W-12`, `2. W-7` and `3. W-11`, and the last line is `o. other board: home, personal`.
  10. `fr49_bs_park_heredoc_parks_to_referenced_card`: given `WithNote`, when step 4's heredoc block, substituted for `SESSION_A` and filled with Doing `switched repo to v13`, Next `rerun migration test` and Watch out `timeout`, runs through `run_sh`, then it exits 0, stdout is exactly `parked to W-12: migrate invoices to v13` and a newline, `W-12.md` holds two notes, the last note's heading ends in ` auto` and its parts equal the sample, and the `column: Doing` line is unchanged.
  11. `fr50_bs_park_heredoc_without_auto_writes_plain_note`: given `WithNote`, when the same filled block runs with `--auto` removed (step 4's rule when all three parts are the user's), then the last note's heading has no ` auto`.
  12. `ts8_6_bs_park_heredoc_keeps_text_literal`: given `WithNote`, when the block runs with Doing set to ``cost is $HOME and `date` "quoted"``, then the stored Doing part is exactly that text (the quoted `<<'BS_EOF'` delimiter prevents expansion).
- DoD:
  - [ ] The tests above pass; the first test was committed red before the file turned it green (12.2 item 1).
  - [ ] `bs-park/SKILL.md` is byte-identical to the TECHSPEC 8.3 block as updated by E0-F1, checked with `diff` at review.
  - [ ] `ContextState` has one builder per state, reused unchanged by E3-F1-T4 and E3-F1-T5.

### E3-F1-T4 bs-back skill

- Status: todo
- Depends on: E3-F1-T3
- Covers: FR-46, FR-47, FR-51, FR-52, FR-53, FR-56; TECHSPEC 6.4, 8.1, 8.4, 8.6, 12.1 (Pack)
- Size: M
- Scope: Add `pack/claude/skills/bs-back/SKILL.md` with the exact text of TECHSPEC 8.4 and its tests, reusing the helpers and `ContextState` of E3-F1-T3. The tests pin the CLI facts each step of the skill reads: the printed card line and parts (step 4), `latest note: none` (step 5), the author session that the staleness rule weighs (step 3), the link after a pick and the `show` fallback (step 2), and the catch-up park (step 5).
- Not in scope: Claude's staleness judgement itself (FR-53, verified by hand in E3-F1-T9 and scored by E0-F3, which may tighten step 3 per 8.6); the 40 minute age of AS-2 (E3-F1-T7).
- Tests first: pack layer; 1 to 5 static, 6 to 12 spawned.
  1. `ts8_1_bs_back_frontmatter_names_and_version`: given `bs-back/SKILL.md`, when `frontmatter` parses it, then `name` is `bs-back`, `description` is non-empty, there is no `argument-hint` key, and `metadata` holds `pack: brain-swap` and `version` equal to `env!("CARGO_PKG_VERSION")`.
  2. `fr46_bs_back_user_only_and_cli_only`: given the same frontmatter, when it is read, then `disable-model-invocation` is `true` and `allowed-tools` is exactly `Bash(brain-swap:*)`.
  3. `ts8_1_bs_back_inline_command_is_context_only`: given the file, when `inline_commands` runs, then it returns exactly `brain-swap context --session "${CLAUDE_SESSION_ID}"`.
  4. `ts12_1_bs_back_commands_parse`: given `commands` of the file, each substituted for `SESSION_A`, when each argv goes to `Cli::try_parse_from`, then every one parses and the subcommands found are exactly `context`, `ls --open --guess --board home`, `link W-12 --session <SESSION_A>`, `show W-12`, `park --card W-12 --session <SESSION_A> --auto` and `move`, in file order.
  5. `ts8_1_bs_back_session_quoted_card_named_move_line_last`: given the unsubstituted file, when its commands and last non-empty line are read, then each `--session` value is `"${CLAUDE_SESSION_ID}"`, each `brain-swap park` carries `--card <ID>`, and the last line is the move line of E3-F1-T1 test 6.
  6. `ts12_1_bs_back_inline_context_exits_0_in_every_state`: given each of the ten `ContextState` values, when the inline command prepared for that state runs, then it exits 0 and its first stdout line is `brain-swap context v1`.
  7. `fr51_bs_back_context_carries_the_printed_fields`: given `WithNote` (note at 11:00, now 11:12), when the inline command runs, then stdout holds `card: W-12 migrate invoices to v13`, a `latest note:` line that contains `12 min ago` and ends with `by this session`, and `Doing:`, `Next:` and `Watch out:` lines equal to the fixture note, which are the fields step 4 prints as `W-12 migrate invoices to v13 (note 12 min old)`.
  8. `fr52_bs_back_context_without_note_reads_none`: given `WithoutNote` (`SESSION_A` linked to W-11), when the inline command runs, then stdout holds `latest note: none`.
  9. `fr53_bs_back_context_names_the_note_author`: given W-12's note parked by `SESSION_A`, `SESSION_B` also linked to W-12, and `SESSION_C` linked to W-7 (whose note has no session), when the inline command runs for each session, then the `latest note:` line ends with `by this session` for A, `by another session` for B and `no session` for C.
  10. `fr56_bs_back_link_after_pick_sets_reference`: given `NoReference` for `SESSION_C` on `PackEnv::new().with_w12()`, when step 2's `brain-swap link <ID> --session "${CLAUDE_SESSION_ID}"`, substituted with `W-12` and `SESSION_C`, runs, then stdout starts with `linked to W-12: migrate invoices to v13` followed by `card: W-12 migrate invoices to v13`, and the inline command for `SESSION_C` then prints `reference: W-12`.
  11. `ts8_4_bs_back_show_fallback_when_no_session_resolves`: given `Unsubstituted` outside herdr, when step 2's link command runs with `W-12`, then it exits 1; and when step 2's `brain-swap show <ID>` runs with `W-12`, then it exits 0 and stdout starts with `card: W-12 migrate invoices to v13`.
  12. `fr52_bs_back_catch_up_heredoc_saves_auto_note`: given `WithNote`, when step 5's heredoc block, substituted for `SESSION_A` and filled with sample parts, runs, then stdout is exactly `parked to W-12: migrate invoices to v13`, `W-12.md` holds two notes, and the last note's heading ends in ` auto`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the file turned it green (12.2 item 1).
  - [ ] `bs-back/SKILL.md` is byte-identical to the TECHSPEC 8.4 block as updated by E0-F1, checked with `diff` at review.
  - [ ] Test 9 uses `brain-swap link` to seed every reference (no session file is written by hand).

### E3-F1-T5 bs-card skill

- Status: todo
- Depends on: E3-F1-T3
- Covers: FR-46, FR-47, FR-48; TECHSPEC 6.3 (`new --body -`), 8.1, 8.2, 8.6, 12.1 (Pack)
- Size: M
- Scope: Add `pack/claude/skills/bs-card/SKILL.md` with the exact text of TECHSPEC 8.2. Add `fill_card_heredoc(block, title, body)` to `tests/common/skill_md.rs`, replacing `<title>` and `<filled skeleton>`; the sample body is the Feature skeleton (`## Goal`, `## Acceptance`, `## Context`) with Goal `Switch invoice reads and writes to the v13 procedures.` and Context ``run $PATH check and `make` "now"``. The tests run against `PackEnv::new()` (work board at `next: 12`, no W-12) and reuse `ContextState` for the inline `templates` command.
- Not in scope: how Claude picks the template, shortens the title and fills fields (FR-48, verified by hand in E3-F1-T8); the `new` and `templates` subcommands themselves (E2-F1, E2-F2, E2-F3).
- Tests first: pack layer; 1 to 5 static, 6 to 10 spawned.
  1. `ts8_1_bs_card_frontmatter_names_and_version`: given `bs-card/SKILL.md`, when `frontmatter` parses it, then `name` is `bs-card`, `description` is non-empty, `argument-hint` is `<task in a few words> [on home|personal]`, and `metadata` holds `pack: brain-swap` and `version` equal to `env!("CARGO_PKG_VERSION")`.
  2. `fr46_bs_card_user_only_and_cli_only`: given the same frontmatter, when it is read, then `disable-model-invocation` is `true` and `allowed-tools` is exactly `Bash(brain-swap:*)`.
  3. `ts8_1_bs_card_inline_command_is_templates_only`: given the file, when `inline_commands` runs, then it returns exactly `brain-swap templates`, which contains no `$ARGUMENTS`.
  4. `ts12_1_bs_card_commands_parse`: given `commands` of the file plus the `new` command with `--board home` appended (the skill's named-board rule), each substituted for `SESSION_A`, when each argv goes to `Cli::try_parse_from`, then every one parses and the subcommands found are exactly `templates`, `new --template Feature --session <SESSION_A> --body -`, the same with `--board home`, and `move`.
  5. `ts8_1_bs_card_session_quoted_and_move_line_last`: given the unsubstituted file, when its commands and last non-empty line are read, then each `--session` value is `"${CLAUDE_SESSION_ID}"` and the last line is the move line of E3-F1-T1 test 6.
  6. `ts12_1_bs_card_inline_templates_exits_0_in_every_state`: given each of the ten `ContextState` values, when `brain-swap templates` runs through `run_sh` in that env, then it exits 0 and stdout holds the line `template: Feature (key f)`.
  7. `fr48_bs_card_heredoc_creates_card_and_links_session`: given `PackEnv::new()` and no reference for `SESSION_A`, when the heredoc block, substituted for `SESSION_A` and filled with title `migrate invoices to v13` and the sample body, runs, then stdout is exactly `created W-12: migrate invoices to v13 (Todo, work)`, `W-12.md` holds `column: Todo`, `template: Feature`, the line `# migrate invoices to v13`, the Goal sentence under `## Goal` and exactly one `## Timeline`, and `brain-swap context --session <SESSION_A>` prints `reference: W-12`.
  8. `fr48_bs_card_named_board_creates_on_that_board`: given `PackEnv::new()`, when the same filled block runs with `--board home` added after `new`, then stdout is exactly `created H-4: migrate invoices to v13 (Todo, home)` and the work board gains no file.
  9. `ts8_6_bs_card_heredoc_keeps_body_literal`: given `PackEnv::new()`, when the filled block runs, then the Context line in `W-12.md` is exactly ``run $PATH check and `make` "now"``.
  10. `fr47_move_line_command_moves_the_card`: given W-12 created by the filled block in Todo, when the move line's command, substituted to `brain-swap move W-12 Doing`, runs, then stdout is `moved W-12 to Doing` and `W-12.md` differs from its bytes before only in the `column:` line.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the file turned it green (12.2 item 1).
  - [ ] `bs-card/SKILL.md` is byte-identical to the TECHSPEC 8.2 block as updated by E0-F1, checked with `diff` at review.
  - [ ] Test 8 shows the global `--board` flag is accepted after `new`'s own options, as the skill tells Claude to append it.

### E3-F1-T6 Embedded skills in the binary

- Status: todo
- Depends on: E3-F1-T1, E3-F1-T3, E3-F1-T4, E3-F1-T5
- Covers: TECHSPEC 2.1, 8.1, 8.8 (embedded skills), 13
- Size: S
- Scope: Add `src/adapters/claude.rs` (declared in `src/adapters/mod.rs`) with `pub struct Skill { pub name: &'static str, pub body: &'static str }` and `pub const SKILLS: [Skill; 4]`, each body taken with `include_str!("../../pack/claude/skills/<name>/SKILL.md")`, in the order bs-card, bs-park, bs-back, bs-link. The module imports nothing from `cli` or `tui` (TECHSPEC 2.2). E3-F2 writes these bodies to disk; nothing reads the checkout at run time.
- Not in scope: writing the skills anywhere (E3-F2-T1); rewriting them for the SP-3 fallback (E3-F2-T7).
- Tests first: pack layer.
  1. `ts8_8_embedded_skills_equal_pack_files`: given `SKILLS`, when each body is compared with `read_skill(name)`, then all four are byte-identical.
  2. `ts8_1_pack_holds_exactly_the_four_embedded_skills`: given the folder `pack/claude/skills/`, when its subfolder names are listed and sorted, then they equal the sorted `SKILLS` names, `bs-back`, `bs-card`, `bs-link`, `bs-park`, and each subfolder holds a `SKILL.md`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the module turned it green (12.2 item 1).
  - [ ] `tests/layering.rs` (E1-F1) passes with the new module.
  - [ ] `cargo package --list --allow-dirty` lists the four `SKILL.md` files, so a packaged build embeds them (FR-68).

### E3-F1-T7 Command replay of AS-1, AS-2 and AS-3

- Status: todo
- Depends on: E3-F1-T3, E3-F1-T4, E3-F1-T5
- Covers: FR-49, FR-52, FR-54, FR-56, FR-57; AS-1 (steps 3, 4), AS-2, AS-3 (CLI side); TECHSPEC 6.4, 6.6, 8.7
- Size: M
- Scope: Tests in `tests/pack.rs` that chain the skills' own commands (taken from the files with `commands`, `heredoc_commands` and `substitute`) the way Claude issues them in AS-1, AS-2 and AS-3, inside fake herdr panes. Add `PackEnv::in_pane(pane, tab, workspace)`, which sets `HERDR_ENV=1`, `HERDR_PANE_ID`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID` and `FAKE_HERDR_SCENARIO=pack_two_panes`, and add that scenario in the E5-F1 fake herdr format: `pane get w4V:p9` and `pane get w4V:p11` answer as live panes running agent sessions `SESSION_A` and `SESSION_B`. No skill text changes.
- Not in scope: what Claude says or decides in these scenarios (E3-F1-T8 to T10); the board, detail view and jump steps of AS-1 (E4-F2, E4-F3, E5-F3); the best guess tiers themselves (E1-F8).
- Tests first: pack layer, fake herdr.
  1. `as1_park_records_the_pane_place`: given `SESSION_A` linked to W-12 in pane `w4V:p9` (tab `w4V:t1`, workspace `w4V`), when bs-park's heredoc block runs as `SESSION_A` in that pane, then stdout is `parked to W-12: migrate invoices to v13` and the new note's place line holds `pane=w4V:p9`, `tab=w4V:t1`, `workspace=w4V`, `session=<SESSION_A>` and the process's `cwd`.
  2. `as1_link_in_a_second_session_keeps_the_first_reference`: given `SESSION_A` linked to W-12 in pane `w4V:p9`, when bs-link's step 1 command for `W-7` runs as `SESSION_B` in pane `w4V:p11`, then its first line is `linked to W-7: fix rounding in act export`, its output holds W-7's `latest note:` line, and bs-park's inline command for `SESSION_A` still prints `reference: W-12`.
  3. `as2_back_context_reports_a_40_min_old_note`: given `SESSION_A` linked to W-12 and `BRAIN_SWAP_NOW=2026-10-08T11:40:00+03:00` (40 minutes after its note), when bs-back's inline command runs for `SESSION_A`, then the `latest note:` line contains `40 min ago`, the age Claude quotes in `no note since 40 min ago on W-12`.
  4. `as3_after_clear_the_picker_puts_linked_here_first`: given `SESSION_A` linked to W-12 in pane `w4V:p9` and then a bs-park heredoc run as `SESSION_B` on W-7 from pane `w4V:p11` (so W-7 has the newest activity), when bs-park's inline command runs for `SESSION_C` in pane `w4V:p9`, then it prints `reference: none`, the first picker line starts with `1. W-12 migrate invoices to v13 [Doing]` and contains `, linked here:`, and the last line is `o. other board: home, personal`.
  5. `as3_a_pick_restores_the_reference_for_the_next_park`: given `SESSION_C` with no reference in pane `w4V:p9`, when bs-park's heredoc block runs as `SESSION_C` with `--card W-12`, then stdout is `parked to W-12: migrate invoices to v13` and bs-park's inline command for `SESSION_C` then prints `reference: W-12`, so the next `/bs-park` asks nothing.
  6. `as3_other_board_pick_sets_a_home_reference`: given `SESSION_C` with no reference, when bs-park's `brain-swap ls --open --guess --board <name>` runs with `home`, then it prints `1. H-3 renew car insurance [Todo] no note`; and when bs-park's heredoc block then runs as `SESSION_C` with `--card H-3`, then stdout is `parked to H-3: renew car insurance` and the inline command for `SESSION_C` prints `reference: H-3` and `board: home, column: Todo`.
- DoD:
  - [ ] The tests above pass; the first test was committed red (no `in_pane`, no scenario) before the harness turned it green (12.2 item 1).
  - [ ] The `pack_two_panes` scenario sits beside the other E5-F1 scenarios and logs argv like them.
  - [ ] Every test passes a valid `--session`, so none depends on `pane_session` (T-15).

### E3-F1-T8 Manual AS-1 run (steps 1, 3, 4, 8)

- Status: todo
- Depends on: E3-F1-T1, E3-F1-T3, E3-F1-T4, E3-F1-T5
- Covers: FR-46, FR-47, FR-48, FR-49, FR-50, FR-51, FR-54; AS-1 (manual)
- Size: S
- Scope: Run the Claude steps of AS-1 by hand in real herdr and Claude Code, with the checkout's skills symlinked into `~/.claude/skills/` by hand (TECHSPEC 15) and scratch boards. The run also checks the Claude path of step 2 (FR-47), the FR-48 named-board and long-title rule and the FR-50 label examples (A-E3-06).
- Not in scope: steps 5 to 7 (board, detail view, jump: E4-F2, E4-F3, E5-F3); `brain-swap install claude` (E3-F2); staleness (E3-F1-T9).
- Procedure:
  1. Install the build (`cargo install --path .`), check `brain-swap` is on `PATH`, and for each of bs-card, bs-park, bs-back and bs-link run `ln -s "$PWD/pack/claude/skills/<name>" ~/.claude/skills/<name>`.
  2. Pick an empty folder `$S` and export `XDG_CONFIG_HOME=$S/config`, `XDG_DATA_HOME=$S/data` and `XDG_STATE_HOME=$S/state`; with them set, create W-1 to W-11 with `brain-swap new` and park one note on W-7 with `brain-swap park --card W-7`.
  3. In herdr pane A (record its ID; AS-1 calls it `w4V:p9`) start Claude Code in that environment and type `/bs-card migrate invoices to v13`. Pass when Claude asks nothing, runs exactly one `brain-swap new --template Feature ...`, replies `created W-12: migrate invoices to v13 (Todo, work)`, and `W-12.md` sits in Todo with the Feature fields filled from the words and the conversation, inventing no facts.
  4. Copy `W-12.md` aside, then ask Claude in pane A to move the card to Doing. Pass when it runs `brain-swap move W-12 Doing`, replies with `moved W-12 to Doing`, and `diff` against the copy shows only the `column:` line changed.
  5. Type `/bs-park`. Pass when Claude asks nothing, runs one `brain-swap park --card W-12 --session ... --auto`, replies exactly `parked to W-12: migrate invoices to v13`, and W-12's timeline gains a note with three parts, a stamp, `auto` and a place line naming pane A, the column unchanged.
  6. Type `/bs-park next: rerun migration test, watch timeout`. Pass when the new note has Next `rerun migration test`, Watch out `timeout`, a drafted Doing and `auto`. Type `/bs-park rerun the next test`: pass when Doing is `rerun the next test`. Type `/bs-park doing: tests green, next: open the MR, watch out: nothing`: pass when all three parts are verbatim and the heading has no `auto`.
  7. In herdr pane B start a second Claude Code session in the same environment and type `/bs-link W-7`. Pass when it replies `linked to W-7: fix rounding in act export` followed by W-7's latest note; then type `/bs-park` in pane A and pass when it parks to W-12 without asking.
  8. In pane B type `/bs-card` followed by a task of more than eight words ending in `on home`. Pass when the reply is `created H-<n>: <title> (Todo, home)`, the title has at most eight words in the user's wording and holds no card ID.
  9. Wait at least one minute, switch to pane A with herdr and type `/bs-back`. Pass when Claude runs no tool and prints exactly `W-12 migrate invoices to v13 (note <age> old)` followed by the latest note's `Doing:`, `Next:` and `Watch out:` lines.
  10. In a fresh session ask Claude in plain words to park the work with the brain-swap skill. Pass when Claude does not invoke `/bs-park` itself (FR-46).
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the Claude Code and herdr versions, the pane IDs used, each procedure step as passed or failed with the observed reply, and every question or extra text Claude produced despite "ask nothing" (TECHSPEC 14 unspiked risk); this replaces 12.2 items 1 and 3.
  - [ ] A failed step is fixed in the skill text, with the pack tests of E3-F1-T1, T3, T4 or T5 kept green, and the run repeated before this task closes.
  - [ ] The scratch folder `$S` is not inside a configured board, and the author's real boards are unchanged after the run.

### E3-F1-T9 Manual AS-2 run

- Status: todo
- Depends on: E3-F1-T1, E3-F1-T3, E3-F1-T4
- Covers: FR-52, FR-53; AS-2 (manual)
- Size: S
- Scope: Run AS-2 by hand: a stale note and a missing note both lead `/bs-back` to a catch-up note, and a fresh conversation leaves a note current. The 40 minute age is produced by editing the note's stamp by hand (A-E3-07).
- Not in scope: scoring the staleness rule over ten sessions (E0-F3, SP-8).
- Procedure:
  1. Set up as E3-F1-T8 steps 1 and 2, with W-12 present (reuse that scratch folder or create W-12 with `brain-swap new "migrate invoices to v13"`).
  2. In pane A start Claude Code, type `/bs-link W-12`, then `/bs-park`.
  3. Edit `W-12.md` by hand: set the newest note's stamp 40 minutes earlier, keeping its offset.
  4. Ask Claude to do one more concrete piece of work on the task, so the conversation shows work after the note.
  5. Type `/bs-back`. Pass when the first line is `no note since 40 min ago on W-12`, Claude runs one `brain-swap park --card W-12 ... --auto`, W-12 gains a note marked `auto` that reflects the later work, and the note is printed headed `W-12 migrate invoices to v13 (note just saved)` with its three parts.
  6. Create W-13 with `brain-swap new "check the export totals"` in the scratch environment; in pane A type `/bs-link W-13`, then `/bs-back`. Pass when the first line is `no note yet on W-13` and a catch-up note marked `auto` is saved and printed headed `W-13 check the export totals (note just saved)`.
  7. In a fresh session type `/bs-link W-12`, then `/bs-back` at once. Pass when the note counts as current: Claude runs no tool and prints `W-12 migrate invoices to v13 (note <age> old)` and the three parts (8.4 step 3: a fresh conversation is current).
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the Claude Code and herdr versions, each procedure step as passed or failed with the observed reply, and any extra text or question Claude produced; this replaces 12.2 items 1 and 3.
  - [ ] A wrong staleness call is recorded as such and handed to E0-F3 rather than patched into step 3 here (8.6 assigns the stricter rule to SP-8).

### E3-F1-T10 Manual AS-3 run

- Status: todo
- Depends on: E3-F1-T1, E3-F1-T3, E3-F1-T4
- Covers: FR-54, FR-55, FR-56, FR-57; AS-3 (manual)
- Size: S
- Scope: Run AS-3 by hand: after `/clear` or in a new session the reference is gone, `/bs-park`, `/bs-back` and `/bs-link` show the picker with the pane's last card on top, one pick restores the reference, `o` reaches another board, and `/bs-link <ID>` needs no list.
- Not in scope: whether compaction changes the session ID (SP-1 in E0-F1; note it here only if it happens).
- Procedure:
  1. Set up as E3-F1-T8 steps 1 and 2; in herdr pane A a Claude Code session is linked to W-12 and has parked it at least once.
  2. Type `/clear`, then `/bs-park`. Pass when Claude shows the work board's open cards numbered exactly as `context` printed them, W-12 first and marked `linked here`, `o. other board: home, personal` last, asks "Park to which card?" and nothing else, and has not parked yet.
  3. Type `1`. Pass when Claude runs one `brain-swap park --card W-12 ...` and replies `parked to W-12: migrate invoices to v13`.
  4. Type `/bs-park` again. Pass when no list appears and it parks to W-12.
  5. Type `/clear`, `/bs-park`, then `o`. Pass when Claude asks which board; answer `home`; pass when it shows home's open cards in the same format; answer `1`; pass when it parks to that card, and a further `/bs-park` asks nothing and parks there.
  6. Type `/clear`, then `/bs-back`. Pass when the list appears with "Back to which card?"; answer `W-12` (an ID, not a number); pass when Claude runs `brain-swap link W-12 ...` and then prints the note as in AS-1 step 8 or saves a catch-up as in AS-2.
  7. Type `/clear`, then `/bs-link W-12`. Pass when no list appears and it prints `linked to W-12: migrate invoices to v13` and the latest note.
  8. Type `/clear`, then `/bs-link` with no ID. Pass when the list appears with "Which card?"; answer `1`; pass when it links that card.
  9. Quit Claude Code, start a new session in pane A and type `/bs-park`. Pass when the picker shows W-12 first, marked `linked here`.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the Claude Code and herdr versions, the pane ID, each procedure step as passed or failed with the observed reply, every question Claude asked beyond the picker, and whether `linked here` appeared (it needs `HERDR_*` in the inline shell, SP-4); this replaces 12.2 items 1 and 3.
  - [ ] A failed step is fixed in the skill text with its pack tests kept green, and the run repeated before this task closes.

## E3-F2 install claude

- Depends on: E3-F1
- Covers: FR-67
- Spec: TECHSPEC 2.2, 5.1, 6.1, 6.2, 6.3, 6.7, 8.8, 10 (atomic writes), 12.1, 12.2, 13, 14 (SP-3, SP-6); PRD FR-67, A-22, A-26
- Scope: `brain-swap install claude` writes the embedded skills (`SKILLS`, E3-F1) to `$XDG_DATA_HOME/brain-swap/pack/<version>/skills/<name>/SKILL.md` and points `~/.claude/skills/<name>` (or `$CLAUDE_CONFIG_DIR/skills/<name>`, SP-6) at each copy through an atomic symlink; `--link <repo>` points them at a checkout's `pack/claude/skills/<name>` instead. A path is ours only when it is a symlink to a versioned copy of any version or to a path ending in `pack/claude/skills/<name>`; anything else at those paths is refused with exit 1 unless `--force`. `--remove` deletes only our links, then the pack folder. Install prints `run /reload-skills in sessions already open` when it created the skills folder, suggests the `Bash(brain-swap:*)` allow rule, never edits Claude Code settings or hooks, and only if SP-3 took its fallback writes the binary's absolute path into the copies.
- Provides: `brain_swap::adapters::claude::install(env: &Env, opts: &InstallOpts) -> Result<InstallReport, Error>`; `brain_swap::adapters::claude::remove(env: &Env) -> Result<InstallReport, Error>`; `brain_swap::adapters::claude::InstallOpts` (`link: Option<PathBuf>`, `force: bool`, `exe: Option<PathBuf>`); `brain_swap::adapters::claude::InstallReport` (`linked: Vec<PathBuf>`, `removed: Vec<PathBuf>`, `notices: Vec<String>`, `warnings: Vec<String>`); `brain_swap::adapters::claude::is_ours(name: &str, target: &Path, pack_root: &Path) -> bool`; `brain_swap::adapters::claude::skills_dir(env: &Env) -> PathBuf`; `brain_swap::cli::cmd::install_claude`; `tests/install_claude.rs`
- Requires: E3-F1 `brain_swap::adapters::claude::SKILLS` and `tests/common/skill_md.rs`; E1-F1 `core::env::Env` (home, XDG data home, `CLAUDE_CONFIG_DIR`, pid, `exe`), `core::error::Error` (`InvalidInput`, `Io`, `exit_code()`, `code()`) and `tests/common/spawn.rs`; E2-F1 `cli::args::Cli` with the `install` subcommand and `cli::args::InstallTarget::Claude { link, force, remove }` (all three flags already parsed, its arm a `todo!()`), the text and `--json` output path (`Out`, with `Out::warn` for `warning: <text>` lines) and the failure envelope of 6.2; E0-F1 the TECHSPEC 16 answers to SP-3 and SP-6
- Feature DoD:
  - [ ] Temp `HOME` test: install writes the versioned copy and the four links (E3-F2-T1).
  - [ ] Install never edits Claude Code settings or hooks (E3-F2-T1).
  - [ ] Temp `HOME` test: an upgrade repoints our links to the new version (E3-F2-T2).
  - [ ] Temp `HOME` test: a foreign folder is refused without `--force` and replaced with it (E3-F2-T3).
  - [ ] `--link <repo>` links a checkout and switches back and forth with the versioned copy (E3-F2-T4).
  - [ ] Temp `HOME` test: removal deletes only our links, then the pack folder (E3-F2-T5).
  - [ ] `$CLAUDE_CONFIG_DIR/skills` is honoured if SP-6 confirmed it (E3-F2-T6).
  - [ ] The SP-3 fallback (absolute path in the copies, `--link` warning) is in place if SP-3 took it (E3-F2-T7).

### E3-F2-T1 Fresh install of the versioned copy

- Status: todo
- Depends on: E3-F1-T6, E2-F1
- Covers: FR-67; TECHSPEC 5.1, 6.7, 8.8, 10
- Size: M
- Scope: Use the `install claude` declaration of E2-F1-T1: replace the `todo!()` arm of `Command::Install(InstallTarget::Claude { .. })` with `cli::cmd::install_claude`, which calls `adapters::claude::install` and prints the report. In `adapters::claude`, `skills_dir(env)` returns `$HOME/.claude/skills` (the `CLAUDE_CONFIG_DIR` case is E3-F2-T6); `install` creates the skills folder if missing and remembers that it did, writes each `SKILLS` body to `$XDG_DATA_HOME/brain-swap/pack/<CARGO_PKG_VERSION>/skills/<name>/SKILL.md` through a temp file renamed into place, then creates `<skills_dir>/<name>` as a temp symlink `.<name>.bs-tmp-<pid>` to the absolute copy folder, renamed over the target (A-E3-15). Text output: one `linked <link> -> <target>` line per link, then `run /reload-skills in sessions already open` when the skills folder was created, then `tip: allow Bash(brain-swap:*) in your Claude Code settings so moves need no prompt` (A-E3-08, A-E3-09). With `--json` stdout is `{"v":1,"linked":[...],"removed":[]}` (6.7) and the reload hint and the tip go to stderr through `Out::warn` as `warning: <text>` lines (6.2: with `--json` stderr carries only warnings). Tests live in `tests/install_claude.rs` and spawn through `tests/common/spawn.rs` with temp `HOME` and `XDG_*`.
- Not in scope: existing entries at the link paths (E3-F2-T2, E3-F2-T3); `--link` (E3-F2-T4); `--remove` (E3-F2-T5); `CLAUDE_CONFIG_DIR` (E3-F2-T6); absolute paths in the copies (E3-F2-T7).
- Tests first: CLI layer, `assert_cmd`, temp `HOME` and `XDG_*`.
  1. `fr67_install_writes_versioned_copies`: given a temp `HOME` without `.claude` and an empty `XDG_DATA_HOME`, when `brain-swap install claude` runs, then it exits 0 and each `<XDG_DATA_HOME>/brain-swap/pack/<CARGO_PKG_VERSION>/skills/<name>/SKILL.md` exists with bytes equal to that skill's `SKILLS` body.
  2. `fr67_install_links_each_skill_to_its_copy`: given the same, when it runs, then each `<HOME>/.claude/skills/<name>` is a symlink whose `read_link` target is the absolute copy folder of that name.
  3. `ts8_8_install_prints_reload_hint_when_it_created_the_skills_dir`: given no `<HOME>/.claude/skills`, when it runs, then stdout holds the line `run /reload-skills in sessions already open`.
  4. `ts8_8_install_omits_reload_hint_when_the_skills_dir_existed`: given an empty `<HOME>/.claude/skills/`, when it runs, then stdout has no `run /reload-skills` line.
  5. `ts8_8_install_suggests_the_allow_rule`: given the same, when it runs, then stdout holds the tip line naming `Bash(brain-swap:*)`.
  6. `ts8_8_install_text_output`: given a temp `HOME` without `.claude`, when it runs, then `insta::assert_snapshot!` of stdout with the temp root replaced through `str::replace` matches.
  7. `ts6_7_install_json_lists_linked_paths`: given the same, when `brain-swap --json install claude` runs, then stdout is exactly one object with `"v": 1`, `linked` holding the four link paths and `removed` empty (`insta::assert_snapshot!` of the raw one-line JSON with the temp root replaced through `str::replace`), and stderr holds the reload hint and the tip only as `warning: ` lines.
  8. `fr67_install_never_edits_claude_settings`: given `<HOME>/.claude/settings.json`, `<HOME>/.claude/settings.local.json` and `<HOME>/.claude/skills/other/SKILL.md` with known bytes, when it runs, then those files are byte-identical and a recursive listing of `<HOME>/.claude` differs from the one taken before only by the four `skills/<name>` entries.
  9. `ts8_8_install_leaves_no_temp_entries`: given a completed install, when the skills folder and the pack folder are listed, then no entry name contains `.bs-tmp-`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `brain-swap install claude --help` describes the command, and README gains a paragraph on installing the Claude Code pack with `brain-swap install claude` (12.2 item 5).
  - [ ] No test can touch the real `~/.claude`: every path derives from the temp `HOME`, and the suite run inside a Claude Code session leaves `~/.claude` unchanged.

### E3-F2-T2 Reinstall and upgrade over our own links

- Status: todo
- Depends on: E3-F2-T1
- Covers: FR-67; TECHSPEC 8.8 (the ours rule)
- Size: M
- Scope: Add `adapters::claude::is_ours(name, target, pack_root)`: true when `target` is `<pack_root>/<one version component>/skills/<name>`, or when its last four components are `pack`, `claude`, `skills`, `<name>`; pure, no filesystem access. `install` inspects each link path with `symlink_metadata` and `read_link`: an absent path, or a symlink of ours (dangling or not), is replaced through the atomic rename of E3-F2-T1. Older `pack/<version>/` folders stay in place (A-E3-12).
- Not in scope: entries that are not ours (E3-F2-T3); checkout links made by `--link` (E3-F2-T4).
- Tests first: 1 to 3 adapter unit, 4 to 6 CLI layer.
  1. `ts8_8_is_ours_accepts_a_versioned_copy_of_any_version`: given pack root `/d/brain-swap/pack`, when `is_ours("bs-park", target, root)` is asked for `/d/brain-swap/pack/0.0.9/skills/bs-park` and `/d/brain-swap/pack/0.1.0/skills/bs-park`, then both answers are true.
  2. `ts8_8_is_ours_accepts_a_checkout_path`: given the same root, when it is asked for `/src/brain-swap/pack/claude/skills/bs-park` and `../bs/pack/claude/skills/bs-park`, then both answers are true.
  3. `ts8_8_is_ours_rejects_other_targets`: given the same root, when it is asked for `/elsewhere/bs-park`, `/d/brain-swap/pack/0.1.0/skills/bs-back`, `/d/brain-swap/pack/0.1.0/extra/skills/bs-park`, `/d/brain-swap/pack/skills/bs-park` and `/src/pack/claude/skills/bs-park/SKILL.md`, then every answer is false.
  4. `fr67_reinstall_of_the_same_version_is_idempotent`: given a completed install, when `brain-swap install claude` runs again, then it exits 0, every link keeps its target and every copy its bytes.
  5. `fr67_upgrade_repoints_links_to_the_current_version`: given copies under `<XDG_DATA_HOME>/brain-swap/pack/0.0.9/skills/<name>/` and the four links pointing at them, when `brain-swap install claude` runs, then it exits 0, each link targets `pack/<CARGO_PKG_VERSION>/skills/<name>`, and `pack/0.0.9/` still exists.
  6. `fr67_install_replaces_a_dangling_link_of_ours`: given `bs-park` linked to `pack/0.0.9/skills/bs-park` whose folder was deleted, when `brain-swap install claude` runs, then it exits 0 and the link targets the current copy.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `is_ours` compares path components, not strings, so `/x/mypack/claude/skills/bs-park` is rejected (a case added to test 3).

### E3-F2-T3 Foreign entries and --force

- Status: todo
- Depends on: E3-F2-T2
- Covers: FR-67; TECHSPEC 6.2, 8.8; PRD A-26
- Size: M
- Scope: Implement the `--force` flag that E2-F1-T1 already parses (the `force` field of `InstallTarget::Claude`). Before writing anything, `install` checks the four link paths: an entry that exists and is not ours (a real folder, a regular file, or a symlink, dangling or not, whose target fails `is_ours`) makes it return `Error::InvalidInput` (exit 1, code `invalid_input`) with the message `<path> exists and was not created by brain-swap (use --force)` for the first such path, and nothing is written (A-E3-10). With `--force`, a foreign folder is deleted recursively and a foreign file or symlink unlinked, each listed in `removed`, then the link is made as in E3-F2-T1 (A-E3-11).
- Not in scope: `--link` (E3-F2-T4); `--remove` (E3-F2-T5).
- Tests first: CLI layer.
  1. `fr67_foreign_directory_is_refused_without_force`: given a real folder `<HOME>/.claude/skills/bs-park/` holding a `SKILL.md` with the text `mine`, when `brain-swap install claude` runs, then it exits 1, stderr is `brain-swap: <HOME>/.claude/skills/bs-park exists and was not created by brain-swap (use --force)`, and the folder and its file are unchanged.
  2. `fr67_a_refusal_writes_nothing`: given the same, when it runs, then no other `<HOME>/.claude/skills/<name>` exists and `<XDG_DATA_HOME>/brain-swap/pack` does not exist.
  3. `fr67_foreign_file_and_symlinks_are_refused_without_force`: given, one case at a time, a regular file at `bs-card`, a symlink `bs-back` to `/elsewhere/bs-back`, and a dangling symlink `bs-link` to `/nowhere/bs-link`, when `brain-swap install claude` runs, then each case exits 1 and leaves the entry as it was.
  4. `ts6_2_a_refusal_under_json_is_the_error_envelope`: given the foreign folder of test 1, when `brain-swap --json install claude` runs, then it exits 1 and stdout is exactly one object `{"v":1,"error":{"code":"invalid_input","message":...}}` carrying the message of test 1.
  5. `fr67_force_replaces_a_foreign_directory`: given the foreign folder of test 1, when `brain-swap --json install claude --force` runs, then it exits 0, `bs-park` is our symlink to the current copy, the old `SKILL.md` is gone, and `removed` lists `<HOME>/.claude/skills/bs-park`.
  6. `fr67_force_replaces_a_foreign_file_and_symlink`: given the regular file and the foreign symlink of test 3 together, when `brain-swap install claude --force` runs, then it exits 0 and both paths are our symlinks.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `--help` describes `--force` as replacing entries brain-swap did not create; README mentions it.

### E3-F2-T4 --link to a checkout

- Status: todo
- Depends on: E3-F2-T2
- Covers: FR-67; TECHSPEC 8.8 (`--link`); PRD A-22
- Size: M
- Scope: Implement the `--link <repo>` flag that E2-F1-T1 already parses (the `link` field of `InstallTarget::Claude`). `install` canonicalizes `<repo>`, checks that each `<repo>/pack/claude/skills/<name>/SKILL.md` exists (else `Error::InvalidInput`, exit 1, message `not a brain-swap checkout: <path> missing`, nothing written; A-E3-13), applies the foreign check of E3-F2-T3 when present, writes no versioned copy, and links `<skills_dir>/<name>` to `<canonical repo>/pack/claude/skills/<name>` through the atomic rename of E3-F2-T1. Each test builds its checkout by copying the repository's `pack/claude/skills/` into a temp folder `<T>/repo`.
- Not in scope: the PATH warning that the SP-3 fallback adds to `--link` (E3-F2-T7); removal of checkout links (E3-F2-T5).
- Tests first: CLI layer.
  1. `fr67_link_points_skills_at_the_checkout`: given a temp checkout `<T>/repo` with the four skill folders, when `brain-swap install claude --link <T>/repo` runs, then it exits 0, each link targets `<canonical T>/repo/pack/claude/skills/<name>`, and `<XDG_DATA_HOME>/brain-swap/pack` does not exist.
  2. `fr67_link_with_a_relative_repo_writes_absolute_targets`: given the checkout as the current folder, when `brain-swap install claude --link .` runs, then every link target is absolute and names the checkout.
  3. `fr67_link_to_a_folder_without_the_skills_fails_and_changes_nothing`: given a checkout missing `pack/claude/skills/bs-back/SKILL.md`, when `brain-swap install claude --link <T>/repo` runs, then it exits 1, stderr names that missing path, and no link exists.
  4. `fr67_link_refuses_a_foreign_entry_without_force`: given a real folder at `<HOME>/.claude/skills/bs-park`, when `brain-swap install claude --link <T>/repo` runs, then it exits 1 and the folder is unchanged.
  5. `fr67_install_after_link_switches_to_the_versioned_copy`: given links made with `--link <T>/repo`, when `brain-swap install claude` runs, then it exits 0 and each link targets the current versioned copy.
  6. `fr67_link_after_install_switches_to_the_checkout`: given a completed plain install, when `brain-swap install claude --link <T>/repo` runs, then it exits 0 and each link targets the checkout.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `--help` and README describe `--link <repo>` for working on the skills from a checkout.

### E3-F2-T5 --remove

- Status: todo
- Depends on: E3-F2-T2, E3-F2-T4
- Covers: FR-67; TECHSPEC 6.7, 8.8 (`--remove`)
- Size: M
- Scope: Implement the `--remove` flag that E2-F1-T1 already parses (the `remove` field of `InstallTarget::Claude`): `cli::cmd::install_claude` calls `adapters::claude::remove(env)` instead of `install`, and `--force` and `--link` given with it change nothing (A-E3-14). For each skill name, `remove` deletes `<skills_dir>/<name>` only when that is a symlink of ours, then deletes `$XDG_DATA_HOME/brain-swap/pack/` recursively if present, returning each deleted path in `removed` (links first, then the pack folder). Text output: one `removed <path>` line per path; `--json`: `{"v":1,"linked":[],"removed":[...]}`.
- Not in scope: the `CLAUDE_CONFIG_DIR` skills folder (E3-F2-T6).
- Tests first: CLI layer.
  1. `fr67_remove_deletes_our_links_and_the_pack_folder`: given a completed install, when `brain-swap install claude --remove` runs, then it exits 0, no `<HOME>/.claude/skills/<name>` exists, `<XDG_DATA_HOME>/brain-swap/pack` does not exist, and stdout holds a `removed <path>` line for each of them.
  2. `fr67_remove_keeps_foreign_entries_and_settings`: given a completed install whose `bs-card` link was then replaced by hand with a real folder, plus `<HOME>/.claude/skills/other/SKILL.md` and `<HOME>/.claude/settings.json`, when `brain-swap install claude --remove` runs, then the `bs-card` folder, `other` and `settings.json` are byte-identical and stdout does not name `bs-card`.
  3. `fr67_remove_deletes_checkout_links_but_not_the_checkout`: given links made with `--link <T>/repo`, when `brain-swap install claude --remove` runs, then the links are gone and every file under `<T>/repo` is unchanged.
  4. `fr67_remove_with_nothing_installed_exits_0`: given an empty temp `HOME`, when `brain-swap --json install claude --remove` runs, then it exits 0 and `removed` is empty.
  5. `ts6_7_remove_json_lists_removed_paths`: given a completed install, when `brain-swap --json install claude --remove` runs, then stdout is one object with `linked` empty and `removed` holding the four links and the pack folder (`insta::assert_snapshot!` of the raw one-line JSON with the temp root replaced through `str::replace`).
  6. `fr67_remove_ignores_force_and_link`: given a completed install, when `brain-swap install claude --remove --force` runs, then it exits 0 and removes only our links and the pack folder (a foreign `<HOME>/.claude/skills/other/SKILL.md` stays byte-identical); and given a fresh completed install, when `brain-swap install claude --remove --link <T>/repo` runs, then it exits 0, the same paths are removed and every file under `<T>/repo` is unchanged.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `--help` and README describe `--remove`.

### E3-F2-T6 $CLAUDE_CONFIG_DIR skills folder

- Status: todo
- Depends on: E3-F2-T5, E0-F1
- Covers: FR-67; TECHSPEC 2.2, 6.1, 8.8 (SP-6)
- Size: S
- Scope: Applies only if TECHSPEC 16 records that Claude Code loads skills from `$CLAUDE_CONFIG_DIR/skills` (SP-6); if it records no, the task closes without code and the reason goes into an empty commit. `skills_dir(env)` returns `$CLAUDE_CONFIG_DIR/skills` when `CLAUDE_CONFIG_DIR` is set and non-empty, else `$HOME/.claude/skills` (A-E3-16); install, the reload hint and `--remove` all follow it. Extend `tests/common/spawn.rs` (E1-F1) to pass `CLAUDE_CONFIG_DIR` when a scenario sets it, as 12.1 does for `VISUAL` and `EDITOR`.
- Not in scope: reading any other `CLAUDE_*` variable (TECHSPEC 2.2 forbids it).
- Tests first: CLI layer.
  1. `ts8_8_claude_config_dir_receives_the_links`: given `CLAUDE_CONFIG_DIR=<T>/cc` without a `skills` folder, when `brain-swap install claude` runs, then each `<T>/cc/skills/<name>` is our symlink, `<HOME>/.claude` does not exist, and stdout holds `run /reload-skills in sessions already open`.
  2. `ts8_8_empty_claude_config_dir_falls_back_to_home`: given `CLAUDE_CONFIG_DIR` set to the empty string, when `brain-swap install claude` runs, then the links are under `<HOME>/.claude/skills`.
  3. `ts8_8_remove_follows_claude_config_dir`: given an install under `<T>/cc`, when `brain-swap install claude --remove` runs with the same variable, then the links under `<T>/cc/skills` are gone and the pack folder is deleted.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The spawn helper still clears every other variable; an existing E1-F1 helper test (or a new one here) shows `CLAUDE_CONFIG_DIR` reaches the child only when the scenario sets it.

### E3-F2-T7 Absolute binary path in the copies (SP-3 fallback)

- Status: todo
- Depends on: E3-F2-T4, E0-F1
- Covers: FR-67; TECHSPEC 8.8 (SP-3), 14
- Size: M
- Scope: Applies only if TECHSPEC 16 records that `brain-swap` is not on the inline shell's `PATH` (SP-3 fallback); otherwise the task closes without code and the reason goes into an empty commit. `cli::cmd::install_claude` passes `env.exe` (the canonical `current_exe` that `src/main.rs` reads into `Env`, E1-F1-T6) as `InstallOpts.exe` (A-E3-17); when it is set, `install` writes each copy with every `brain-swap ` at a command position (start of an inline `` !` `` command, of a fenced command line, or of a code span) replaced by `<exe> `, and the `allowed-tools` value replaced by `Bash(<exe>:*)`; the repository files and `SKILLS` stay unchanged. With `--link`, `install` adds the warning `warning: the checkout's skills need brain-swap on the inline shell's PATH` on stderr. Extend `commands` in `tests/common/skill_md.rs` with a variant taking the program prefix.
- Not in scope: editing Claude Code settings to add a `PATH` (never done, FR-67).
- Tests first: CLI layer, plus the E3-F1 helpers.
  1. `ts8_8_copies_name_the_absolute_binary`: given a fresh temp `HOME`, when `brain-swap install claude` runs, then each copy's `allowed-tools` is `Bash(<canonical CARGO_BIN_EXE_brain-swap>:*)` and every command found with that program prefix starts with that path, while no command starts with `brain-swap `.
  2. `ts8_8_rewritten_commands_still_parse`: given those copies, when each command is substituted for `SESSION_A` and passed to `Cli::try_parse_from`, then every one parses and the subcommand lists equal those of E3-F1-T1, T3, T4 and T5.
  3. `ts8_8_link_warns_that_the_checkout_needs_path`: given a temp checkout `<T>/repo`, when `brain-swap install claude --link <T>/repo` runs, then stderr holds `warning: the checkout's skills need brain-swap on the inline shell's PATH` and the links target the checkout.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `git diff` shows no change under `pack/claude/skills/` from this task.

## Assumptions

- A-E3-01 Does E2-F1 export `brain_swap::cli::args::Cli` from the library, so `tests/pack.rs` can call `Cli::try_parse_from` as 12.1 requires? Affects E3-F1-T1, T3, T4, T5 and E3-F2-T7.
- A-E3-02 Do the pack tests count every inline command, fenced line and code span that starts with `brain-swap ` as a command of the SKILL.md, and substitute placeholders with the fixed table of E3-F1-T1 (`<ID>` W-12, `<column>` Doing, `<name>` home, `<template>` Feature, `${CLAUDE_SESSION_ID}` a fixed valid ID)? Affects E3-F1-T1 to T5.
- A-E3-03 Does the 12.1 state "empty or placeholder session" mean three runs: substituted with the empty string, left unsubstituted (so `sh` expands `"${CLAUDE_SESSION_ID}"` to empty in the cleared environment), and escaped so that the literal `${CLAUDE_SESSION_ID}` reaches the binary? Affects E3-F1-T3, T4, T5.
- A-E3-04 Is a review-time `diff` of each SKILL.md against its TECHSPEC 8.2 to 8.5 block (as updated by E0-F1) enough, with no automated test reading TECHSPEC.md? Affects E3-F1-T1, T3, T4, T5.
- A-E3-05 Does each manual scenario get its own task and its own commit (empty when no file changed) whose message records date, Claude Code and herdr versions, pane IDs, each step passed or failed, and every question or extra text Claude produced despite "ask nothing"? Affects E3-F1-T8, T9, T10.
- A-E3-06 Should the manual AS-1 run also check step 2's Claude path (FR-47), the FR-48 named-board and long-title rule and the FR-50 label examples, although TECHSPEC 15 names only steps 1, 3, 4 and 8? Affects E3-F1-T8.
- A-E3-07 Do the manual runs export a scratch `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` before starting Claude Code, so the real boards stay untouched, and is editing a note's stamp by hand an acceptable way to make it 40 minutes old for AS-2? Affects E3-F1-T8, T9, T10.
- A-E3-08 Is the success text of `install claude` one `linked <link> -> <target>` line per link and one `removed <path>` line per removed path, followed by the 8.8 notices, with the notices on stderr as `warning: <text>` lines through `Out::warn` under `--json` so stdout keeps exactly one object (6.2: with `--json` stderr carries only warnings)? Affects E3-F2-T1, T3, T5.
- A-E3-09 Is the allow-rule suggestion printed after every successful install (not after `--remove`), worded `tip: allow Bash(brain-swap:*) in your Claude Code settings so moves need no prompt`? Affects E3-F2-T1.
- A-E3-10 Are all four link paths checked before anything is written, so a refusal changes nothing, and is the refusal `Error::InvalidInput` (code `invalid_input`, exit 1) with the message `<path> exists and was not created by brain-swap (use --force)` naming the first foreign path? Affects E3-F2-T3, T4.
- A-E3-11 Does `--force` delete a foreign folder recursively (a foreign file or symlink by unlinking it) and list every replaced path in the `removed` field of 6.7? Affects E3-F2-T3.
- A-E3-12 Does an install or upgrade leave older `pack/<version>/` copies in place, so that only `--remove` deletes the pack folder? Affects E3-F2-T2.
- A-E3-13 Does `--link <repo>` canonicalize the path, write no versioned copy, and fail with `invalid_input` (exit 1, message `not a brain-swap checkout: <path> missing`), changing nothing, when any of the four `pack/claude/skills/<name>/SKILL.md` is missing? Affects E3-F2-T4.
- A-E3-14 TECHSPEC 6.3 and 8.8 state no conflict between the flags, so E3-F2-T5 lets `--remove` ignore `--force` and `--link` and exit 0 with an empty `removed` list when nothing is installed. Should `--remove` instead conflict with `--link` and `--force` (clap usage error, exit 2), which would also change E2-F1-T1 test 1? Affects E3-F2-T5.
- A-E3-15 Is "atomically" in 8.8 met by writing each copy file to a temp file renamed into place and each link as a temp symlink `.<name>.bs-tmp-<pid>` renamed over the target, accepting that a foreign folder replaced under `--force` is deleted first and so is not replaced atomically? Affects E3-F2-T1, T3.
- A-E3-16 Is `$CLAUDE_CONFIG_DIR/skills` used when `CLAUDE_CONFIG_DIR` is set and non-empty, only if SP-6 confirms Claude Code loads skills from there, with the reload hint printed when install created that folder, and may E3-F2-T6 extend the E1-F1 spawn helper to pass `CLAUDE_CONFIG_DIR`, which the 12.1 variable list omits? Affects E3-F2-T6.
- A-E3-17 Do both the absolute-path rewrite and the `--link` warning apply only when TECHSPEC 16 records the SP-3 fallback, with the path taken from `Env.exe` (the canonical `current_exe` read in `src/main.rs`) and passed as `InstallOpts.exe`, and the warning worded `warning: the checkout's skills need brain-swap on the inline shell's PATH`? Affects E3-F2-T7.
- A-E3-18 Does E2-F1 declare the `install` subcommand with nested `claude` and `herdr` targets, E3-F2 filling in the `claude` target's handler and E5-F4 the `herdr` one? Affects E3-F2-T1, T3, T4, T5.
- A-E3-19 Does SP-6 confirm symlinked skill folders? E3-F2 assumes yes; its TECHSPEC 14 fallback (copies) would replace the symlink steps of E3-F2-T1 to T5 and need a new ours rule. Affects E3-F2.
- A-E3-20 Do the pack tests seed references only through `brain-swap link` (never by writing session files) and build their boards from `tests/fixtures/pack/` through `PackEnv`, so they do not depend on the E1-F7 file format? Affects E3-F1-T2 to T5, T7.
