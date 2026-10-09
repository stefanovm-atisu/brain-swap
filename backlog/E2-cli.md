# E2 CLI

Milestone: M1. This epic delivers the `brain-swap` subcommands that the Claude pack (E3) drives, that the TUI's editor test (E4-F5) calls, and that every adapter reaches under the public CLI rule (TECHSPEC 2.2): the clap surface with the global `--board` and `--json` flags, the 6.2 exit codes, error envelope and warning lines, `templates`, `new`, `move`, `show`, `ls`, `park`, `link` and `context`, session resolution (6.4), the note grammar on stdin (6.5), the JSON objects (6.7), and the process-level crash and concurrency tests. It sits in M1 with E1 and E5-F1 because principle 1 ships the cue (files, CLI, pack) before the board, and the pack (E3-F1) cannot start before E2-F3 (TECHSPEC 15). Test conventions: TECHSPEC 13 enables no `insta` feature, so neither `assert_json_snapshot!` (feature `json`) nor `Settings::add_filter` (feature `filters`) exists; in every E2 test `<sandbox>` stands for the sandbox root as `Sandbox::redact` prints it, an "insta snapshot" is `insta::assert_snapshot!(sandbox.redact(&output))`, and a "JSON snapshot" (or "insta JSON snapshot") is `insta::assert_snapshot!(sandbox.redact(&stdout))` on the raw one-line JSON. A test that sets `HERDR_*`, `FAKE_HERDR_SCENARIO`, `BRAIN_SWAP_SESSION`, `BRAIN_SWAP_LOG`, `BRAIN_SWAP_NOW` or `BRAIN_SWAP_FAILPOINT` does so through `Sandbox::scenario_mut()` before `cmd(args)`. Spec: TECHSPEC 6 (6.1 to 6.7; `locate`, `jump` and `install` only as clap declarations, their behaviour is E5-F2, E5-F4 and E3-F2), 12.1 (CLI, crash and concurrency bullets), 15 (E2 entries).

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E2-F1 | Skeleton | E1-F3, E1-F4 | 4 | M |
| E2-F2 | new, move, ls, show | E2-F1, E1-F6, E1-F8 | 8 | L |
| E2-F3 | park, link, context | E2-F2, E1-F7, E5-F1 | 12 | L |

## E2-F1 Skeleton

- Depends on: E1-F3, E1-F4
- Covers: FR-11, FR-33, FR-35, FR-36
- Spec: TECHSPEC 2.1, 6.1, 6.2, 6.3 (usage line, `templates`), 6.7 (`templates`), 10 (Config bullet), 12.1 (CLI)
- Scope: The binary entry: `main.rs` (created by E1-F1-T1, with `Env` built unchanged by E1-F1-T6 through `Env::from_vars`) parses the arguments, reports an `Env::from_vars` error with its exit code, and hands `Env` and the parsed arguments to `cli::run`, which applies the section 11 rule that ignores a `BRAIN_SWAP_LOG` path inside a board folder once the config loads. `cli::args` declares the whole 6.3 surface with clap (eleven subcommands, every flag, global `--board` and `--json`), so that E3-F1's pack test can parse every SKILL.md command and E5-F2, E5-F4 and E3-F2 only fill in their handlers. `cli::out` prints results as text or as one versioned JSON object, errors as `brain-swap: <message>` or the `{"v":1,"error":{...}}` envelope, warnings as `warning: <text>`, and returns each `Error`'s 6.2 exit code. `templates` is the first complete subcommand, and the config prelude turns a broken `config.toml` into exit 1. Exit codes 3 to 6 are asserted by the features that produce them (3 in E2-F2, 4 and 5 in E2-F3, 6 in E5-F2).
- Provides: `cli::args::Cli` (clap derive; `Cli::try_parse_from` for E3-F1's pack test; fields `board: Option<String>`, `json: bool`, `command: Option<Command>`), `cli::args::Command` (variants `New`, `Park`, `Context`, `Show`, `Ls`, `Link`, `Move`, `Locate`, `Jump`, `Templates`, `Install(InstallTarget)`), `cli::args::InstallTarget` (`Claude { link: Option<PathBuf>, force: bool, remove: bool }`, `Herdr`), `cli::run(env: &mut Env, cli: Cli) -> i32` (dispatch, output, 6.2 exit code), `cli::parse_failure(err: clap::Error, argv: &[OsString]) -> i32` (help exit 0, usage exit 2), `cli::out::Out` (`result`, `warn(&str)`, `error(&Error) -> i32`), `cli::out::Envelope<T>` (typed, `v` first), `cli::cmd::Ctx` (`Ctx::load(env: &mut Env) -> Result<Ctx, Error>`: the loaded config and `warnings: Vec<String>`, with `core::log::guard_board_folders` applied to `env`), `tests/common/cli.rs` (`Sandbox`: `Sandbox::new()` makes a temp root with `home/`, `config/`, `data/`, `state/` and no config, its `Scenario` built as `Scenario::new(root).xdg(<root>/config, <root>/data, <root>/state)` (E1-F1-T9), so `HOME` is `<root>/home` and `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` are the other three; `write_config()` writes boards `work`, `home`, `personal` at `<root>/boards/<name>` in that order with `default_board = "work"`; `copy_board(fixture, board)` copies `tests/fixtures/boards/<fixture>/` replacing `@SANDBOX@` with the root; `cmd(args)` builds a command through the 12.1 spawn helper (`brain_swap(&scenario)`) from that `Scenario`, whose defaults give `TZ=UTC` and `BRAIN_SWAP_NOW=2026-10-08T11:52:00+03:00`, with cwd `<root>/elsewhere` (created empty, so no fixture place lies inside or around it, 6.6); `redact(&self, text: &str) -> String` replaces the sandbox root with `<sandbox>` through `str::replace`, used for every snapshot since section 13 enables no insta `filters` feature; `scenario_mut(&mut self) -> &mut Scenario` gives the E1-F1 spawn helper `Scenario` behind `cmd`, whose setters (`herdr`, `fake_scenario`, `session`, `log`, `failpoint`, `now`) the E2-F3, E5-F2, E5-F4 and E6-F1 tests call before `cmd(args)`). In every E2 test, "the sandbox" means `Sandbox::new()` plus `write_config()` unless the test says otherwise.
- Requires: E1-F1 `core::env::Env` with `Env::from_vars` (E1-F1-T6; it reads every 6.1 variable, fails an invalid `BRAIN_SWAP_NOW` with `InvalidInput`, and reads `BRAIN_SWAP_FAILPOINT` under the feature per E1-F1-T7), `core::error::Error` with `exit_code()`, `code()` and one-line `Display`, `core::log::guard_board_folders` (E1-F1-T8), XDG path computation, the 12.1 spawn helper (`Scenario` and its setters); E1-F3 config loading and validation returning `Error::Config { path, line, col }`; E1-F4 the merged template list (name, key, skeleton, load warnings)
- Feature DoD:
  - [ ] Exit code 0 produced by a test: `fr11_templates_prints_builtins` (E2-F1-T1).
  - [ ] Exit code 2 produced by a test: `ts6_2_unknown_subcommand_exits_2` (E2-F1-T3).
  - [ ] The JSON error envelope on stdout produced by a process-level test: `ts6_2_usage_error_json_envelope` (E2-F1-T3).
  - [ ] Exit code 1 produced by a test: `ts6_2_broken_config_exits_1_naming_line_and_column` (E2-F1-T4).

### E2-F1-T1 Binary entry, clap surface and `templates` text

- Status: todo
- Depends on: E1-F1, E1-F4
- Covers: FR-11, FR-33; TECHSPEC 2.1, 6.1, 6.3 (`templates`)
- Size: M
- Scope: Extend `src/main.rs` (created by E1-F1-T1; `Env` built unchanged by E1-F1-T6 through `Env::from_vars`, which already reads every 6.1 variable including `BRAIN_SWAP_NOW`, and `BRAIN_SWAP_FAILPOINT` under the feature per E1-F1-T7): parse with `Cli::try_parse_from`, print clap's help on `--help` with exit 0, then build `Env`; on an `Env::from_vars` error print `brain-swap: <Display>` on stderr and exit with its `exit_code()` (E2-F1-T2 routes this through `Out::error`); otherwise call `cli::run` and exit with its code. With no subcommand, `main.rs` instead calls `tui::run(env, cli.board, std::io::stdin().is_terminal() && std::io::stdout().is_terminal())` and exits with its code; if `src/tui/mod.rs` has no `tui::run` yet, add `pub fn run(env: Env, board: Option<String>, terminal: bool) -> i32 { todo!() }` there, which E4-F1-T8 replaces. `main.rs` reads no environment variable itself beyond what E1-F1-T6 passes to `Env::from_vars`. Create `src/cli/args.rs` with the full 6.3 surface: `brain-swap [--board <name>] [--json] [<subcommand>]` with both flags global; `new [--template <name>] [--column <name>] [--session <id>] [--body <src>] [<title>...]`; `park [--card <ID>] [--session <id>] [--auto]`; `context [--session <id>]`; `show <ID> [--all]`; `ls [--open] [--guess]`; `link <ID> [--session <id>]`; `move <ID> <column>`; `locate <ID> [--note <n>]`; `jump <ID> [--note <n>]`; `templates`; `install claude [--link <repo>] [--force] [--remove]` (the `--remove` conflicts are E3-F2-T5's); `install herdr`. Card ID arguments parse through `CardId`'s parser (case-insensitive). Implement `templates` in `src/cli/cmd.rs`: per template `template: <Name> (key <k>)`, or `template: <Name>` without a key, then the skeleton verbatim, one blank line between templates, in E1-F4's order; it never opens `config.toml` (A-E2-01). Arms of subcommands owned by later tasks are `todo!()` placeholders that no test reaches; the owning task replaces them (A-E2-06). Create `tests/common/cli.rs` (`Sandbox`, see Provides) and `tests/cli_templates.rs`.
- Not in scope: JSON output and warning lines (E2-F1-T2); usage errors and the exit code table in `--help` (E2-F1-T3); the config prelude (E2-F1-T4); template loading and override rules (E1-F4); reading `BRAIN_SWAP_NOW` and `BRAIN_SWAP_FAILPOINT` into `Env` (E1-F1-T6, E1-F1-T7); the section 11 rule that ignores a `BRAIN_SWAP_LOG` path inside a board folder (E2-F1-T4); the bare-invocation gate and `tui::run` (E4-F1-T8).
- Tests first:
  1. `ts6_1_every_subcommand_parses` (CLI args unit): given each 6.3 form `new --template bug --column Doing --session s --body - fix it`, `park --card W-12 --session s --auto`, `context --session s`, `show w-12 --all`, `ls --open --guess`, `link W-12 --session s`, `move W-12 Doing`, `locate W-12 --note 2`, `jump W-12 --note 1`, `templates`, `install claude --link /r --force`, `install claude --remove` and `install herdr`, when `Cli::try_parse_from` runs on `brain-swap` followed by the form, then it returns `Ok` with the matching `Command` variant and field values, and `show w-12` holds `CardId { letter: 'W', number: 12 }`.
  2. `ts6_1_board_and_json_are_global` (CLI args unit): given `brain-swap ls --board home --json` and `brain-swap --board home --json ls`, when parsed, then both give `board == Some("home")` and `json == true`.
  3. `fr11_templates_prints_builtins`: given `Sandbox::new()` (no config, no templates folder), when `brain-swap templates` runs, then it exits 0 and stdout matches an insta snapshot holding `template: Feature (key f)`, `template: Bug (key b)`, `template: Research (key r)` and `template: Chore (key c)` in that order, each followed by its 5.4 skeleton.
  4. `fr11_templates_reads_user_folder_from_xdg_config_home`: given `$XDG_CONFIG_HOME/brain-swap/templates/spike.md` holding only `## Question\n`, when `brain-swap templates` runs, then stdout contains the line `template: Spike` followed by `## Question`, and the four built-in header lines are still present.
  5. `ts6_1_config_folder_falls_back_to_home`: given the sandbox scenario with `unset("XDG_CONFIG_HOME")` (E1-F1-T9) and `$HOME/.config/brain-swap/templates/spike.md`, when `brain-swap templates` runs, then stdout contains `template: Spike`.
  6. `ts6_3_templates_never_parses_config`: given a `config.toml` whose only line is `default_board = ` (unparseable), when `brain-swap templates` runs, then it exits 0, stdout equals test 3's snapshot and the config bytes are unchanged.
  7. `ts6_3_templates_writes_no_config`: given `Sandbox::new()`, when `brain-swap templates` runs, then it exits 0 and `$XDG_CONFIG_HOME/brain-swap/config.toml` does not exist afterwards (A-E2-01).
  8. `fr33_help_lists_every_subcommand`: given the sandbox, when `brain-swap --help` runs, then it exits 0 and stdout names `new`, `park`, `context`, `show`, `ls`, `link`, `move`, `locate`, `jump`, `templates` and `install`.
  9. `ts6_1_invalid_brain_swap_now_exits_1`: given the sandbox and `BRAIN_SWAP_NOW=tomorrow`, when `brain-swap ls` runs, then it exits 1, stdout is empty and stderr is `brain-swap: BRAIN_SWAP_NOW: invalid stamp 'tomorrow'` (the `ls` arm is never reached).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every environment read is in `src/main.rs`; `src/core/` gains no `std::env` or `std::process` use and `tests/layering.rs` passes.
  - [ ] `Sandbox` spawns only through the 12.1 spawn helper and every test owns its temp folders.

### E2-F1-T2 JSON envelope, error and warning output

- Status: todo
- Depends on: E2-F1-T1
- Covers: FR-35, FR-36, FR-11; TECHSPEC 6.1 (`--json`), 6.2 (stderr and envelope rules, error codes), 6.7 (`templates`)
- Size: M
- Scope: `src/cli/out.rs`: `Out` writes a text result to stdout, or with `--json` exactly one compact JSON object on one line whose first key is `"v": 1` (A-E2-03). Every output object is a `#[derive(Serialize)]` struct serialised directly: section 13 enables no `preserve_order` on `serde_json`, so `serde_json::Value` maps would sort keys and lose `v` first; `Envelope<T>` is `v` plus the flattened body. `Out::error` prints `brain-swap: <Display>` on stderr, or with `--json` `{"v":1,"error":{"code":"<code()>","message":"<Display>"}}` on stdout and nothing on stderr, and returns `exit_code()`. `Out::warn` prints `warning: <text>` on stderr in both modes. `templates --json` returns `templates`, each with `name`, `key` (null without one) and `skeleton`; E1-F4's load warnings go through `Out::warn`. `main.rs` now prints an `Env::from_vars` error through `Out::error`, so `--json` gets the envelope.
- Not in scope: clap parse failures (E2-F1-T3); the Card, note and Candidate objects (E2-F2-T1, E2-F2-T5, E2-F2-T8); the `context` object (E2-F3-T8 to E2-F3-T10); the `locate`, `jump` and `install` objects (E5-F2, E5-F4, E3-F2).
- Tests first:
  1. `fr35_templates_json_is_one_object_v_first`: given `Sandbox::new()`, when `brain-swap --json templates` runs, then it exits 0, stdout is one line starting with `{"v":1,` that parses as one JSON object whose `templates` array holds four entries with `name`, `key` and `skeleton` (insta JSON snapshot).
  2. `fr35_json_flag_after_subcommand`: given the same sandbox, when `brain-swap templates --json` runs, then stdout is byte-identical to test 1's.
  3. `fr35_templates_json_null_key`: given the user template `spike.md` of E2-F1-T1 test 4, when `brain-swap --json templates` runs, then the entry named `Spike` has `"key":null`.
  4. `ts6_2_error_text_on_stderr` (unit, `cli::out`): given each `Error` variant (`Io`, `Unreadable`, `InvalidInput`, `Config`, `Verify`, `Usage`, `NotFound`, `NoReference`, `Busy`) and text mode, when `Out::error` writes to in-memory streams, then stderr is `brain-swap: <Display>` plus a newline, stdout is empty, and the returned code is 1, 1, 1, 1, 1, 2, 3, 4, 5 respectively.
  5. `ts6_2_error_json_envelope` (unit, `cli::out`): given the same variants and JSON mode, when `Out::error` writes, then stdout is exactly `{"v":1,"error":{"code":"<code>","message":"<Display>"}}` plus a newline with codes `io`, `unreadable`, `invalid_input`, `config`, `verify_failed`, `usage`, `not_found`, `no_reference`, `busy`, and stderr is empty.
  6. `ts6_2_warning_prefixed_on_stderr` (unit, `cli::out`): given text mode and then JSON mode, when `Out::warn("x")` runs, then stderr is `warning: x` plus a newline in both and stdout is untouched.
  7. `ts5_4_unreadable_template_folder_keeps_json_one_object`: given `$XDG_CONFIG_HOME/brain-swap/templates` created as a regular file, when `brain-swap --json templates` runs, then it exits 0, stdout is one object listing the four built-ins, and stderr holds at least one line starting `warning: `.
  8. `ts6_2_invalid_brain_swap_now_json_envelope`: given the sandbox and `BRAIN_SWAP_NOW=tomorrow`, when `brain-swap --json ls` runs, then it exits 1, stdout is exactly `{"v":1,"error":{"code":"invalid_input","message":"BRAIN_SWAP_NOW: invalid stamp 'tomorrow'"}}` and stderr is empty.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] No output object in `src/cli/` is built with `serde_json::Value` or `json!` (a grep finds none), so `v` is first by construction.
  - [ ] The 6.2 code strings live only in `Error::code()` (E1-F1); `cli::out` does not repeat them.

### E2-F1-T3 Usage errors and the exit code table in help

- Status: todo
- Depends on: E2-F1-T2
- Covers: FR-36; TECHSPEC 6.2 (exit 2, code `usage`)
- Size: S
- Scope: `cli::parse_failure` handles a `clap::Error`: help (`DisplayHelp`) prints and exits 0; any other kind becomes `Error::Usage` with the message `<clap's first error line without its "error: " prefix>`, printed through `Out::error` (exit 2, code `usage`) and followed in text mode by clap's usage lines (A-E2-04). Because parsing failed, `--json` is detected by scanning the raw arguments for `--json` before any `--`. The top-level `--help` ends with the 6.2 exit code table (0 to 6 with their meaning), which is how FR-36's "documented" is met (A-E2-02); README gets the same table.
- Not in scope: exit 2 of the bare TUI without a terminal (E4-F1-T8); `--body` value checking (E2-F2-T3); malformed card IDs inside a valid parse are clap value errors and so land here (A-E2-05).
- Tests first:
  1. `ts6_2_unknown_subcommand_exits_2`: given the sandbox, when `brain-swap frobnicate` runs, then it exits 2, stdout is empty and stderr's first line starts with `brain-swap: ` and contains `frobnicate` (insta snapshot of stderr).
  2. `ts6_2_missing_argument_exits_2`: given the sandbox, when `brain-swap move W-12` runs, then it exits 2 and stderr's first line starts with `brain-swap: `.
  3. `ts6_2_invalid_card_id_exits_2`: given the sandbox, when `brain-swap show W12` runs, then it exits 2 (A-E2-05).
  4. `ts6_2_usage_error_json_envelope`: given the sandbox, when `brain-swap --json frobnicate` runs, then it exits 2, stdout is exactly one object with `"v":1` and `"error":{"code":"usage","message":...}`, and stderr is empty (insta JSON snapshot).
  5. `fr36_help_documents_exit_codes`: given the sandbox, when `brain-swap --help` runs, then it exits 0 and stdout ends with a table naming 0 success, 1 runtime error, 2 usage, 3 not found, 4 no reference, 5 busy and 6 jump not performed (insta snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] README carries the same exit code table as `--help` (12.2 item 5).

### E2-F1-T4 Config prelude and config errors

- Status: todo
- Depends on: E2-F1-T2, E1-F3
- Covers: FR-36; TECHSPEC 6.2 (exit 1, code `config`), 10 (Config bullet), 11 (`BRAIN_SWAP_LOG` inside a board folder)
- Size: S
- Scope: `cli::cmd::Ctx::load(env)` loads `config.toml` through E1-F3. After a successful load it calls `core::log::guard_board_folders(env, &config.boards)` and keeps a returned text in `Ctx.warnings`; `cli::run` (now taking `env: &mut Env`) prints each `Ctx.warnings` entry through `Out::warn`, and `context` (E2-F3-T10), which calls `Ctx::load` itself, prints it as a `warning:` line, so no subcommand appends its log inside a board folder (section 11, FR-05). `cli::run` calls it before dispatch for every subcommand except `templates`, `install` and `context` (`context` handles its own result, E2-F3-T10; A-E2-06). A parse or 5.2 validation failure is `Error::Config { path, line, col }`: the command exits 1 with `brain-swap: config <path>:<line>:<col>: <problem>` (or the envelope with code `config`) before any board is touched, never falling back to defaults (section 10). The first-run branch (no config file) is added by E2-F2-T1, where a working subcommand can show it.
- Not in scope: the guard's path test and warning text (E1-F1-T8); the TUI's call of the guard (E4-F1-T8); first-run write and notice (E2-F2-T1); config warnings on stderr and the CLI test of the log guard (E2-F2-T7, the first config-loading subcommand); `context`'s `error: config` line (E2-F3-T10); the TUI's ConfigError (E4-F1); parsing and validation rules and their texts (E1-F3).
- Tests first:
  1. `ts6_2_broken_config_exits_1_naming_line_and_column`: given a config whose lines are `default_board = "work"`, `editor = ""`, `[boards]`, `work "~/b"`, when `brain-swap ls` runs, then it exits 1, stdout is empty and stderr is one line `brain-swap: config <path>:4:<col>: <problem>` (insta snapshot of the redacted stderr).
  2. `ts6_2_broken_config_json_envelope`: given the same config, when `brain-swap --json ls` runs, then it exits 1, stdout is one object with `"error":{"code":"config","message":"config <path>:4:..."}` and stderr is empty (insta JSON snapshot).
  3. `ts6_2_invalid_default_board_exits_1`: given a parseable config with `default_board = "nope"` and `[boards]` holding only `work = "~/w"`, when `brain-swap ls` runs, then it exits 1, the JSON variant has code `config`, and the message names line 1.
  4. `ts10_broken_config_touches_no_board`: given the config of test 1, when `brain-swap new x` runs, then it exits 1 and neither `$HOME/.local/share/brain-swap/` nor `<root>/boards/` exists.
  5. `ts11_ctx_load_drops_log_inside_board_folder` (unit, `cli::cmd::Ctx::load`): given an `Env` from `core_env` whose config file lists `work = "<tmp>/boards/work"` and whose `log` is `<tmp>/boards/work/bs.log`, when `Ctx::load(&mut env)` runs, then `env.log` is `None` and `ctx.warnings` holds `BRAIN_SWAP_LOG inside board work, ignored`; given `log` `<tmp>/bs.log`, then `env.log` is unchanged and `ctx.warnings` holds no such text.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] No subcommand handler runs when `Ctx::load` fails (the `todo!()` arms of later tasks stay unreached in these tests).

## E2-F2 new, move, ls, show

- Depends on: E2-F1, E1-F6, E1-F8
- Covers: FR-13, FR-38, FR-39
- Spec: TECHSPEC 4.3 (`board.md` error), 4.7 (extra columns), 6.1 (`--board`, card IDs), 6.2 (exit 1 `io` and `unreadable`, exit 3 and its texts), 6.3 (`new`, `move`, `show`, `ls`, picker line), 6.6 (labels and order as printed), 6.7 (Card, note, Candidate; `new`, `move`, `show`, `ls`), AS-9 (subcommand half)
- Scope: The subcommands that need no session. `new` creates a card from a template skeleton or from a body on stdin, in the first or named column, and prints `created <ID>: <title> (<column>, <board>)`; its `--session` flag parses but resolves nothing until E2-F3-T5. `move` changes only the column line. `show` prints the card block from `card:` on, which `link` and `context` reuse. `ls` prints picker lines in column order, and with `--guess` in best-guess order with labels and the `o. other board:` line. Board, template, column and card names resolve here with the 6.2 `not_found` texts, and the first run writes the config with its stderr notice.
- Provides: `cli::cmd::Ctx::board(&self, name: Option<&str>) -> Result<Board, Error>` (default board; delegates to E1-F3's `Config::board`, unknown board exit 3), `cli::cmd::find_template(&Ctx, &str) -> Result<Template, Error>` (delegates to E1-F4's `core::template::find`), `cli::cmd::find_column(&Board, &str) -> Result<String, Error>` (delegates to E1-F5's `Board::column`), `cli::cmd::find_card(&Ctx, CardId) -> Result<(Board, Card), Error>` (wraps E1-F5's `core::board::find_card` (E1-F5-T4), whose error is already `Error::NotFound("<ID> not found")`, exit 3), `cli::cmd::split_body(text: &str) -> (Option<String>, String)` (title line, body), `cli::out::card_json`, `cli::out::note_json`, `cli::out::SessionView` (`Unresolved` for `show`; `Resolved(Option<String>)` for `link` and `context`), `cli::out::card_block(&Card, &Board, now: &Stamp, &SessionView) -> String` (the block from `card:` on), `cli::out::picker_line(n: usize, &Card, label: Option<&str>, now: &Stamp) -> String`, `cli::out::preview(&Note) -> String`, `cli::out::other_boards_line(&Ctx, shown: &str) -> Option<String>`, `tests/fixtures/boards/work/` (the 4.9 worked example plus `W-7.md`, places under `@SANDBOX@`)
- Requires: E1-F5 board loading and card lookup by ID (`Board`, `Card`, `activity()`, `is_open()`, board warnings, extra columns, `core::board::find_card` failing `NotFound("<ID> not found")`, `Board::column` failing with the 6.2 column text); E1-F6 `core::store` create (with `Created.warnings`, which carry E1-F2's `## Timeline in body dropped`) and move; E1-F8 best guess (tiers, reasons, order); E1-F1 the FR-21 age text (`core::time`); E1-F3 first-run write, config warnings and `Config::board` (6.2 board text); E1-F4 templates and `core::template::find` (6.2 template text); E2-F1 `Ctx`, `Out`, `Sandbox`
- Feature DoD:
  - [ ] Text and JSON snapshots of `new` (E2-F2-T1).
  - [ ] Unknown board, template and column names exit 3 and list the valid names, in text and JSON (E2-F2-T2).
  - [ ] `new --body -` snapshots with and without a positional title (E2-F2-T3).
  - [ ] Text snapshots of `show` (E2-F2-T4).
  - [ ] JSON snapshots of `show` and `show --all` (E2-F2-T5).
  - [ ] Text and JSON snapshots of `move` (E2-F2-T6).
  - [ ] Text and JSON snapshots of `ls` and `ls --open` (E2-F2-T7).
  - [ ] Text and JSON snapshots of `ls --open --guess` (E2-F2-T8).

### E2-F2-T1 `new <title>` on the default board, and the first run

- Status: todo
- Depends on: E2-F1-T4, E1-F6
- Covers: FR-13; TECHSPEC 4.3 (invalid `board.md`), 6.2 (exit 1 `io`, `unreadable`, `invalid_input`), 6.3 (`new` defaults and output), 6.7 (Card, `new`), AS-9 (subcommand half)
- Size: M
- Scope: `cmd::new` with positional title words: joined by single spaces and trimmed (A-E2-09); no title exits 1 with `Error::InvalidInput` `no title` (A-E2-08); the default template `feature` in the first column of the default board (`Ctx::board(None)`); creation through `core::store`; prints `created <ID>: <title> (<column>, <board>)`; `--json` returns `card`. `out::card_json` renders the 6.7 Card (`id`, `board`, `title`, `column`, `open`, `path`, `latest_note`, absent values null). Store errors map through `Out::error`: an invalid `board.md` frontmatter is `unreadable` with message `board.md:<line>: <problem>` and writes nothing; a board folder that cannot be created is `io`. `Ctx::load` gains the first-run branch: with no config file, E1-F3 writes the default config with create-new semantics and the command prints `created config <path>` on stderr in text mode, and under `--json` prints it through `Out::warn` as `warning: created config <path>` so stderr carries only warnings (6.2, A-E2-34), then continues.
- Not in scope: `--template`, `--column`, `--board` and unknown names (E2-F2-T2); `--body -` (E2-F2-T3); `--session` and the reference (E2-F3-T5); the created file's layout and ID allocation (E1-F6); the busy exit (E2-F3-T1).
- Tests first:
  1. `fr13_new_creates_card_from_feature_skeleton`: given the sandbox (work folder absent), when `brain-swap new migrate invoices to v13` runs, then it exits 0, stdout is `created W-1: migrate invoices to v13 (Todo, work)`, and the work folder holds `W-1.md` (insta snapshot: `column: Todo`, `template: Feature`, `created: 2026-10-08T08:52:00+00:00`, `# migrate invoices to v13`, `## Goal`, `## Acceptance`, `## Context`, an empty `## Timeline`) and `board.md` reading `---`, `next: 2`, `---`.
  2. `ts6_3_new_joins_title_words`: given the sandbox, when `brain-swap new "  fix" rounding` runs, then stdout is `created W-1: fix rounding (Todo, work)`.
  3. `ts6_3_new_without_title_exits_1`: given the sandbox, when `brain-swap new` and `brain-swap --json new` run, then each exits 1, the first prints `brain-swap: no title` on stderr, the second the envelope with code `invalid_input` on stdout, and no card file exists (two snapshots).
  4. `ts6_7_new_json_returns_card`: given the sandbox, when `brain-swap --json new x` runs, then stdout is `{"v":1,"card":{"id":"W-1","board":"work","title":"x","column":"Todo","open":true,"path":"<sandbox>/boards/work/W-1.md","latest_note":null}}` (insta JSON snapshot of the redacted stdout).
  5. `as9_new_on_first_run_writes_config_and_card`: given `Sandbox::new()` (no config), when `brain-swap new first card` runs, then it exits 0, stderr's first line is `created config <XDG_CONFIG_HOME>/brain-swap/config.toml`, that file exists, and the work folder named in it holds `W-1.md`; given a second `Sandbox::new()`, when `brain-swap --json new first card` runs, then it exits 0, stdout is one object and stderr's first line is `warning: created config <XDG_CONFIG_HOME>/brain-swap/config.toml`.
  6. `ts4_3_new_with_invalid_board_md_exits_1_unreadable`: given a work folder whose `board.md` is `---\nletter W\n---\n`, when `brain-swap new x` runs, then it exits 1, stderr is `brain-swap: board.md:2: <problem>` (snapshot), the JSON variant has code `unreadable`, no card file exists and `board.md` is byte-identical.
  7. `ts6_2_new_on_board_path_that_is_a_file_exits_1_io`: given the work board path created as a regular file, when `brain-swap new x` runs, then it exits 1 and the JSON variant has code `io` (text and JSON snapshots).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/cli/` renders no card file bytes: the file comes only from `core::store`.

### E2-F2-T2 `new --template`, `--column`, `--board` and unknown names

- Status: todo
- Depends on: E2-F2-T1
- Covers: FR-13; TECHSPEC 6.1 (`--board`), 6.2 (exit 3, code `not_found`, the three texts, names matched ignoring case), 6.3 (`new` flags), I3
- Size: M
- Scope: `Ctx::board(Some(name))`, `cmd::find_template` and `cmd::find_column` delegate to `Config::board` (E1-F3), `core::template::find` (E1-F4) and `Board::column` (E1-F5) and build no text of their own. Those E1 functions match ignoring case, return the canonical names, and fail `Error::NotFound` with the exact 6.2 texts: `unknown board 'work2' (work, home, personal)` (config order), `unknown template 'epic' (Feature, Bug, Research, Chore)` (E1-F4 order), `unknown column 'Doign' on work (Todo, Doing, Done)` (`board.md` order); E2 passes the error through `Out::error` unchanged. Only configured columns are targets; an extra column such as `Doign` is unknown (4.7). `new` uses `--template`, `--column` and the global `--board`.
- Not in scope: building the three not-found texts (E1-F3-T1, E1-F4-T3, E1-F5-T4); card lookup (E2-F2-T4); `move`'s column (E2-F2-T6); the TUI's unknown `--board` (E4-F1-T8).
- Tests first:
  1. `ts6_2_unknown_template_exits_3_listing_names`: given the sandbox, when `brain-swap new --template epic x` runs, then it exits 3, stderr is `brain-swap: unknown template 'epic' (Feature, Bug, Research, Chore)`, no card file exists, and the `--json` variant prints the envelope with code `not_found` (two snapshots).
  2. `ts6_2_unknown_column_exits_3_listing_columns`: given the sandbox, when `brain-swap new --column Doign x` runs, then it exits 3 with `brain-swap: unknown column 'Doign' on work (Todo, Doing, Done)` (text and JSON snapshots).
  3. `ts6_2_unknown_board_exits_3_listing_boards`: given the sandbox, when `brain-swap --board work2 new x` runs, then it exits 3 with `brain-swap: unknown board 'work2' (work, home, personal)` (text and JSON snapshots).
  4. `i3_new_template_and_column_flags`: given the sandbox, when `brain-swap new --template bug --column Doing x` runs, then stdout is `created W-1: x (Doing, work)` and `W-1.md` holds `column: Doing`, `template: Bug` and the headings `## Symptom`, `## Expected`, `## Repro`, `## Context`.
  5. `ts6_2_names_match_ignoring_case`: given the sandbox, when `brain-swap --board WORK new --template BUG --column doing x` runs, then stdout is `created W-1: x (Doing, work)` and the file holds `column: Doing` and `template: Bug`.
  6. `ts6_3_new_on_named_board_uses_its_letter`: given the sandbox, when `brain-swap new x --board home` runs, then stdout is `created H-1: x (Todo, home)` and `H-1.md` is in the home folder.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The three not-found texts are built only in `Config::board`, `core::template::find` and `Board::column` (E1): a grep of `src/cli/` finds no `unknown board`, `unknown template` or `unknown column` literal.

### E2-F2-T3 `new --body -`

- Status: todo
- Depends on: E2-F2-T1
- Covers: FR-13; TECHSPEC 6.3 (`--body -`), 5.4 (`## Timeline` dropped with a warning), R4
- Size: M
- Scope: `--body` accepts only `-`; another value is a usage error, exit 2 (A-E2-10). With `--body -`, `cmd::split_body` reads stdin and only takes the title line: the first non-blank line, if it starts with `# `, is removed and its rest, trimmed, is the title; a positional title wins and the line is still removed; the rest is the body handed to the store instead of the skeleton. A `## Timeline` line in the body is left to E1-F2's `render_new`, which drops it and reports `## Timeline in body dropped` in `Created.warnings`; `new` prints every `Created.warnings` entry through `Out::warn`, so the created card has exactly one `# ` title line and one timeline (R4). No title from either source exits 1 as in E2-F2-T1.
- Not in scope: skeleton bodies (E2-F2-T1); dropping `## Timeline` and its warning text (E1-F2-T6); the pack's heredoc (E3-F1).
- Tests first:
  1. `fr13_new_body_from_stdin_replaces_skeleton`: given stdin `# migrate invoices\n\n## Goal\nswitch reads\n`, when `brain-swap new --body -` runs, then stdout is `created W-1: migrate invoices (Todo, work)` and `W-1.md` matches an insta snapshot with that title, `## Goal`, `switch reads`, no `## Acceptance` or `## Context`, and one empty `## Timeline` (the snapshot without a positional title).
  2. `ts6_3_new_positional_title_wins_and_heading_removed`: given the same stdin, when `brain-swap new --body - v13 migration` runs, then stdout is `created W-1: v13 migration (Todo, work)`, the file holds no `# migrate invoices` line, and it matches an insta snapshot (the snapshot with a positional title).
  3. `ts6_3_new_body_title_is_first_nonblank_line`: given stdin `\n\n# t\nrest\n`, when `brain-swap new --body -` runs, then the title is `t` and the body is `rest`.
  4. `ts6_3_new_body_later_heading_is_not_title`: given stdin `intro\n# t\n` and no positional title, when `brain-swap new --body -` runs, then it exits 1 with `brain-swap: no title` and no card file exists.
  5. `r4_new_body_drops_timeline_heading_with_warning`: given stdin `# t\n## Timeline\n### 2026-10-08T10:00:00+03:00\n- Doing: x\n`, when `brain-swap new --body -` runs, then stderr holds `warning: ## Timeline in body dropped`, `W-1.md` has exactly one `## Timeline` line and one `# ` line, and E1-F5 loads it with zero notes.
  6. `ts6_3_new_body_empty_stdin_keeps_positional_title`: given empty stdin, when `brain-swap new --body - x` runs, then `W-1.md` has title `x`, no template field heading and an empty `## Timeline`.
  7. `ts6_3_new_body_flag_accepts_only_dash`: given the sandbox, when `brain-swap --json new --body notes.md x` runs, then it exits 2 with code `usage`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Both `--body -` snapshots are reviewed with `cargo insta review` and committed (12.2 item 3).

### E2-F2-T4 Card lookup and the `show` block

- Status: todo
- Depends on: E2-F2-T1
- Covers: TECHSPEC 6.1 (card IDs resolve by letter), 6.2 (exit 3 for a card), 6.3 (`show`)
- Size: M
- Scope: `cmd::find_card` wraps E1-F5's `core::board::find_card` (E1-F5-T4), resolving an ID over the configured boards (by letter, whatever `--board` says); that function's error is already `Error::NotFound("<ID> not found")`, which `find_card` passes through unchanged for an unknown number or a letter no board has; it builds no text of its own (A-E2-07). `out::card_block` renders in fixed order: `card: <ID> <title>`; `board: <board>, column: <column>`; `latest note: <stamp>, <age> ago[, auto], <ending>` or `latest note: none`; then `Doing: `, `Next: ` and `Watch out: ` lines, and `Where: <cwd> (pane <pane>)`. The stamp is printed with the section 3 strftime; a bad stamp prints the heading's text and `? ago`; age `now` prints `now` without ` ago` (A-E2-15); `auto` comes before the ending (A-E2-14); continuation lines are indented two spaces, an empty part is its bare label, no pane drops ` (pane ...)`, and a note without a place prints `Where: none` (A-E2-16). `SessionView::Unresolved` gives the endings `by session <id>` and `no session`, since `show` resolves no session. `show <ID>` prints the block. Create `tests/fixtures/boards/work/`: the 4.9 `board.md`, `W-9.md` and `W-12.md` with each `cwd` rewritten to `@SANDBOX@/brain-swap` and `@SANDBOX@/ati.billing`; `W-11.md` (column Todo, template Bug, created `2026-10-08T09:00:00+03:00`, `# invoice PDF shows the wrong VAT`, the Bug fields, an empty `## Timeline`); and `W-7.md` (`# fix rounding in act export`, column Doing, template Bug, created `2026-10-07T15:00:00+03:00`, one note `### 2026-10-08T09:40:00+03:00 auto` with Doing `rounding fix drafted`, Next `check the 0.005 case`, Watch out `nothing`, place `cwd=@SANDBOX@/acts session=7c0ffee0-0000-4000-8000-000000000007`).
- Not in scope: `--all` and the note JSON (E2-F2-T5); the `by this session` and `by another session` endings (E2-F3-T7, E2-F3-T8); note parsing (E1-F2); age arithmetic (E1-F1).
- Tests first:
  1. `ts6_3_show_prints_block_from_card_line`: given the `work` fixture, when `brain-swap show W-12` runs, then it exits 0 and stdout is exactly `card: W-12 migrate invoices to v13`, `board: work, column: Doing`, `latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by session 55583127-a22a-49bc-a803-e777c377a595`, `Doing: migration test passes on staging`, `Next: open the MR, ask the DBA for the v13 deploy list`, `Watch out: nothing new`, `Where: <sandbox>/ati.billing (pane w4V:p9)` (insta snapshot).
  2. `ts6_3_show_card_without_notes`: given the fixture, when `brain-swap show W-11` runs, then the third line is `latest note: none` and it is the last line.
  3. `ts6_3_show_note_without_pane_or_session`: given the fixture, when `brain-swap show W-9` runs, then it prints `latest note: 2026-10-06T17:05:12+03:00, 1 d ago, no session` and `Where: <sandbox>/brain-swap`.
  4. `ts6_3_show_auto_note`: given the fixture, when `brain-swap show W-7` runs, then it prints `latest note: 2026-10-08T09:40:00+03:00, 2 h ago, auto, by session 7c0ffee0-0000-4000-8000-000000000007`.
  5. `ts6_3_block_age_now_and_bad_stamp` (unit, `out::card_block`): given a note stamped 20 seconds before now, then its line starts `latest note: <stamp>, now, `; given a note headed `### yesterday` (`at` none), then its line starts `latest note: yesterday, ? ago, `.
  6. `ts6_3_block_continuation_and_missing_place` (unit, `out::card_block`): given a note whose Next is `a` continued by `b`, an empty Watch out and no place, then the block holds `Next: a`, `  b`, `Watch out:` and `Where: none`.
  7. `ts6_1_show_resolves_board_by_letter`: given the fixture as work and an empty home board, when `brain-swap --board home show w-12` runs, then stdout equals test 1's.
  8. `ts6_2_show_unknown_card_exits_3`: given the fixture, when `brain-swap show W-99` and `brain-swap show Z-1` run, then each exits 3 with `brain-swap: W-99 not found` and `brain-swap: Z-1 not found`, and the `--json` variants carry code `not_found` (snapshots).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `card_block` is the only renderer of the block; `SessionView::Resolved` is accepted already, so `link` and `context` (E2-F3) reuse it unchanged.

### E2-F2-T5 `show --all` and the note JSON

- Status: todo
- Depends on: E2-F2-T4
- Covers: TECHSPEC 6.3 (`show --all`, `by_this_session` null), 6.7 (note, `show`)
- Size: S
- Scope: `show --all` prints the block, then each older note, newest first, as a blank line and a block headed `note <n>: <stamp>, <age> ago[, auto], <ending>` with its parts and `Where:` line, n counted from the oldest as 1 (A-E2-13). `show --json` returns `card` and `notes` (the latest only, or every note newest first with `--all`). `out::note_json` renders `at` (RFC 3339 as written, null for a bad stamp), `age_min` (floored minutes, 0 for a future stamp), `auto`, `doing`, `next`, `watch_out`, `by_this_session` and `place` (`cwd`, `session`, `herdr` with `pane`, `tab`, `workspace`, or null); `by_this_session` is null under `SessionView::Unresolved` and when the note has no session (A-E2-21). `card_json`'s `latest_note` now uses it.
- Not in scope: `by_this_session` true or false (E2-F3-T7, E2-F3-T8).
- Tests first:
  1. `ts6_3_show_all_adds_older_notes_newest_first`: given the `work` fixture, when `brain-swap show W-12 --all` runs, then stdout is test 1's block of E2-F2-T4, a blank line, `note 1: 2026-10-08T10:31:05+03:00, 1 h ago, auto, by session 55583127-a22a-49bc-a803-e777c377a595` and that note's three parts and `Where:` line (insta snapshot).
  2. `ts6_7_show_json_card_and_latest_note`: given the fixture, when `brain-swap --json show W-12` runs, then `card.latest_note` has `at` `2026-10-08T11:12:40+03:00`, `age_min` 39, `auto` false, `by_this_session` null and `place.herdr.pane` `w4V:p9`, and `notes` holds that one note (insta JSON snapshot).
  3. `ts6_7_show_all_json_notes_newest_first`: given the fixture, when `brain-swap --json show W-12 --all` runs, then `notes` holds two notes whose `at` values are `2026-10-08T11:12:40+03:00` then `2026-10-08T10:31:05+03:00` (insta JSON snapshot).
  4. `ts6_7_note_json_absent_values_null` (unit, `out::note_json`): given W-9's note, then `place.herdr` and `place.session` are null and `by_this_session` is null; given a bad-stamp note, then `at` and `age_min` are null.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Note numbering in `show --all` counts from the oldest, matching `locate --note` (6.3).

### E2-F2-T6 `move`

- Status: todo
- Depends on: E2-F2-T2, E2-F2-T4, E1-F6
- Covers: FR-39; TECHSPEC 6.2 (exit 0 for the current column, exit 1 `unreadable`, exit 3), 6.3 (`move`), 6.7 (`move`)
- Size: M
- Scope: `cmd::move_card`: find the card (E2-F2-T4), match the column ignoring case against the board's configured columns (E2-F2-T2), and call the store's move, which changes only the `column:` line. A move to the card's current column writes nothing and exits 0 with the same output (A-E2-12). Prints `moved <ID> to <Column>` with canonical names; `--json` returns `card`. A card the store refuses (conflict markers, non-UTF-8, invalid frontmatter) exits 1 with code `unreadable` and writes nothing.
- Not in scope: the splice and the three-line frontmatter for a card without one (E1-F6, R2); TUI moves (E4-F2).
- Tests first:
  1. `fr39_move_changes_only_column_line`: given the `work` fixture, when `brain-swap move W-12 Done` runs, then it exits 0, stdout is `moved W-12 to Done`, `W-12.md` differs from the fixture in exactly one line (`column: Done`), and every other file is byte-identical.
  2. `ts6_3_move_matches_column_ignoring_case`: given the fixture, when `brain-swap move W-11 doing` runs, then stdout is `moved W-11 to Doing` and the file holds `column: Doing`.
  3. `ts6_1_move_card_id_case_insensitive`: given the fixture, when `brain-swap move w-12 Done` runs, then stdout is `moved W-12 to Done`.
  4. `ts6_2_move_to_current_column_exits_0_writes_nothing`: given the fixture, when `brain-swap move W-12 Doing` runs, then it exits 0, stdout is `moved W-12 to Doing`, and `W-12.md`'s bytes and mtime are unchanged.
  5. `ts6_2_move_to_extra_column_exits_3`: given the fixture plus `W-20.md` holding `---\ncolumn: Doign\n---\n# spike on caching\n`, when `brain-swap move W-12 Doign` runs, then it exits 3 with `brain-swap: unknown column 'Doign' on work (Todo, Doing, Done)` and `W-12.md` is unchanged (text and JSON snapshots).
  6. `ts6_2_move_unknown_card_exits_3`: given the fixture, when `brain-swap move W-99 Done` runs, then it exits 3 with `brain-swap: W-99 not found` (text and JSON snapshots).
  7. `ts6_2_move_unreadable_card_exits_1`: given `W-12.md` with a line `<<<<<<< HEAD` inserted after its title, when `brain-swap move W-12 Done` runs, then it exits 1, the `--json` variant has code `unreadable`, and the file is unchanged (snapshots).
  8. `ts6_7_move_json_returns_card`: given the fixture, when `brain-swap --json move W-11 Doing` runs, then `card` has `id` `W-11`, `column` `Doing`, `open` true and `latest_note` null (insta JSON snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] A move to the current column writes nothing: no file in the board folder changes or appears (test 4 also compares the folder listing).

### E2-F2-T7 `ls` and `ls --open`

- Status: todo
- Depends on: E2-F2-T2, E2-F2-T5
- Covers: TECHSPEC 4.7 (extra columns), 5.2 (unknown keys warn), 6.2 (exit 1 `io`), 6.3 (`ls`, picker line), 6.7 (`ls`), 11 (`BRAIN_SWAP_LOG` inside a board folder, as printed)
- Size: M
- Scope: `out::picker_line` renders `<n>. <ID> <title> [<column>] <age>[, <label>]: <preview>`, or `<n>. <ID> <title> [<column>] no note[, <label>]`. `out::preview` takes the first line of the latest note's Next, else Doing, cut at the last word boundary within 40 characters (Unicode scalar values); one longer word is cut at 40; an empty preview drops `: <preview>` (A-E2-17). `ls` prints `cards on <board>:`, then every card numbered from 1 in column order (configured columns, then extra columns in order of first appearance, shown as written), newest activity first within a column, without labels; `--open` keeps only open cards under the same header (A-E2-18). `--json` returns `board` and `cards` (Card objects). Board load warnings, the config loader's warnings and `Ctx.warnings` (the E2-F1-T4 log guard) go to stderr through `Out::warn` (A-E2-32); a board folder that cannot be read exits 1 with code `io`.
- Not in scope: `--guess`, labels and `o. other board:` (E2-F2-T8); `context`'s picker (E2-F3-T9); ordering rules (E1-F5).
- Tests first:
  1. `ts6_3_ls_lists_cards_in_column_order`: given the `work` fixture, when `brain-swap ls` runs, then stdout is exactly `cards on work:`, `1. W-11 invoice PDF shows the wrong VAT [Todo] no note`, `2. W-12 migrate invoices to v13 [Doing] 39 min: open the MR, ask the DBA for the v13`, `3. W-7 fix rounding in act export [Doing] 2 h: check the 0.005 case`, `4. W-9 bump rust-version to 1.89 [Done] 1 d: nothing left` (insta snapshot).
  2. `ts6_3_ls_open_omits_done_column`: given the fixture, when `brain-swap ls --open` runs, then stdout is `cards on work:` followed by lines 1 to 3 of test 1 (insta snapshot).
  3. `ts6_3_picker_preview_cut_at_word_boundary` (unit, `out::preview`): given Next `open the MR, ask the DBA for the v13 deploy list`, then the preview is `open the MR, ask the DBA for the v13`; given a Next of exactly 40 characters, then it is unchanged; given one 50-character word, then its first 40 characters; given `ü` repeated 45 times, then `ü` repeated 40 times.
  4. `ts6_3_picker_preview_falls_back_to_doing` (unit, `out::preview` and `out::picker_line`): given a latest note with empty Next and Doing `a` continued by `b`, then the preview is `a`; given a latest note with only Watch out, then the picker line ends with the age and holds no `: `.
  5. `ts4_7_ls_extra_column_after_configured`: given the fixture plus `W-20.md` holding `---\ncolumn: Doign\n---\n# spike on caching\n`, when `brain-swap ls` runs, then the last line is `5. W-20 spike on caching [Doign] no note` and stderr holds E1-F5's unknown-column warning as a `warning: ` line.
  6. `ts6_3_ls_board_flag_after_subcommand`: given the sandbox with no home folder, when `brain-swap ls --board home` runs, then it exits 0, stdout is only `cards on home:` and no home folder is created.
  7. `ts6_7_ls_json_board_and_cards`: given the fixture, when `brain-swap --json ls` runs, then stdout has `board` `work` and `cards` holding four Card objects in test 1's order (insta JSON snapshot).
  8. `ts5_2_unknown_config_key_warns_on_stderr`: given the sandbox config plus a top-level `colour = "red"`, when `brain-swap ls` runs, then it exits 0 and stderr holds one `warning: ` line naming `colour` (snapshot).
  9. `ts6_2_ls_unreadable_board_folder_exits_1_io`: given the work board path created as a regular file, when `brain-swap ls` runs, then it exits 1 and the `--json` variant has code `io` (text and JSON snapshots).
  10. `ts11_log_inside_board_folder_ignored_on_cli`: given the sandbox and `BRAIN_SWAP_LOG=<root>/boards/work/bs.log`, when `brain-swap ls` runs, then it exits 0, stderr holds `warning: BRAIN_SWAP_LOG inside board work, ignored`, and `<root>/boards/work/bs.log` does not exist.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `picker_line` and `preview` are shared by `ls`, `ls --guess` and `context` (no second renderer).

### E2-F2-T8 `ls --guess`

- Status: todo
- Depends on: E2-F2-T7, E1-F8
- Covers: FR-38; TECHSPEC 6.3 (`ls --guess`), 6.6 (labels, order as printed, current pane only when `HERDR_ENV=1`), 6.7 (Candidate, `other_boards`)
- Size: M
- Scope: `ls --guess`, with or without `--open` (A-E2-18), passes the open cards, the current pane (`HERDR_PANE_ID`, only when `HERDR_ENV=1`), the cwd and the current pane's hint to E1-F8 and prints `open cards on <board>, best guess first:`, the picker lines with labels `linked here`, `same pane` and `same folder` (none for tiers 3 and 4), then `o. other board: <names>` with the other configured boards in config order, omitted when there is none (A-E2-19). `out::other_boards_line` is shared with `context`. `--json` returns `cards` as Candidates (Card plus `rank`, the 1-based printed position, A-E2-20, and `reason`: `linked_here`, `same_pane`, `same_folder`, `middle_column` or `other`) and `other_boards`. No herdr command runs.
- Not in scope: tier rules and ties (E1-F8); writing pane hints (E2-F3-T5); `context`'s picker (E2-F3-T9).
- Tests first:
  1. `fr38_ls_guess_orders_open_cards_by_tier`: given the `work` fixture, the default cwd `<sandbox>/elsewhere` and no `HERDR_*`, when `brain-swap ls --open --guess` runs, then stdout is exactly `open cards on work, best guess first:`, `1. W-12 migrate invoices to v13 [Doing] 39 min: open the MR, ask the DBA for the v13`, `2. W-7 fix rounding in act export [Doing] 2 h: check the 0.005 case`, `3. W-11 invoice PDF shows the wrong VAT [Todo] no note`, `o. other board: home, personal` (insta snapshot).
  2. `fr38_ls_guess_same_folder_label`: given cwd `<sandbox>/acts/src` (inside W-7's note cwd), when `brain-swap ls --open --guess` runs, then line 2 of stdout is `1. W-7 fix rounding in act export [Doing] 2 h, same folder: check the 0.005 case`.
  3. `fr38_ls_guess_same_pane_label`: given `HERDR_ENV=1` and `HERDR_PANE_ID=w4V:p9`, when `brain-swap ls --open --guess` runs, then line 2 is `1. W-12 migrate invoices to v13 [Doing] 39 min, same pane: open the MR, ask the DBA for the v13`.
  4. `ts6_6_pane_ignored_outside_herdr`: given `HERDR_PANE_ID=w4V:p9` without `HERDR_ENV`, when `brain-swap ls --open --guess` runs, then stdout equals test 1's.
  5. `fr38_ls_guess_linked_here_from_pane_hint`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p11` and a pane hint for `w4V:p11` naming W-11, written through E1-F7, when `brain-swap ls --open --guess` runs, then line 2 is `1. W-11 invoice PDF shows the wrong VAT [Todo] no note, linked here`.
  6. `ts6_3_ls_guess_implies_open`: given the fixture, when `brain-swap ls --guess` runs, then stdout equals test 1's.
  7. `ts6_3_other_board_line_omitted_with_one_board`: given a config with only `work`, when `brain-swap ls --open --guess` runs, then no line starts with `o. `.
  8. `ts6_7_ls_guess_json_candidates`: given the fixture, when `brain-swap --json ls --open --guess` runs, then `cards` holds W-12, W-7 and W-11 with `rank` 1, 2, 3 and `reason` `middle_column`, `middle_column`, `other`, and `other_boards` is `["home","personal"]` (insta JSON snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `ls --guess` reads `HERDR_PANE_ID` from `Env` only and calls no `Runner`.

## E2-F3 park, link, context

- Depends on: E2-F2, E1-F7, E5-F1
- Covers: FR-15, FR-16, FR-18, FR-34, FR-37, FR-41, FR-56, FR-57
- Spec: TECHSPEC 6.1 (`BRAIN_SWAP_FAILPOINT`), 6.2 (exit 1 `invalid_input` and `unreadable`, exits 3, 4 and 5), 6.3 (`park`, `link`, `context`, reference setting), 6.4, 6.5, 6.7 (`park`, `link`, `context`), 9.1 and 9.2 (as called), 10 (lock, fresh reads), 12.1 (CLI, crash, concurrency), AS-3 (CLI half), AS-6 and AS-8 (CLI steps)
- Scope: The session-aware subcommands. `park` reads a note from stdin (6.5) or prompts on a terminal, captures the place (cwd, herdr pane, tab and workspace, session) and appends it to the card named by `--card` or by the session's reference, exiting 4 without either. One resolver implements 6.4 for `new`, `park`, `link` and `context`, with herdr's pane session (9.2) as step 3; `new`, `park --card` and `link` set the reference and, inside herdr, the pane hint. `link` prints `linked to <ID>: <title>` and the card block. `context` prints the fixed v1 format with the reference's card and latest note, or the best-guess picker, and turns every expected problem into lines with exit 0. Process-level crash tests (`create:*`, `park:*`) and concurrency tests run the real binary.
- Provides: `cli::cmd::NoteText { doing: String, next: String, watch_out: String }`, `cli::cmd::parse_note(text: &str) -> NoteText` (6.5), `cli::cmd::read_note(input: &mut dyn BufRead, prompts: &mut dyn Write, terminal: bool) -> Result<NoteText, Error>`, `cli::cmd::resolve_session(env: &Env, flag: Option<&str>, runner: &dyn Runner) -> (Option<String>, Vec<String>)` (6.4 steps 1 to 4 and their warnings), the `park`, `link` and `context` contracts of 6.3 and 6.7 that E3-F1's skills and E4-F5's scripted editor (`park --session test`) call, `tests/fixtures/boards/pair/` (W-1 and W-2 in Todo)
- Requires: E1-F6 `core::store` park (append, verify, board lock with `Error::Busy` after 2 s, `core::store::lock_board` and `core::store::lock_path(env, board_dir)` (E1-F6-T1) for the busy test, failpoints `create:after_next`, `create:after_tmp`, `create:after_link`, `park:after_tmp` with process abort); E1-F7 `core::session` (I8 validation, steps 1, 2 and 4 with the warnings `session id not substituted` and `invalid session id`, reference read and write with `card`, `board`, `pane`, `cwd`, `set_at`, pane hints); E5-F1 `adapters::runner::Runner`, `adapters::runner::ProcessRunner`, `adapters::herdr::capture(env, runner)` (A-E5-02), `adapters::herdr::pane_session`, the fake herdr script driven by `FAKE_HERDR_SCENARIO` with its argv log and the scenarios `pane-session` (`pane get w4V:p9` answers `agent_session.value` `herdr-s1`) and `pane-no-agent`, the SP-4 moved-pane check of E5-F1-T9 when the TECHSPEC 16 SP-4 line enabled it; E1-F8 best guess; E2-F2 `find_card`, `card_block`, `SessionView`, `picker_line`, `other_boards_line`, `card_json`, `note_json`
- Feature DoD:
  - [ ] Exit 5 `busy` produced by a test (E2-F3-T1).
  - [ ] AS-6 CLI step: `park --card` from stdin writes a note without `auto` (E2-F3-T1).
  - [ ] `new --session ""` inside a fake herdr pane sets the reference under the herdr session (E2-F3-T5).
  - [ ] Exit 4 `no_reference` produced by a test (E2-F3-T5).
  - [ ] AS-6 CLI step: `park --card` from a terminal prompts for the three parts and writes no `auto` (E2-F3-T6, test 6 through a pty).
  - [ ] `context` pinned with a reference, with and without notes (E2-F3-T8).
  - [ ] `context` pinned with no reference and with a reference to a deleted card (E2-F3-T9).
  - [ ] `context` pinned with no config, a broken config, a missing folder, an empty and a placeholder session, always exit 0 (E2-F3-T10).
  - [ ] The process-level 12.1 crash tests (`create:after_next`, `create:after_tmp`, `create:after_link`, `park:after_tmp`) pass (E2-F3-T11).
  - [ ] The process-level 12.1 concurrency test passes (E2-F3-T12).
  - [ ] AS-8 CLI half: two parallel parks on different cards and on one card (E2-F3-T12).

### E2-F3-T1 `park --card` from stdin and its error exits

- Status: todo
- Depends on: E2-F2-T5, E1-F6
- Covers: FR-16, FR-34, FR-15; TECHSPEC 6.2 (exit 1 `unreadable`, exit 3, exit 5 `busy`), 6.3 (`park` output), 6.7 (`park`), 10 (lock), AS-6 (stdin step)
- Size: M
- Scope: `cmd::park` with `--card`: find the card (E2-F2-T4), read stdin (not a terminal) through a first `parse_note` that recognises plain `Doing:`, `Next:` and `Watch out:` lines, build the note with `at` = `env.now` (already in the local zone, E1-F1-T6), `auto` false and a place holding only the cwd, append it through the store, and print `parked to <ID>: <title>`; `--json` returns `card`. An unknown `--card` exits 3 (E1-F5's `<ID> not found`); a card the store refuses (conflict markers, non-UTF-8) exits 1 with code `unreadable`; a board lock not obtained within 2 seconds exits 5 with code `busy`. Outside herdr no herdr command runs.
- Not in scope: the full 6.5 grammar and `empty note` (E2-F3-T2); `--auto`, herdr capture and the session in the place (E2-F3-T3); session resolution (E2-F3-T4); references and exit 4 (E2-F3-T5); prompts (E2-F3-T6); the splice and the lock itself (E1-F6).
- Tests first:
  1. `fr16_park_appends_note_only_to_timeline`: given the `work` fixture, cwd `<sandbox>/ati.billing` and stdin `Doing: a\nNext: b\nWatch out: c\n`, when `brain-swap park --card W-12` runs, then it exits 0, stdout is `parked to W-12: migrate invoices to v13`, `W-12.md` equals the fixture bytes followed by `\n### 2026-10-08T08:52:00+00:00\n- Doing: a\n- Next: b\n- Watch out: c\n<!-- where: cwd=<sandbox>/ati.billing -->\n` (insta snapshot), and every other file is byte-identical.
  2. `as6_park_from_stdin_writes_no_auto`: given the input of test 1, when the park runs, then the new heading line is exactly `### 2026-10-08T08:52:00+00:00` and the `column: Doing` line is unchanged.
  3. `fr15_park_outside_herdr_records_cwd_only`: given no `HERDR_*`, no `--session`, no `BRAIN_SWAP_SESSION` and cwd `<sandbox>/x`, when `brain-swap park --card W-11` runs with stdin `Doing: a`, then the place line is exactly `<!-- where: cwd=<sandbox>/x -->`.
  4. `ts6_1_park_card_id_case_insensitive`: given the fixture, when `brain-swap park --card w-12` runs with the stdin of test 1, then stdout is `parked to W-12: migrate invoices to v13`.
  5. `ts6_7_park_json_returns_card`: given the fixture, when `brain-swap --json park --card W-12` runs with the stdin of test 1, then `card.latest_note` has `doing` `a`, `next` `b`, `watch_out` `c`, `auto` false, `age_min` 0, `by_this_session` null and `place.herdr` null (insta JSON snapshot).
  6. `ts6_2_park_unknown_card_exits_3`: given the fixture, when `brain-swap park --card W-99` runs with stdin `Doing: a`, then it exits 3 with `brain-swap: W-99 not found` and no file changes (text and JSON snapshots).
  7. `ts6_2_park_unreadable_card_exits_1`: given `W-12.md` with a `<<<<<<< HEAD` line inserted after its title, and separately `W-11.md` with a byte `0xFF` appended, when `brain-swap park --card <that card>` runs with stdin `Doing: a`, then each exits 1, the `--json` variant has code `unreadable`, and the file is byte-identical (snapshots).
  8. `ts6_2_park_busy_exits_5`: given the fixture and the test process holding `core::store::lock_board(&env, &<root>/boards/work)`, where `env` is built with `Env::from_vars` over the sandbox Scenario's `vars()`, so its state folder is `<root>/state` and `core::store::lock_path(env, board_dir)` (E1-F6-T1) names the same lock file as the child's, released by drop after the assertions, when `brain-swap park --card W-12` runs with stdin `Doing: a`, then it exits 5 after at least 1.9 seconds, the `--json` variant has code `busy`, and `W-12.md` is byte-identical (text and JSON snapshots).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The busy test releases its lock in a drop guard, so a failing assertion cannot leave it held for other tests.

### E2-F3-T2 Note grammar on stdin and `empty note`

- Status: todo
- Depends on: E2-F3-T1
- Covers: FR-18, FR-34; TECHSPEC 6.5
- Size: S
- Scope: Complete `cli::cmd::parse_note` to 6.5, hand-written without a regex crate: a line matching `^\s*-?\s*(\*\*)?(doing|next|watch out|watch)\s*:\s*(\*\*)?\s*(.*)$` in any case starts that part with the captured rest; other non-empty lines continue the current part on a new line; unlabelled lines before any label belong to Doing; empty lines are skipped; a repeated label continues its part on a new line (A-E2-24). Three empty parts are `Error::InvalidInput` `empty note`: exit 1, nothing written.
- Not in scope: writing continuation lines (E1-F2's writer); terminal prompts (E2-F3-T6); `/bs-park`'s argument labels (E3-F1).
- Tests first: (unit, `cli::cmd::parse_note`, unless marked CLI)
  1. `ts6_5_labels_start_parts`: given `Doing: a\nNext: b\nWatch out: c`, then Doing, Next and Watch out are `a`, `b` and `c`.
  2. `ts6_5_dash_bold_and_spacing_variants`: given `- Doing: a\n**Next:** b\n  watch out :  c`, then the parts are `a`, `b` and `c`.
  3. `ts6_5_labels_match_any_case`: given `DOING: a\nnExT: b\nWatch Out: c`, then the parts are `a`, `b` and `c`.
  4. `ts6_5_watch_alone_is_watch_out`: given `watch: timeout`, then Watch out is `timeout` and Doing and Next are empty.
  5. `ts6_5_unlabelled_leading_lines_are_doing`: given `rerun the next test\nNext: x`, then Doing is `rerun the next test` and Next is `x`.
  6. `ts6_5_label_word_without_colon_is_text`: given `next step is x`, then Doing is `next step is x`.
  7. `ts6_5_other_lines_continue_part`: given `Next: a\nb\n\nc`, then Next is `a\nb\nc`.
  8. `ts6_5_colon_after_closing_bold_is_text`: given `**Next**: x`, then Doing is `**Next**: x` and Next is empty.
  9. `ts6_5_repeated_label_continues_part`: given `Doing: a\nDoing: b`, then Doing is `a\nb`.
  10. `fr18_park_empty_note_exits_1` (CLI): given the `work` fixture, when `brain-swap park --card W-12` runs with stdin `Doing:\nNext:  \n`, and again with empty stdin, then each exits 1 with `brain-swap: empty note`, the `--json` variant has code `invalid_input`, and `W-12.md` is byte-identical (snapshots).
  11. `ts6_5_continuation_written_indented` (CLI): given the fixture, when `brain-swap park --card W-12` runs with stdin `Next: a\nb\n`, then the new note holds `- Next: a` followed by `  b`.
  12. `ts6_5_non_ascii_text_kept`: given `Doing: über\nwätch: x`, then Doing is `über\nwätch: x` (no label matches `wätch`) and no panic occurs.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `parse_note` matches labels on characters, never by byte index, so multi-byte text cannot split a UTF-8 character (test 12).

### E2-F3-T3 `--auto`, herdr capture and the session in the place

- Status: todo
- Depends on: E2-F3-T1, E5-F1, E1-F7
- Covers: FR-15; TECHSPEC 6.3 (`--auto`, capture), 9.1 (as called), T-15
- Size: S
- Scope: `park --auto` sets the note's `auto` marker. The place adds `adapters::herdr::capture(env, &runner)`, where `runner = ProcessRunner::new(env)`, the same runner E2-F3-T4 passes to `resolve_session` (pane, tab and workspace from `HERDR_*` when `HERDR_ENV=1`), and the session, taken until E2-F3-T4 through E1-F7's steps 1, 2 and 4 (a valid `--session`, else a valid `BRAIN_SWAP_SESSION`, else none). A valid session never calls herdr.
- Not in scope: the herdr pane-session fallback (E2-F3-T4); the moved-pane check inside `capture` (E5-F1, SP-4).
- Tests first:
  1. `ts6_3_park_auto_flag_marks_heading`: given the `work` fixture, when `brain-swap park --card W-12 --auto` runs with stdin `Doing: a`, then the new heading is `### 2026-10-08T08:52:00+00:00 auto`.
  2. `fr15_park_inside_herdr_records_pane_tab_workspace`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9`, `HERDR_TAB_ID=w4V:t1`, `HERDR_WORKSPACE_ID=w4V`, `HERDR_BIN_PATH` the fake script, `FAKE_HERDR_SCENARIO=pane-session` (so a moved-pane check gets an answer) and `--session s1`, when `brain-swap park --card W-12` runs with stdin `Doing: a`, then the place line is `<!-- where: cwd=<sandbox>/elsewhere pane=w4V:p9 tab=w4V:t1 workspace=w4V session=s1 -->` and the fake script's argv log is empty, or exactly `["pane get w4V:p9"]` when the TECHSPEC 16 SP-4 line enabled the moved-pane check (E5-F1-T9).
  3. `fr15_park_records_env_session`: given no `HERDR_*` and `BRAIN_SWAP_SESSION=s2`, when `brain-swap park --card W-12` runs with stdin `Doing: a`, then the place line is `<!-- where: cwd=<sandbox>/elsewhere session=s2 -->`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] I6 holds for every note `park` writes: `at`, `place.cwd` and a non-empty part (asserted by reloading the card in tests 1 to 3).

### E2-F3-T4 Session resolution for `new`, `park`, `link` and `context`

- Status: todo
- Depends on: E2-F3-T3, E5-F1
- Covers: FR-41; TECHSPEC 6.1 (`BRAIN_SWAP_LOG`), 6.4 (steps 1 to 4, warnings), 9.2 (as called), 11 (`BRAIN_SWAP_LOG` inside a board folder, with a herdr call)
- Size: M
- Scope: `cli::cmd::resolve_session(env, flag, runner)`: E1-F7's steps 1 and 2 with their warnings (`session id not substituted` for an empty or placeholder flag, `invalid session id` for another invalid one); then, only when neither gives a valid ID and `HERDR_ENV=1`, `adapters::herdr::pane_session` through the real `Runner` (step 3); else none (step 4). A missing flag resolves like an empty one without the warning. Nothing fails. `park` uses it now and prints its warnings through `Out::warn`; `new`, `link` and `context` call the same function (E2-F3-T5, E2-F3-T7, E2-F3-T8). The scenarios `pane-session` and `pane-no-agent` come from E5-F1.
- Not in scope: I8 validation and the warning texts (E1-F7); the herdr call and its 2 s timeout (E5-F1); reference setting (E2-F3-T5); warnings as `context` lines (E2-F3-T10).
- Tests first:
  1. `fr41_flag_session_wins_over_env`: given `--session a` and `BRAIN_SWAP_SESSION=b`, when `brain-swap park --card W-12` runs with stdin `Doing: x`, then the place has `session=a` and stderr is empty.
  2. `fr41_missing_flag_uses_env_without_warning`: given no flag and `BRAIN_SWAP_SESSION=b`, when the same park runs, then the place has `session=b` and stderr is empty.
  3. `fr41_empty_flag_warns_and_falls_back`: given `--session ""` and `BRAIN_SWAP_SESSION=b`, when the park runs, then stderr is `warning: session id not substituted` and the place has `session=b`.
  4. `fr41_placeholder_flag_warns_and_degrades_to_none`: given `--session '${CLAUDE_SESSION_ID}'`, no `BRAIN_SWAP_SESSION` and no `HERDR_*`, when the park runs, then it exits 0, stderr is `warning: session id not substituted`, and the place holds no `session=`.
  5. `fr41_invalid_flag_warns_invalid_session_id`: given `--session 'a b'` and no other source, when the park runs, then it exits 0 and stderr is `warning: invalid session id`.
  6. `fr41_inside_herdr_falls_back_to_pane_session`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9`, `FAKE_HERDR_SCENARIO=pane-session` and `--session ""`, when the park runs, then the place has `session=herdr-s1` and the argv log is exactly `["pane get w4V:p9"]`, or that call twice when the TECHSPEC 16 SP-4 line enabled the moved-pane check (E5-F1-T9), since `capture` then makes it too.
  7. `fr41_valid_env_session_never_calls_herdr`: given the herdr setup of test 6 and `BRAIN_SWAP_SESSION=b`, when the park runs without `--session`, then the argv log is empty, or exactly `["pane get w4V:p9"]` when the TECHSPEC 16 SP-4 line enabled the moved-pane check (E5-F1-T9).
  8. `fr41_pane_without_agent_session_resolves_none`: given `FAKE_HERDR_SCENARIO=pane-no-agent` and `--session ""` inside herdr, when the park runs, then it exits 0 and the place holds no `session=`.
  9. `ts6_1_brain_swap_log_receives_herdr_call`: given the setup of test 6 and `BRAIN_SWAP_LOG=<sandbox>/bs.log`, when the park runs, then `bs.log` holds one `herdr` line per entry of the argv log.
  10. `ts11_cli_ignores_log_inside_board_folder`: given the `work` fixture, `BRAIN_SWAP_LOG=<root>/boards/work/bs.log`, `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9` and `FAKE_HERDR_SCENARIO=pane-session`, when `brain-swap park --card W-12 --session ""` runs with stdin `Doing: x`, then it exits 0, the argv log is not empty, stderr holds `warning: BRAIN_SWAP_LOG inside board work, ignored`, and `<root>/boards/work/bs.log` does not exist.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `resolve_session` is the only place in `src/cli/` that calls `pane_session` (a grep finds one call).

### E2-F3-T5 Reference setting and `park` by reference

- Status: todo
- Depends on: E2-F3-T4, E1-F7
- Covers: FR-57, FR-41; TECHSPEC 6.2 (exit 4, code `no_reference`), 6.3 (`park` card resolution; `new` and `park --card` set the reference), 6.4 (reference file, pane hint), I9
- Size: M
- Scope: `new` and `park --card` resolve the session (E2-F3-T4) and, when one resolves, write its reference through E1-F7 (`card`, `board`, `pane` from capture, `cwd`, `set_at` = `env.now` (local zone, E1-F1-T6)) and, inside herdr, the pane hint; with no session they write neither and still succeed. `park` without `--card` takes the reference's card when it resolves (I9: board configured, card file present); otherwise `Error::NoReference` `no reference: pass --card <ID>` (A-E2-25), exit 4, before reading stdin, writing nothing. `park` through the reference leaves the reference file untouched (A-E2-31).
- Not in scope: `link` (E2-F3-T7); the reference file format, pruning and hint encoding (E1-F7).
- Tests first:
  1. `fr57_park_card_sets_reference`: given the `work` fixture and `--session s1` outside herdr, when `brain-swap park --card W-12` runs with stdin `Doing: x`, then the reference for `s1` read through E1-F7 has card `W-12`, board `work`, cwd `<sandbox>/elsewhere`, `set_at` `2026-10-08T08:52:00+00:00` and no pane.
  2. `fr57_park_by_reference_needs_no_card`: given the reference of test 1, when `brain-swap park --session s1` runs with stdin `Doing: y`, then it exits 0, stdout is `parked to W-12: migrate invoices to v13`, W-12 has two more notes than the fixture, and the reference file is byte-identical to its state after test 1's park.
  3. `ts6_2_park_without_card_or_reference_exits_4`: given no reference for `s9`, when `brain-swap park --session s9` runs with stdin `Doing: x`, then it exits 4 with `brain-swap: no reference: pass --card <ID>`, the `--json` variant has code `no_reference`, and no card file changes (text and JSON snapshots).
  4. `ts6_2_park_without_any_session_exits_4`: given no flag, no `BRAIN_SWAP_SESSION` and no `HERDR_*`, when `brain-swap park` runs with stdin `Doing: x`, then it exits 4.
  5. `i9_reference_to_deleted_card_exits_4`: given the reference of test 1 and `W-12.md` deleted, when `brain-swap park --session s1` runs, then it exits 4.
  6. `ts12_1_new_empty_session_in_fake_herdr_sets_reference`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9` and `FAKE_HERDR_SCENARIO=pane-session`, when `brain-swap new --session "" x` runs, then stderr holds `warning: session id not substituted` and the reference for `herdr-s1` has card `W-1`, board `work` and pane `w4V:p9`.
  7. `ts6_4_reference_inside_herdr_writes_pane_hint`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9` and `--session s1`, when `brain-swap park --card W-12` runs with stdin `Doing: x`, then the pane hint for `w4V:p9` read through E1-F7 names card W-12 and session `s1`.
  8. `ts6_3_new_with_session_sets_reference`: given the sandbox and `--session s1` outside herdr, when `brain-swap new x` runs, then the reference for `s1` has card `W-1` and no pane hint exists.
  9. `ts6_4_no_session_sets_no_reference`: given no session source, when `brain-swap new x` runs, then it exits 0 and `$XDG_STATE_HOME/brain-swap/sessions/` holds no file.
  10. `ts6_3_failed_command_sets_no_reference`: given `--session s1`, when `brain-swap park --card W-99 --session s1` (stdin `Doing: x`) and `brain-swap new --template epic --session s1 x` run, then they exit 3 and no reference for `s1` exists.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/cli/` writes no file under `$XDG_STATE_HOME` itself: references and hints go through E1-F7 only.

### E2-F3-T6 Terminal prompts

- Status: todo
- Depends on: E2-F3-T2
- Covers: FR-34, FR-18; TECHSPEC 6.3 (`park` prompts), AS-6 (terminal step)
- Size: S
- Scope: `cli::cmd::read_note(input, prompts, terminal)`: when `terminal` is false it parses the whole input with `parse_note`; when true it writes `Doing: `, `Next: ` and `Watch out: ` to `prompts` one at a time, reads one line per prompt and takes it verbatim as that part; end of input stops prompting with the remaining parts empty (A-E2-23); three empty parts are `empty note`. `park` passes stdin, stderr (so `--json` keeps stdout one object) and `std::io::stdin().is_terminal()`. Section 13 has no pty crate, so the unit tests drive `read_note` in process and one CLI test runs the binary under util-linux `script`, which gives it a pty (A-E2-35).
- Not in scope: the 6.5 grammar (E2-F3-T2); `--auto` (E2-F3-T3).
- Tests first: (unit, `cli::cmd::read_note`, unless marked CLI)
  1. `fr34_terminal_prompts_for_each_part`: given input `a\nb\nc\n` and `terminal = true`, then the prompt stream holds `Doing: Next: Watch out: ` and the parts are `a`, `b` and `c`.
  2. `fr34_terminal_answer_taken_verbatim`: given input `next: x\n\n\n` and `terminal = true`, then Doing is `next: x` and Next and Watch out are empty.
  3. `fr34_terminal_end_of_input_stops_prompting`: given input `a\n` followed by end of input and `terminal = true`, then the prompt stream holds `Doing: Next: `, Doing is `a` and the other parts are empty.
  4. `fr18_terminal_three_empty_answers_rejected`: given input `\n\n\n` and `terminal = true`, then the result is `Err(Error::InvalidInput(..))` whose message is `empty note`.
  5. `fr34_non_terminal_parses_without_prompts`: given input `Next: b\n` and `terminal = false`, then the prompt stream is empty and Next is `b`.
  6. `as6_park_from_terminal_prompts_and_writes_no_auto` (CLI, a pty through util-linux `script`): given the `work` fixture, when `/usr/bin/script -qec "<brain-swap> park --card W-12" /dev/null` runs through the spawn helper (`<brain-swap>` the absolute path of the built binary) with stdin `a\nb\nc\n`, then it exits 0, its output holds `Doing: `, `Next: ` and `Watch out: ` in that order followed by `parked to W-12: migrate invoices to v13`, and the new note's heading in `W-12.md` has no ` auto` and its parts are `a`, `b` and `c`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `cmd::park` passes `stderr` as the prompt stream (code review item; `--json` stdout stays one object).

### E2-F3-T7 `link`

- Status: todo
- Depends on: E2-F3-T5, E2-F2-T4
- Covers: FR-57, FR-41; TECHSPEC 6.3 (`link`), 6.4, 6.7 (`link`)
- Size: M
- Scope: `cmd::link`: find the card (unknown: exit 3, nothing written); resolve the session (none: `Error::InvalidInput` `no session`, exit 1, after any resolution warning, A-E2-26); write the reference and, inside herdr, the pane hint; print `linked to <ID>: <title>` and then `card_block` with `SessionView::Resolved(session)`, whose `latest note:` ending is `by this session`, `by another session` or `no session`. `--json` returns `card` with `by_this_session` set (null for a note without a session).
- Not in scope: `/bs-link` and its picker (E3-F1).
- Tests first:
  1. `fr57_link_sets_reference_and_prints_block`: given the `work` fixture, when `brain-swap link W-12 --session 55583127-a22a-49bc-a803-e777c377a595` runs, then it exits 0, stdout is `linked to W-12: migrate invoices to v13` followed by E2-F2-T4 test 1's block with the ending `by this session` (insta snapshot), and that session's reference names W-12.
  2. `ts6_3_link_by_another_session`: given the fixture, when `brain-swap link W-12 --session s1` runs, then the `latest note:` line is `latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by another session`.
  3. `ts6_3_link_note_without_session`: given the fixture, when `brain-swap link W-9 --session s1` runs, then the `latest note:` line ends with `1 d ago, no session`.
  4. `ts6_3_link_unknown_card_exits_3_changes_nothing`: given `s1` already linked to W-12, when `brain-swap link W-99 --session s1` runs, then it exits 3 with `brain-swap: W-99 not found` and the reference file for `s1` is byte-identical (text and JSON snapshots).
  5. `fr41_link_without_session_exits_1`: given no flag, no `BRAIN_SWAP_SESSION` and no `HERDR_*`, when `brain-swap link W-12` runs, then it exits 1 with `brain-swap: no session`, the `--json` variant has code `invalid_input`, and `sessions/` holds no file (text and JSON snapshots).
  6. `fr41_link_placeholder_session_outside_herdr_exits_1`: given `--session '${CLAUDE_SESSION_ID}'` and no other source, when `brain-swap link W-12` runs, then stderr holds `warning: session id not substituted` followed by `brain-swap: no session`, and it exits 1.
  7. `fr41_link_inside_herdr_uses_pane_session`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9`, `FAKE_HERDR_SCENARIO=pane-session` and `--session ""`, when `brain-swap link W-11` runs, then the reference for `herdr-s1` and the pane hint for `w4V:p9` both name W-11.
  8. `ts6_7_link_json_returns_card`: given the fixture, when `brain-swap --json link W-12 --session 55583127-a22a-49bc-a803-e777c377a595` runs, then `card.latest_note.by_this_session` is true (insta JSON snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `link` prints the block through `card_block` only (no second renderer).

### E2-F3-T8 `context` with a reference

- Status: todo
- Depends on: E2-F3-T4, E2-F2-T5, E1-F7
- Covers: FR-37; TECHSPEC 6.3 (`context`, first format), 6.7 (`context`)
- Size: M
- Scope: `cmd::context` loads the config itself through `Ctx::load` (problem lines arrive in E2-F3-T10), resolves the session, reads its reference, and when it resolves (I9) prints `brain-swap context v1`, `session: <id>`, `reference: <ID>` and then `card_block` with `SessionView::Resolved`; exit 0. `--json` returns, after `v`, `session`, `reference`, `card`, `candidates` (empty with a reference), `other_boards` (empty with a reference), `problem` (null) and `warnings` (A-E2-29). Tests write references through E1-F7.
- Not in scope: the picker (E2-F3-T9); problem lines, warning lines and the first run (E2-F3-T10); `/bs-park` and `/bs-back` (E3-F1).
- Tests first:
  1. `fr37_context_with_reference_and_note`: given the `work` fixture and session `55583127-a22a-49bc-a803-e777c377a595` referencing W-12, when `brain-swap context --session 55583127-a22a-49bc-a803-e777c377a595` runs, then it exits 0 and stdout is exactly the first 6.3 example with `<sandbox>/ati.billing` as the cwd (insta snapshot).
  2. `fr37_context_note_by_another_session`: given `s1` referencing W-12, when `brain-swap context --session s1` runs, then line 6 is `latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by another session`.
  3. `fr37_context_auto_note`: given `s1` referencing W-7, when `brain-swap context --session s1` runs, then line 6 is `latest note: 2026-10-08T09:40:00+03:00, 2 h ago, auto, by another session`.
  4. `fr37_context_reference_without_notes`: given `s1` referencing W-11, when `brain-swap context --session s1` runs, then the last line is `latest note: none` (insta snapshot).
  5. `fr37_context_note_without_session`: given `s1` referencing W-9, when `brain-swap context --session s1` runs, then line 6 ends with `1 d ago, no session`.
  6. `fr37_context_reference_on_another_board`: given home holding `H-1.md` (Todo) and `s1` referencing it, when `brain-swap context --session s1` runs, then lines 3 and 5 are `reference: H-1` and `board: home, column: Todo`.
  7. `ts6_7_context_json_with_reference`: given test 1's state, when `brain-swap --json context --session 55583127-a22a-49bc-a803-e777c377a595` runs, then stdout is one object whose keys are `v`, `session`, `reference`, `card`, `candidates`, `other_boards`, `problem`, `warnings` in that order, with `reference` `W-12`, `card.latest_note.by_this_session` true, `candidates` `[]`, `other_boards` `[]`, `problem` null and `warnings` `[]` (insta JSON snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `context` never writes: the board folder, `sessions/` and `panes/` are byte-identical after each test.

### E2-F3-T9 `context` picker without a reference

- Status: todo
- Depends on: E2-F3-T8, E2-F3-T7, E2-F2-T8
- Covers: FR-56, FR-57, FR-37; TECHSPEC 6.3 (`context`, second format, `reference: none (W-12 not found)`), 6.6, 6.7 (Candidate in `context`), AS-3 (CLI half)
- Size: M
- Scope: When no session resolves, the session has no reference, or the reference does not resolve (I9), `context` prints `reference: none` or `reference: none (<ID> not found)` (A-E2-07), then `open cards on <board>, best guess first:` for `--board` or the default board, the labelled picker lines in E2-F2-T8's order, and `out::other_boards_line`. JSON: `reference` and `card` null, `candidates` as Candidates, `other_boards`; an unresolved reference adds `<ID> not found` to `warnings` (A-E2-29).
- Not in scope: problem lines (E2-F3-T10); `/bs-park`'s handling of the pick (E3-F1).
- Tests first:
  1. `fr56_context_without_reference_lists_picker`: given the `work` fixture and no reference for `s9`, when `brain-swap context --session s9` runs, then it exits 0 and stdout is exactly `brain-swap context v1`, `session: s9`, `reference: none`, `open cards on work, best guess first:`, the three picker lines of E2-F2-T8 test 1 and `o. other board: home, personal` (insta snapshot).
  2. `fr56_context_deleted_card_reference`: given `s1` referencing W-12 and `W-12.md` deleted, when `brain-swap context --session s1` runs, then line 3 is `reference: none (W-12 not found)` and the picker lists W-7 then W-11.
  3. `fr56_context_reference_to_unconfigured_board`: given `s1` referencing `H-1` and a config without `home`, when `brain-swap context --session s1` runs, then line 3 is `reference: none (H-1 not found)`.
  4. `fr57_new_session_in_same_pane_sees_linked_here`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p11` and `brain-swap link W-11 --session old` run first, when `brain-swap context --session new` runs with the same `HERDR_*`, then line 5 is `1. W-11 invoice PDF shows the wrong VAT [Todo] no note, linked here`.
  5. `fr56_context_board_flag_lists_named_board`: given home holding `H-1.md` (Todo) and no reference for `s9`, when `brain-swap --board home context --session s9` runs, then line 4 is `open cards on home, best guess first:` and the last line is `o. other board: work, personal`.
  6. `fr56_context_no_session_lists_picker_without_warning`: given no session source, when `brain-swap context` runs, then lines 2 and 3 are `session: none` and `reference: none`, no line starts with `warning:`, and stderr is empty.
  7. `ts6_7_context_json_candidates`: given test 2's state, when `brain-swap --json context --session s1` runs, then `reference` and `card` are null, `candidates` holds W-7 and W-11 with `rank` 1 and 2 and `reason` `middle_column` and `other`, `other_boards` is `["home","personal"]`, `problem` is null and `warnings` is `["W-12 not found"]` (insta JSON snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The picker lines of `context` and `ls --open --guess` are byte-identical for the same state (one test compares them).

### E2-F3-T10 `context` problems, warnings and first run

- Status: todo
- Depends on: E2-F3-T9
- Covers: FR-37, FR-41; TECHSPEC 4.3 (`board.md` warning), 6.2 (`context` exits 0 in every state), 6.3 (problem lines), 6.4 (warnings), 10 (Config bullet), 11 (`BRAIN_SWAP_LOG` inside a board folder), 12.1 (`context` states)
- Size: M
- Scope: `context` exits 0 in every expected state. A missing config is the first run (`created config <path>` on stderr, under `--json` `warning: created config <path>` through `Out::warn`, then the empty default board). An unparseable or invalid config prints `error: config <path>:<line>:<col>: <problem>` and `hint: fix <path>, then run the command again`; a board folder that cannot be read prints `error: board <name>: <os error>` and `hint: check the folder of board <name> in <config path>`; an unknown `--board` prints `error: unknown board '<name>' (<names>)` (E1-F3's text) and `hint: use one of the listed boards`; output ends after the `hint:` line (A-E2-28). Session warnings, board load warnings (such as `board.md:<line>: <problem>`) and config warnings, including `Ctx.warnings` (the E2-F1-T4 log guard), become `warning: <text>` lines right after the `session:` line and are not printed on stderr (A-E2-27); `session: none` when none resolves. JSON: `problem` is `{"code","message","hint"}` with code `config`, `io` or `not_found` (unknown board); `warnings` holds the same texts. Exit 0 in each case (6.2, FR-37).
- Not in scope: E1-F3's error texts; the TUI's ConfigError (E4-F1).
- Tests first:
  1. `fr37_context_first_run_without_config`: given `Sandbox::new()` (no config), when `brain-swap context --session s9` runs, then it exits 0, the config file exists, stderr is `created config <path>`, and stdout is `brain-swap context v1`, `session: s9`, `reference: none`, `open cards on work, best guess first:`, `o. other board: home, personal`.
  2. `fr37_context_broken_config_reports_error_line`: given the broken config of E2-F1-T4, when `brain-swap context --session s9` runs, then it exits 0, stderr is empty and stdout is exactly `brain-swap context v1`, `session: s9`, `error: config <path>:4:<col>: <problem>`, `hint: fix <path>, then run the command again` (insta snapshot).
  3. `fr37_context_unreadable_board_reports_error_line`: given the work board path created as a regular file, when `brain-swap context --session s9` runs, then it exits 0 and the last two lines are `error: board work: <os error>` and `hint: check the folder of board work in <config path>` (insta snapshot).
  4. `fr37_context_missing_board_folder_reads_empty`: given the sandbox (work folder absent), when `brain-swap context --session s9` runs, then it exits 0, `open cards on work, best guess first:` is followed directly by `o. other board: home, personal`, and no board folder is created.
  5. `fr41_context_empty_session_warns_in_output`: given `--session ""` and no other source, when `brain-swap context` runs, then lines 2 and 3 are `session: none` and `warning: session id not substituted`, the picker follows, and stderr is empty.
  6. `fr41_context_placeholder_session_warns_in_output`: given `--session '${CLAUDE_SESSION_ID}'` and no other source, when `brain-swap context` runs, then lines 2 and 3 are as in test 5.
  7. `fr41_context_invalid_session_warns_in_output`: given `--session 'a b'` and no other source, when `brain-swap context` runs, then line 3 is `warning: invalid session id`.
  8. `fr41_context_inside_herdr_uses_pane_session`: given `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9`, `FAKE_HERDR_SCENARIO=pane-session` and `--session ""`, when `brain-swap context` runs, then lines 2 and 3 are `session: herdr-s1` and `warning: session id not substituted`.
  9. `ts4_3_context_invalid_board_md_warns`: given the `work` fixture with `board.md` replaced by `---\nletter W\n---\n`, when `brain-swap context --session s9` runs, then it exits 0, line 3 is `warning: board.md:2: <problem>` and the picker is still printed.
  10. `ts6_7_context_json_problem`: given the broken config, when `brain-swap --json context --session s9` runs, then it exits 0 and stdout is `{"v":1,"session":"s9","reference":null,"card":null,"candidates":[],"other_boards":[],"problem":{"code":"config","message":"config <path>:4:...","hint":"fix <path>, then run the command again"},"warnings":[]}` (insta JSON snapshot).
  11. `fr37_context_unknown_board_reports_error_line`: given the sandbox, when `brain-swap --board work2 context --session s9` runs, then it exits 0, stderr is empty and stdout is exactly `brain-swap context v1`, `session: s9`, `error: unknown board 'work2' (work, home, personal)` and `hint: use one of the listed boards` (insta snapshot), and the `--json` variant exits 0 with `problem.code` `not_found`.
  12. `ts11_context_log_inside_board_folder_warns_in_output`: given the sandbox and `BRAIN_SWAP_LOG=<root>/boards/work/bs.log`, when `brain-swap context --session s9` runs, then it exits 0, line 3 is `warning: BRAIN_SWAP_LOG inside board work, ignored`, stderr is empty, and `<root>/boards/work/bs.log` does not exist.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every 12.1 `context` state (no config, broken config, missing folder, no reference, deleted card, empty and placeholder session, with and without notes) has a committed text snapshot across E2-F3-T8 to E2-F3-T10, and each of those tests asserts exit 0.

### E2-F3-T11 Process-level crash tests

- Status: todo
- Depends on: E2-F3-T1, E2-F2-T1
- Covers: FR-16; TECHSPEC 4.6, 6.1 (`BRAIN_SWAP_FAILPOINT`), 12.1 (crash, process level), I1 to I4
- Size: M
- Scope: Relies on E1-F1-T7: `Env::from_vars` already reads `BRAIN_SWAP_FAILPOINT` as an aborting failpoint under the feature, so the store aborts the process at the named point (12.1); `main.rs` is unchanged. `tests/crash_cli.rs` (`#![cfg(feature = "failpoints")]`) runs `new` with `create:after_next`, `create:after_tmp` and `create:after_link`, and `park` with `park:after_tmp`, each on a fresh copy of the `work` fixture (`next: 13`), then reloads the board through E1-F5 and asserts that the process did not exit 0, every card file parses and is writable, I1 to I4 hold, and the next `new` takes a number never used before.
- Not in scope: in-process crash tests, the failpoint mechanism and the `BRAIN_SWAP_FAILPOINT` read (E1-F6, E1-F1-T7); `edit:after_tmp` (E4-F5).
- Tests first: (layer: crash, `cargo test --features failpoints`)
  1. `ts12_1_crash_create_after_next_skips_number`: given the fixture, when `BRAIN_SWAP_FAILPOINT=create:after_next brain-swap new x` runs, then it does not exit 0, `board.md` holds `next: 14`, no `W-13.md` exists, `brain-swap ls` exits 0 listing the four fixture cards, and a following `brain-swap new y` prints `created W-14: y (Todo, work)`.
  2. `ts12_1_crash_create_after_tmp_leaves_ignored_temp`: given the fixture, when `BRAIN_SWAP_FAILPOINT=create:after_tmp brain-swap new x` runs, then it does not exit 0, a file matching `.W-13.md.bs-tmp-*` exists, no `W-13.md` exists, `brain-swap ls` exits 0 listing four cards, and `brain-swap new y` creates `W-14`.
  3. `ts12_1_crash_create_after_link_card_parses`: given the fixture, when `BRAIN_SWAP_FAILPOINT=create:after_link brain-swap new x` runs, then it does not exit 0, `W-13.md` exists and loads with title `x`, `brain-swap ls` lists five cards, and `brain-swap new y` creates `W-14`.
  4. `ts12_1_crash_park_after_tmp_leaves_card_unchanged`: given the fixture, when `BRAIN_SWAP_FAILPOINT=park:after_tmp brain-swap park --card W-12` runs with stdin `Doing: x`, then it does not exit 0, `W-12.md` is byte-identical to the fixture, a `.W-12.md.bs-tmp-*` file exists, every card loads, and a following `brain-swap park --card W-12` without the failpoint exits 0 and adds exactly one note.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/main.rs` and `src/cli/` name no `BRAIN_SWAP_FAILPOINT` (a grep finds it only in `src/core/`), and `cargo clippy --all-targets -- -D warnings` passes with and without `--all-features`.

### E2-F3-T12 Process-level concurrency and the AS-8 CLI half

- Status: todo
- Depends on: E2-F3-T1, E2-F2-T6
- Covers: FR-16; TECHSPEC 10 (lock, fresh reads), 12.1 (concurrency, process level), AS-8 (CLI steps)
- Size: M
- Scope: `tests/concurrency_cli.rs` starts `park`, `move` and `new` processes from threads through the spawn helper, waits for all, and checks the files with E1-F5's loader. Create `tests/fixtures/boards/pair/`: `board.md` (`next: 3`), `W-1.md` and `W-2.md` in Todo with empty timelines.
- Not in scope: the in-process thread variant (E1-F6); the TUI editor case of AS-8 (E4-F5).
- Tests first: (layer: concurrency)
  1. `ts12_1_parallel_parks_and_moves_lose_nothing`: given the `pair` fixture, when 20 `park` processes (10 on each card, stdin `Doing: n<i>` for i from 1 to 20) and 5 `move` processes (alternating W-1 and W-2 between Doing and Todo) start together, then every process exits 0, both files load, they hold 20 notes in total with each `n<i>` exactly once, and each card's column is Todo or Doing.
  2. `ts12_1_parallel_creations_get_distinct_ids`: given the `pair` fixture, when 10 `new` processes start together, then every process exits 0, 10 new card files exist with 10 distinct IDs, every file loads, and `board.md`'s `next` exceeds every card number (I4).
  3. `as8_two_parks_on_different_cards_in_one_second`: given the `work` fixture and one `BRAIN_SWAP_NOW` for both, when `park --card W-12` and `park --card W-7` start together, then both exit 0, each file equals its fixture bytes plus one note block, and `W-9.md`, `W-11.md` and `board.md` are byte-identical.
  4. `as8_two_parks_on_one_card_append_both`: given the `work` fixture, when two `park --card W-12` processes with stdin `Doing: one` and `Doing: two` start together, then both exit 0 and W-12 holds two new notes, `one` and `two` in either order, under identical headings.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The suite runs in under 10 seconds on the author's machine (no sleeps; the only waits are process exits).

## Assumptions

- A-E2-01 Do `templates` and `install` neither read nor write the config, reading FR-66's "any invocation writes the default config" as "any invocation that reads the config", so `templates` always exits 0 (6.3)? Affects E2-F1-T1, E2-F1-T4.
- A-E2-02 Is FR-36's "documented" met by the 6.2 exit code table at the end of `brain-swap --help` and in README? Affects E2-F1-T3.
- A-E2-03 Is `--json` output one compact line, not pretty-printed? Affects E2-F1-T2.
- A-E2-04 Is a clap parse failure printed as `brain-swap: ` plus clap's first error line without its `error: ` prefix (then clap's usage lines in text mode), with `--json` detected by scanning the raw arguments before any `--`? Affects E2-F1-T3.
- A-E2-05 Is a malformed card ID argument (`W12`, `foo`) a usage error (exit 2) rather than not found (exit 3)? Affects E2-F1-T3.
- A-E2-06 Does E2-F1 declare the full clap surface with `todo!()` arms for subcommands built later, and does `cli::run` load the config before dispatch for every subcommand except `templates`, `install` and `context`? Affects E2-F1-T1, E2-F1-T4.
- A-E2-07 Is an unknown card reported by every subcommand with E1-F5's `Error::NotFound("<ID> not found")` passed through unchanged (also for a letter no board has), while `context` keeps the 6.3 literal `reference: none (<ID> not found)` and the JSON warning `<ID> not found`, also for a reference whose board is no longer configured? Affects E2-F2-T4, E2-F2-T6, E2-F3-T1, E2-F3-T7, E2-F3-T9.
- A-E2-08 Does `new` without any title exit 1 with code `invalid_input` and message `no title`? Affects E2-F2-T1, E2-F2-T3.
- A-E2-09 Are positional title words joined by single spaces and the result trimmed? Affects E2-F2-T1.
- A-E2-10 Does `--body` accept only `-`, any other value being a usage error (exit 2)? Affects E2-F2-T3.
- A-E2-12 Does `move` to the card's current column print the usual `moved <ID> to <Column>`? Affects E2-F2-T6.
- A-E2-13 Does `show --all` keep the normal block and then add each older note, newest first, as a blank line and a block headed `note <n>: <stamp>, <age> ago[, auto], <ending>`, n counted from the oldest as `locate --note` counts? Affects E2-F2-T5.
- A-E2-14 Does `auto` sit between the age and the session ending, as in `latest note: <stamp>, <age> ago, auto, by this session`? Affects E2-F2-T4, E2-F3-T8.
- A-E2-15 Is age `now` printed as `now` without ` ago`, and a bad stamp as the heading's text followed by `? ago`? Affects E2-F2-T4.
- A-E2-16 In the card block, are a part's continuation lines printed under it indented by two spaces, an empty part as its bare label, `Where: <cwd>` without ` (pane ...)` when no pane is recorded, and `Where: none` for a note without a place? Affects E2-F2-T4.
- A-E2-17 Is a preview with no word boundary in its first 40 characters cut at 40 characters (Unicode scalar values), is `: <preview>` dropped when the preview is empty, and is an extra column shown as written (`[Doign]`)? Affects E2-F2-T7.
- A-E2-18 Does `ls --open` keep the header `cards on <board>:`, and does `--guess` imply `--open`? Affects E2-F2-T7, E2-F2-T8.
- A-E2-19 Is the `o. other board:` line omitted when no other board is configured, with the names in config order? Affects E2-F2-T8, E2-F3-T9.
- A-E2-20 Is a Candidate's `rank` its 1-based position in the printed list? Affects E2-F2-T8, E2-F3-T9.
- A-E2-21 In note JSON, is `by_this_session` null when the command resolves no session (always for `show`) or the note has no session, `age_min` floored whole minutes with 0 for a future stamp, and `at` null for a bad stamp? Affects E2-F2-T5.
- A-E2-23 Are the terminal prompts `Doing: `, `Next: ` and `Watch out: ` on stderr, one line read per prompt and taken verbatim, end of input ending the prompts? Affects E2-F3-T6.
- A-E2-24 Does a repeated label on stdin continue its part on a new line rather than replace it? Affects E2-F3-T2.
- A-E2-25 Is the exit 4 message `no reference: pass --card <ID>`? Affects E2-F3-T5.
- A-E2-26 Is `link`'s failure without a session code `invalid_input` with message `no session`? Affects E2-F3-T7.
- A-E2-27 Does `context` print every warning (session, board load, config) as a `warning: <text>` line right after the `session:` line, and nothing on stderr except the first-run notice? Affects E2-F3-T10.
- A-E2-28 Are the `context` hints `hint: fix <path>, then run the command again` (config), `hint: check the folder of board <name> in <config path>` (board folder) and `hint: use one of the listed boards` (unknown `--board`), with output ending after the hint? Affects E2-F3-T10.
- A-E2-29 With a reference, are `context`'s JSON `candidates` and `other_boards` empty arrays, and with an unresolved reference is `reference` null with `<ID> not found` in `warnings`? Affects E2-F3-T8, E2-F3-T9.
- A-E2-31 Does `park` through the reference (no `--card`) leave the reference file untouched, since 6.3 names only `park --card` as setting it? Affects E2-F3-T5.
- A-E2-32 Does every subcommand that loads the config print the loader's warnings (such as unknown keys) on stderr, leaving key-binding warnings to the TUI? Affects E2-F2-T7.
- A-E2-34 Is the first-run notice the plain line `created config <path>` on stderr in text mode, and `warning: created config <path>` through `Out::warn` under `--json` (6.2: with `--json` stderr carries only warnings)? Affects E2-F2-T1, E2-F3-T10.
- A-E2-35 Does util-linux `script` (`/usr/bin/script -qec`) stand in for a pty crate, which section 13 does not list, so one CLI test runs `park --card` on a real pty while the other terminal cases drive `read_note` in process; is that test Linux-only (macOS `script` takes other flags), skipped with a printed note when `/usr/bin/script` is not util-linux? Affects E2-F3-T6.
