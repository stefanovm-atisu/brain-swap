# E1 Core

Milestone: M1. E1 delivers the tool-agnostic library under `src/core/`: the `Env` value, errors and exit codes, card IDs, stamps and ages, the failpoint and log hooks, the hermetic test helpers, the card and board file grammar with its byte-preserving splices, configuration and key bindings, templates, board loading, the locked store with atomic writes, crash safety and the editor merge, the session reference store and the best guess. It sits in M1 because principle 1 ships the cue (files, CLI, pack) before the board, and every later epic calls these modules: E2 CLI wraps them in subcommands, E3 reaches them through the CLI, E4 TUI renders and writes through them, E5 herdr fills `Place` and reads `jump_note()`. TECHSPEC 15.1 starts E1-F1 first and then runs E1-F2, E1-F3, E1-F4 and E1-F7 in parallel. Spec: TECHSPEC 2, 3, 4, 5, 6.1, 6.2, 6.4, 6.6, 10, 11, 12.1, 12.2, 13 and the E1 entries of 15.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E1-F1 | Foundations | none | 10 | L |
| E1-F2 | Card grammar | E1-F1 | 8 | L |
| E1-F3 | Config and keys | E1-F1 | 6 | M |
| E1-F4 | Templates | E1-F1 (task E1-F4-T1 also needs E1-F2-T1, A-E1-02) | 3 | S |
| E1-F5 | Board loading | E1-F2, E1-F3 | 5 | M |
| E1-F6 | Store | E1-F4, E1-F5 (task E1-F6-T8 also needs E1-F7-T4, A-E1-03) | 10 | L |
| E1-F7 | Session store | E1-F1 | 4 | M |
| E1-F8 | Best guess | E1-F5, E1-F7 | 3 | S |

## E1-F1 Foundations

- Depends on: none
- Covers: FR-36, NFR-03, NFR-04, NFR-09, NFR-11
- Spec: TECHSPEC 2, 3 (CardId, Place, HerdrPlace, stamps), 5.1, 6.1, 6.2, 7.3 (age rule), 11, 12.1, 12.2, 13
- Scope: The Cargo package of section 13 with the module layout of 2.1 and the layering rules of 2.2 enforced by `tests/layering.rs`. The `Error` enum with its exit codes and error codes (6.2, 11), the `CardId` type (I1, I2) with the plain `Place` and `HerdrPlace` structs, stamp parsing and writing (section 3), ages (7.3), and the `Env` value with XDG paths that `main` builds once (5.1, 6.1). The failpoint hook (12.1) and the `BRAIN_SWAP_LOG` sink (11) that the store and the adapters call. The hermetic spawn helper and the docs test of 12.1 that every later test relies on.
- Provides: crate `brain_swap` with modules `core`, `cli`, `tui`, `adapters` (2.1) and `src/main.rs`; `core::error::Error` (`Io(String)`, `Unreadable(String)`, `InvalidInput(String)`, `Config { path: PathBuf, line: usize, col: usize, msg: String }`, `Verify(String)`, `Usage(String)`, `NotFound(String)`, `NoReference(String)`, `Busy(String)`) with `exit_code() -> i32`, `code() -> &'static str` and `Display`; `core::error::Result<T>`; `core::error::EXIT_JUMP_NOT_PERFORMED` (6); `core::model::CardId { letter: char, number: u32 }` with `CardId::parse(&str) -> Option<CardId>`, `CardId::from_file_name(&str) -> Option<CardId>`, `CardId::file_name() -> String`, `Display`; `core::model::Place { cwd, herdr, session }`, `core::model::HerdrPlace { pane, tab, workspace }` (section 3); `core::time::Stamp` (= `jiff::Zoned`), `core::time::parse_stamp(&str, &TimeZone) -> Option<Stamp>`, `core::time::format_stamp(&Stamp) -> String`, `core::time::Age` (`Display`), `core::time::age(Option<&Stamp>, &Stamp) -> Age`, `core::time::age_minutes(Option<&Stamp>, &Stamp) -> Option<i64>`; `core::env::Env` (fields of E1-F1-T6, `Clone`) with `Env::from_vars`, `Env::herdr_active()`, `config_file()`, `templates_dir()`, `data_dir()`, `default_board_dir(name)`, `state_dir()`, `sessions_dir()`, `panes_dir()`, `locks_dir()`, `edit_dir()`; `Env.exe` (canonical `current_exe`, read in main); `core::env::encode_file_name(&str) -> String`; `core::failpoint::Failpoint { name: String, abort: bool }` (`Clone`), `core::failpoint::check(&Env, &str) -> Result<()>`, `core::failpoint::NAMES`; `core::log::event(&Env, kind: &str, text: &str)`, `core::log::guard_board_folders(&mut Env, &[(String, PathBuf)]) -> Option<String>`; `tests/common/env.rs` `core_env(home: &Path, now: &str) -> Env` (time zone: the fixed offset of `now`); `tests/common/spawn.rs` `Scenario` (`Scenario::new(tmp: &Path)`; setters `herdr`, `fake_scenario`, `failpoint`, `session`, `log`, `visual`, `editor`, `now`, `xdg(config: &Path, data: &Path, state: &Path)`, `unset(name: &str)`, `path(value: &str)`; `vars(&self) -> Vec<(String, String)>`), `command(&Scenario, &Path) -> std::process::Command`, `brain_swap(&Scenario) -> assert_cmd::Command`, `FAKE_HERDR`; `tests/layering.rs`; `tests/docs.rs`
- Requires: none
- Feature DoD:
  - [ ] A planted `std::process` in `src/core/` fails the layering test (E1-F1-T1).
  - [ ] Exactly six runtime crates, each justified in TECHSPEC 13 (E1-F1-T1).
  - [ ] The crate type-checks for macOS (`aarch64-apple-darwin`, NFR-09) (E1-F1-T1).
  - [ ] Exit codes 3, 4, 5 and 6 are distinct and every `Error` variant maps to its 6.2 exit and error code (E1-F1-T2).
  - [ ] Stamps round-trip byte for byte (E1-F1-T4).
  - [ ] Every age boundary of 7.3 is tested on both sides (E1-F1-T5).
  - [ ] Paths follow 5.1 with the same defaults on Linux and macOS (E1-F1-T6).
  - [ ] A named failpoint stops the caller with `Error::Io` in-process (E1-F1-T7).
  - [ ] The log sink appends one line per event written through its API to the file named by `BRAIN_SWAP_LOG` and writes nothing when it is unset (E1-F1-T8).
  - [ ] Every spawned test process goes through the one helper and sees only the 12.1 variables (E1-F1-T9).
  - [ ] No U+2014 or U+2013 in any repository markdown file (E1-F1-T10).

### E1-F1-T1 Package skeleton and layering test

- Status: done
- Depends on: none
- Covers: NFR-03, NFR-04, NFR-09, NFR-11; TECHSPEC 2.1, 2.2, 13
- Size: M
- Scope: Create `Cargo.toml` exactly as TECHSPEC 13 (package `brain-swap`, lib `brain_swap`, bin `brain-swap`, edition 2024, rust-version 1.89, feature `failpoints = []`, the six runtime crates with their justification comments, dev crates `insta`, `assert_cmd`, `tempfile`), `src/lib.rs` declaring `core`, `cli`, `tui`, `adapters`, empty module files for the 2.1 file list, and a `src/main.rs` that exits 0 for now. Write `tests/layering.rs` with a plain-text scanner `scan(root: &Path) -> Vec<Violation>` (file and line per hit) so tests can point it at planted temp trees. Rules: `src/core/` uses neither `std::process` nor `std::env` (only `src/core/failpoint.rs` may use `std::process`) and names none of `crate::cli`, `crate::tui`, `crate::adapters`; `src/adapters/` names neither `crate::cli` nor `crate::tui`; no file under `src/` uses `std::net`, `TcpStream`, `TcpListener`, `UdpSocket`, `UnixStream` or `UnixListener` (NFR-04, 2.2 "never the socket"); `Command::new` appears only in `src/adapters/runner.rs` and `src/tui/editor.rs` (NFR-03, A-E1-12); `cfg(target_os` appears nowhere (NFR-09, one code path for Linux and macOS).
- Not in scope: module contents (later tasks); the docs test (E1-F1-T10); the spawn-only-through-helper rule for `tests/` (E1-F1-T9); `cli::run` (E2-F1) and `tui::run` (E4-F1); the macOS type-check of the release commit (`cargo check --target aarch64-apple-darwin`) is E6-F1-T6 step 4 (A-E6-21).
- Tests first:
  1. `nfr11_manifest_lists_six_runtime_crates`: given the repository `Cargo.toml`, when its `[dependencies]` table is read with `toml`, then its keys are exactly `ratatui`, `clap`, `serde`, `toml`, `serde_json` and `jiff`.
  2. `ts2_2_layering_passes_on_repository`: given the repository `src/`, when `scan` runs, then it returns no violation.
  3. `ts2_2_planted_process_in_core_fails`: given a temp tree whose `src/core/store.rs` has `use std::process::Command;` on line 3, when `scan` runs, then it returns exactly one violation naming `src/core/store.rs:3`.
  4. `ts2_2_planted_env_in_core_fails`: given `std::env::var("HOME")` on line 5 of `src/core/config.rs` in a temp tree, when `scan` runs, then one violation names `src/core/config.rs:5`.
  5. `ts2_2_failpoint_may_use_process`: given `std::process::abort()` in `src/core/failpoint.rs` of a temp tree, when `scan` runs, then it returns no violation.
  6. `ts2_2_core_importing_tui_fails`: given `use crate::tui::App;` in `src/core/board.rs` of a temp tree, when `scan` runs, then one violation names that line.
  7. `ts2_2_adapters_importing_cli_fails`: given `use crate::cli::out;` in `src/adapters/herdr.rs` of a temp tree, when `scan` runs, then one violation names that line.
  8. `nfr04_socket_type_anywhere_fails`: given `use std::os::unix::net::UnixStream;` in `src/adapters/herdr.rs` of a temp tree, when `scan` runs, then one violation names that line.
  9. `nfr03_command_outside_runner_and_editor_fails`: given `Command::new("sh")` in `src/cli/cmd.rs` of a temp tree, when `scan` runs, then one violation names it, and given the same line only in `src/adapters/runner.rs`, then none.
  10. `nfr09_target_os_cfg_fails`: given `#[cfg(target_os = "linux")]` in `src/core/store.rs` of a temp tree, when `scan` runs, then one violation names that line.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `cargo tree -e normal --depth 1` lists exactly the six crates of TECHSPEC 13, and `cargo build` succeeds with and without `--features failpoints`.
  - [ ] `cargo check --all-targets --target aarch64-apple-darwin` succeeds (after `rustup target add aarch64-apple-darwin`), recorded in the commit message (NFR-09).

### E1-F1-T2 Error enum and exit codes

- Status: done
- Depends on: E1-F1-T1
- Covers: FR-36; TECHSPEC 6.2, 11, T-14
- Size: S
- Scope: `src/core/error.rs` with `pub enum Error { Io(String), Unreadable(String), InvalidInput(String), Config { path: PathBuf, line: usize, col: usize, msg: String }, Verify(String), Usage(String), NotFound(String), NoReference(String), Busy(String) }` (A-E1-06), `pub type Result<T> = std::result::Result<T, Error>`, `exit_code(&self) -> i32` and `code(&self) -> &'static str` per 6.2, `Display` as one line (the variant's message; `Config` renders `config <path>:<line>:<col>: <msg>`) and `std::error::Error`. `pub const EXIT_JUMP_NOT_PERFORMED: i32 = 6` (A-E1-07). A doc comment on `Error` lists every exit code 0 to 6 with its meaning, so FR-36's "documented" has one source that E2-F1 copies into `--help`.
- Not in scope: printing `brain-swap: <message>` and the JSON error envelope (E2-F1); producing exit 6 (E5-F2).
- Tests first:
  1. `fr36_not_found_exits_3`: given `Error::NotFound("W-99 not found".into())`, when `exit_code()` and `code()` are called, then they return 3 and `not_found`.
  2. `fr36_no_reference_exits_4`: given `Error::NoReference(..)`, when `exit_code()` and `code()` are called, then they return 4 and `no_reference`.
  3. `fr36_busy_exits_5`: given `Error::Busy(..)`, when `exit_code()` and `code()` are called, then they return 5 and `busy`.
  4. `ts6_2_runtime_errors_exit_1`: given one value each of `Io`, `Unreadable`, `InvalidInput`, `Config` and `Verify`, when mapped, then every exit code is 1 and the codes are `io`, `unreadable`, `invalid_input`, `config` and `verify_failed`.
  5. `ts6_2_usage_exits_2`: given `Error::Usage(..)`, when mapped, then exit 2 and code `usage`.
  6. `fr36_exit_codes_are_distinct`: given the exit codes of `NotFound`, `NoReference`, `Busy` and `EXIT_JUMP_NOT_PERFORMED`, when compared, then they are 3, 4, 5 and 6, pairwise distinct and none is 0, 1 or 2.
  7. `ts11_config_display_names_file_line_col`: given `Config { path: "/h/.config/brain-swap/config.toml", line: 4, col: 7, msg: "expected '='" }`, when displayed, then the text is `config /h/.config/brain-swap/config.toml:4:7: expected '='`.
  8. `ts11_display_is_one_line`: given each variant built with a single-line message, when displayed, then no output contains `\n`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The `Error` doc comment lists exit codes 0 to 6 and the nine error codes of 6.2.

### E1-F1-T3 Card ID type

- Status: done
- Depends on: E1-F1-T1
- Covers: TECHSPEC 3 (`CardId`, `Place`, `HerdrPlace`, I1, I2), 4.1 (card file name pattern)
- Size: S
- Scope: `src/core/model.rs` with `pub struct CardId { pub letter: char, pub number: u32 }` (Copy, Eq, Ord by letter then number, Hash, Debug), `Display` as `W-12`, `CardId::parse(&str) -> Option<CardId>` accepting `^[A-Za-z]-[1-9][0-9]*$` within `u32` and upper-casing the letter, `CardId::file_name(&self) -> String` (`W-12.md`, I2) and `CardId::from_file_name(&str) -> Option<CardId>` accepting exactly `^[A-Z]-[1-9][0-9]*\.md$` (4.1). Also add the plain section 3 structs `pub struct Place { pub cwd: Option<PathBuf>, pub herdr: Option<HerdrPlace>, pub session: Option<String> }` and `pub struct HerdrPlace { pub pane: String, pub tab: Option<String>, pub workspace: Option<String> }` (Clone, Debug, PartialEq, Eq, and `Default` for `Place`, whose default E1-F2-T4 compares against), so E5-F1 can build a place right after E1-F1 (A-E1-01). The types live here so E1-F2, E1-F7 and E5-F1 can start in parallel (A-E1-01).
- Not in scope: resolving an ID to a board by its letter (E1-F5-T4); the other section 3 types (E1-F2-T3, E1-F5-T2, E1-F7-T2); reading and writing the place line (E1-F2-T3, E1-F2-T4, E1-F2-T7); filling `HerdrPlace` from herdr (E5-F1-T7).
- Tests first:
  1. `i1_parse_reads_letter_and_number`: given `W-12`, when `CardId::parse` runs, then it returns letter `W` and number 12.
  2. `i1_parse_upper_cases_letter`: given `w-12`, when parsed, then the result equals `CardId::parse("W-12")`.
  3. `i1_parse_rejects_malformed_ids`: given each of `W-0`, `W-012`, `W-`, `WW-1`, `1-2`, `W12`, `W-1x` and the empty string, when parsed, then each returns `None`.
  4. `i1_parse_rejects_number_beyond_u32`: given `W-4294967296`, when parsed, then it returns `None`.
  5. `i2_file_name_is_id_dot_md`: given `W-12`, when `file_name()` and `to_string()` are called, then they return `W-12.md` and `W-12`.
  6. `ts4_1_from_file_name_requires_card_pattern`: given `W-12.md`, then `Some(W-12)`, and given each of `w-12.md`, `W-012.md`, `.W-12.md.bs-tmp-4711`, `W-12.md.bak` and `README.md`, then `None`.
  7. `ts3_place_default_has_no_parts`: given `Place::default()`, when its fields are read, then `cwd`, `herdr` and `session` are all `None`, and it equals `Place { cwd: None, herdr: None, session: None }`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `CardId` derives `Ord` so that E1-F5 and E1-F8 tie rules can compare numbers directly.

### E1-F1-T4 Stamps

- Status: blocked bug hunter still fails after two fix rounds (a fraction longer than 9 digits reads as a bad stamp), branch pushed as wip/E1-F1-T4
- Depends on: E1-F1-T1
- Covers: TECHSPEC 3 (stamps), 4.4 (accepted stamp forms)
- Size: S
- Scope: `src/core/time.rs` with `pub type Stamp = jiff::Zoned`, `parse_stamp(text: &str, local: &jiff::tz::TimeZone) -> Option<Stamp>` and `format_stamp(&Stamp) -> String`. RFC 3339 with an offset (`Z` reads as `+00:00`, A-E1-14) is parsed through `jiff::fmt::temporal::Pieces` (or a `Timestamp` plus the parsed offset) and stored in `TimeZone::fixed(offset)`; `YYYY-MM-DD HH:MM` and `YYYY-MM-DD HH:MM:SS` without offset are read in `local`; anything else is `None`. `format_stamp` uses `strftime("%Y-%m-%dT%H:%M:%S%:z")`, never `Display`.
- Not in scope: ages (E1-F1-T5); resolving the local zone (`main`, E1-F1-T6); serde for `set_at` (E1-F7-T2).
- Tests first:
  1. `ts3_stamp_round_trips_byte_for_byte`: given `2026-10-08T10:31:05+03:00`, when parsed with any local zone and formatted, then the output equals the input byte for byte.
  2. `ts3_round_trip_keeps_negative_and_zero_offsets`: given `2026-01-31T23:59:59-05:30` and `2026-10-08T08:31:05+00:00`, when parsed and formatted, then each output equals its input.
  3. `ts3_format_never_appends_bracketed_zone`: given a `Zoned` in `TimeZone::fixed(+03:00)`, when formatted, then the text contains no `[`.
  4. `ts4_4_local_stamp_without_seconds_uses_local_zone`: given `2026-10-08 10:31` and local zone fixed `+03:00`, when parsed and formatted, then the text is `2026-10-08T10:31:00+03:00`.
  5. `ts4_4_local_stamp_with_seconds`: given `2026-10-08 10:31:05` and local zone UTC, when parsed and formatted, then the text is `2026-10-08T10:31:05+00:00`.
  6. `ts4_4_z_offset_reads_as_utc`: given `2026-10-08T07:31:05Z`, when parsed and formatted, then the text is `2026-10-08T07:31:05+00:00`.
  7. `ts4_4_bad_stamp_is_none`: given each of `yesterday`, `2026-13-01T00:00:00+03:00`, `2026-10-08T10:31:05` and the empty string, when parsed, then each returns `None`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] No `Zoned` is formatted with `Display` or `{}` anywhere in `src/core/` (checked by grep in review).

### E1-F1-T5 Ages

- Status: todo
- Depends on: E1-F1-T4
- Covers: TECHSPEC 7.3 (age rule; FR-21 itself, with its refresh and rendering, is E4-F2), 10 (clock skew), 6.7 (`age_min`)
- Size: S
- Scope: `core::time::Age` (`Now`, `Min(i64)`, `Hours(i64)`, `Days(i64)`, `Unknown`) with `Display` `now`, `N min`, `N h`, `N d` and `?`; `age(at: Option<&Stamp>, now: &Stamp) -> Age` comparing instants, flooring every unit, future stamps and anything under 60 s giving `now`, `None` giving `?`; `age_minutes(at, now) -> Option<i64>` (floored minutes, a future stamp gives 0) for the JSON `age_min` of 6.7.
- Not in scope: the 30-second refresh and card rendering (E4-F2); the `ago` suffix in text output (E2-F2, E2-F3).
- Tests first:
  1. `fr21_under_60_seconds_is_now`: given `at` 59 s before `now`, when `age` runs, then it displays `now`.
  2. `fr21_future_stamp_is_now`: given `at` 5 min after `now`, when `age` runs, then it displays `now`.
  3. `fr21_60_seconds_is_1_min`: given `at` 60 s before `now`, then `1 min`.
  4. `fr21_59_min_59_s_is_59_min`: given `at` 59 min 59 s before `now`, then `59 min`.
  5. `fr21_60_min_is_1_h`: given `at` 60 min before `now`, then `1 h`.
  6. `fr21_23_h_59_min_is_23_h`: given `at` 23 h 59 min before `now`, then `23 h`.
  7. `fr21_24_h_is_1_d`: given `at` 24 h before `now`, then `1 d`.
  8. `fr21_42_h_47_min_is_1_d`: given `at` 42 h 47 min before `now`, then `1 d`.
  9. `fr21_missing_stamp_is_question_mark`: given `at` `None`, then `?`.
  10. `fr21_age_compares_instants_across_offsets`: given `at` `2026-10-08T10:31:05+03:00` and `now` `2026-10-08T08:10:05+00:00`, then `39 min`.
  11. `ts6_7_age_minutes_floors_and_clamps`: given `at` 39 min 59 s before `now`, then `Some(39)`; given a future `at`, then `Some(0)`; given `None`, then `None`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Each 7.3 boundary (60 s, 60 min, 24 h) has a test just below and at the boundary.

### E1-F1-T6 Env and XDG paths

- Status: todo
- Depends on: E1-F1-T2, E1-F1-T4
- Covers: NFR-09; TECHSPEC 3 (local zone for new stamps), 5.1, 6.1, 9.1 (detection accessor), 9.4 (binary path), 10 (lock and hint name encoding), T-08, T-09, T-21
- Size: M
- Scope: `src/core/env.rs` with `#[derive(Clone, Debug)] pub struct Env { cwd: PathBuf, home: PathBuf, config_home: PathBuf, data_home: PathBuf, state_home: PathBuf, visual: Option<String>, editor: Option<String>, herdr_env: Option<String>, herdr_bin_path: Option<PathBuf>, herdr_pane_id: Option<String>, herdr_tab_id: Option<String>, herdr_workspace_id: Option<String>, session: Option<String>, log: Option<PathBuf>, claude_config_dir: Option<PathBuf>, exe: Option<PathBuf>, pid: u32, tz: TimeZone, now: Stamp }` (all `pub`; E1-F1-T7 adds `failpoint`). `Env` derives `Clone`, so an adapter can own a copy (E5-F1-T1 `ProcessRunner`). `exe` is the canonical path of the running binary: `Env::from_vars` and `core_env` leave it `None`. `Env::from_vars(vars: impl IntoIterator<Item = (String, String)>, cwd: PathBuf, pid: u32, tz: TimeZone, clock: Stamp) -> Result<Env>` reads only the 6.1 names: `BRAIN_SWAP_NOW` (RFC 3339) replaces `clock`, an invalid value fails `Error::InvalidInput("BRAIN_SWAP_NOW: invalid stamp '<v>'")` (A-E1-10); `now` is the instant of `BRAIN_SWAP_NOW` (or of `clock`) converted to `tz` (`.timestamp().to_zoned(tz.clone())`), so every new stamp taken from it is in the local zone (section 3); `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` count only when absolute, else `$HOME/.config`, `$HOME/.local/share`, `$HOME/.local/state`, on every platform; an unset or empty `HOME` fails `Error::Io("HOME not set")` (A-E1-11). Path accessors: `config_file()` (`<config_home>/brain-swap/config.toml`), `templates_dir()`, `data_dir()` (`<data_home>/brain-swap`), `default_board_dir(name)`, `state_dir()`, `sessions_dir()`, `panes_dir()`, `locks_dir()`, `edit_dir()`; `herdr_active()` is true only for `HERDR_ENV=1`. `encode_file_name(&str) -> String` replaces `%` with `%25`, then `/` with `%2F` (10, 6.4). `src/main.rs` builds the value from `std::env::vars()`, `std::env::current_dir()`, `std::process::id()`, `TimeZone::system()` and `Timestamp::now()`, then sets `exe` once with `std::env::current_exe().and_then(std::fs::canonicalize).ok()`. Add `tests/common/env.rs` with `core_env(home: &Path, now: &str) -> Env`, whose `tz` is the fixed offset of `now` (so `format_stamp(&env.now)` gives `now` back unchanged in every core test), so core tests never read the process environment.
- Not in scope: the failpoint field (E1-F1-T7); FR-58 behaviour of the herdr adapter (E5-F1); session resolution from these fields (E1-F7-T1).
- Tests first:
  1. `nfr09_xdg_defaults_under_home_on_every_platform`: given vars `HOME=/h` only, when `Env::from_vars` runs, then `config_file()` is `/h/.config/brain-swap/config.toml`, `templates_dir()` `/h/.config/brain-swap/templates`, `default_board_dir("work")` `/h/.local/share/brain-swap/work`, and `sessions_dir()`, `panes_dir()`, `locks_dir()`, `edit_dir()` are `sessions`, `panes`, `locks`, `edit` under `/h/.local/state/brain-swap`.
  2. `ts5_1_absolute_xdg_values_override`: given `XDG_CONFIG_HOME=/x/c`, `XDG_DATA_HOME=/x/d` and `XDG_STATE_HOME=/x/s`, when built, then the paths lie under `/x/c/brain-swap`, `/x/d/brain-swap` and `/x/s/brain-swap`.
  3. `ts5_1_relative_or_empty_xdg_is_ignored`: given `XDG_STATE_HOME=rel` and `XDG_DATA_HOME=` with `HOME=/h`, when built, then the state and data paths are the defaults under `/h`.
  4. `ts6_1_brain_swap_now_overrides_clock_in_local_zone`: given `BRAIN_SWAP_NOW=2026-10-08T11:52:00+03:00`, tz UTC and a clock of `2030-01-01T00:00:00+00:00`, when built, then `format_stamp(&env.now)` is `2026-10-08T08:52:00+00:00`.
  5. `ts6_1_invalid_brain_swap_now_is_invalid_input`: given `BRAIN_SWAP_NOW=tomorrow`, when built, then the result is `Err(Error::InvalidInput(_))` with exit code 1.
  6. `ts6_1_missing_home_is_io_error`: given no `HOME`, when built, then the result is `Err(Error::Io("HOME not set"))`.
  7. `ts6_1_listed_variables_land_in_fields`: given `VISUAL`, `EDITOR`, `BRAIN_SWAP_SESSION`, `BRAIN_SWAP_LOG`, `CLAUDE_CONFIG_DIR`, `HERDR_ENV`, `HERDR_BIN_PATH`, `HERDR_PANE_ID`, `HERDR_TAB_ID` and `HERDR_WORKSPACE_ID`, when built, then each value is in its field.
  8. `ts6_1_claude_session_variable_is_not_read`: given `CLAUDE_SESSION_ID=abc` and no `BRAIN_SWAP_SESSION`, when built, then `env.session` is `None` (T-09).
  9. `ts9_1_herdr_active_only_when_herdr_env_is_1`: given `HERDR_ENV=1`, then `herdr_active()` is true; given `0`, `true` or no `HERDR_ENV`, then false.
  10. `ts10_encode_file_name_escapes_percent_then_slash`: given `/home/a%b/work`, when encoded, then the result is `%2Fhome%2Fa%25b%2Fwork`.
  11. `ts3_clock_is_converted_to_local_zone`: given no `BRAIN_SWAP_NOW`, a clock of `2026-10-08T08:52:00+00:00` and tz fixed `+03:00`, when built, then `format_stamp(&env.now)` is `2026-10-08T11:52:00+03:00`.
  12. `ts3_core_env_keeps_offset_of_now`: given `core_env(home, "2026-10-08T11:52:00+03:00")`, when its fields are read, then `format_stamp(&env.now)` is `2026-10-08T11:52:00+03:00` and `env.tz` is fixed `+03:00`.
  13. `ts9_4_from_vars_and_core_env_leave_exe_unset`: given vars `HOME=/h` only, when `Env::from_vars` runs, then `env.exe` is `None`, and given `core_env(home, "2026-10-08T11:52:00+03:00")`, then its `exe` is `None` too.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/main.rs` is the only file that calls `std::env::vars`, `std::env::current_dir`, `std::env::current_exe` or `TimeZone::system` (grep).

### E1-F1-T7 Failpoints

- Status: todo
- Depends on: E1-F1-T2, E1-F1-T6
- Covers: TECHSPEC 12.1 (crash, failpoint names), 6.1 (`BRAIN_SWAP_FAILPOINT`), T-21
- Size: S
- Scope: `src/core/failpoint.rs`, compiled always, active only with feature `failpoints`: `#[derive(Clone, Debug, PartialEq, Eq)] pub struct Failpoint { pub name: String, pub abort: bool }` (`Clone` so `Env` stays `Clone`, E1-F1-T6), field `Env.failpoint: Option<Failpoint>`, `pub const NAMES: [&str; 5] = ["create:after_next", "create:after_tmp", "create:after_link", "park:after_tmp", "edit:after_tmp"]` and `pub fn check(env: &Env, name: &str) -> Result<()>`. With the feature, when `env.failpoint` names `name`, `check` returns `Err(Error::Io(format!("failpoint {name}")))` if `abort` is false and calls `std::process::abort()` if true; without the feature it always returns `Ok(())`. `Env::from_vars` reads `BRAIN_SWAP_FAILPOINT` only with the feature, as `Failpoint { abort: true }` (process level, 12.1); in-process tests build `Failpoint { abort: false }` themselves.
- Not in scope: placing the checks in the store (E1-F6-T2, E1-F6-T5, E1-F6-T7); process-level abort tests (E2-F3 for `create:*` and `park:*`, E4-F5 for `edit:after_tmp`).
- Tests first:
  1. `ts12_1_named_failpoint_returns_io_error`: given feature `failpoints` and `env.failpoint = Failpoint { name: "park:after_tmp", abort: false }`, when `check(&env, "park:after_tmp")` runs, then it returns `Err(Error::Io("failpoint park:after_tmp"))`.
  2. `ts12_1_other_failpoint_name_passes`: given the same env, when `check(&env, "create:after_tmp")` runs, then it returns `Ok(())`.
  3. `ts12_1_no_failpoint_passes`: given `env.failpoint = None`, when `check` runs for every name in `NAMES`, then each returns `Ok(())`.
  4. `ts12_1_env_reads_failpoint_with_abort`: given feature `failpoints` and vars with `BRAIN_SWAP_FAILPOINT=park:after_tmp`, when `Env::from_vars` runs, then `env.failpoint` is `Some(Failpoint { name: "park:after_tmp", abort: true })`.
  5. `ts12_1_failpoint_ignored_without_feature`: given a build without `failpoints` (`#[cfg(not(feature = "failpoints"))]`) and `BRAIN_SWAP_FAILPOINT=park:after_tmp`, when the env is built and `check` runs, then `env.failpoint` is `None` and `check` returns `Ok(())`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `cargo test` without features also passes, which runs test 5.

### E1-F1-T8 Log sink

- Status: todo
- Depends on: E1-F1-T6
- Covers: FR-05; TECHSPEC 11 (logging)
- Size: S
- Scope: `src/core/log.rs`, a module 2.1 does not list (A-E1-08). `pub fn event(env: &Env, kind: &str, text: &str)` appends one line `<format_stamp(&env.now)> <kind> <text>` to `env.log` (append mode, created when missing) and does nothing when `env.log` is `None`; write errors are ignored, so logging never fails a command. `pub fn guard_board_folders(env: &mut Env, boards: &[(String, PathBuf)]) -> Option<String>`: when `env.log` lies inside a board folder (component-wise prefix), it sets `env.log = None` and returns `BRAIN_SWAP_LOG inside board <name>, ignored` (A-E1-09); `cli::cmd::Ctx::load` (E2-F1-T4, for every subcommand that loads the config, `context` included) and `tui::prepare` (E4-F1-T8) call it. Kinds used later: `herdr` (E5-F1), `lock_wait` (E1-F6-T1), `reload` (E4-F1), `merge` (E1-F6-T7), `verify_failed` (E1-F6-T2).
- Not in scope: emitting those events (the tasks named above); calling the guard and printing its warning (E2-F1-T4, E4-F1-T8).
- Tests first:
  1. `ts11_event_appends_one_line_per_call`: given `env.log` pointing at `<tmp>/bs.log` and `env.now` `2026-10-08T11:52:00+03:00`, when `event` is called twice with `lock_wait` and `/b 150`, then the file holds exactly two lines, each `2026-10-08T11:52:00+03:00 lock_wait /b 150`.
  2. `ts11_no_log_when_unset`: given `env.log = None` and an empty temp dir as cwd and home, when `event` runs, then no file is created anywhere in the temp dir.
  3. `ts11_event_keeps_existing_lines`: given a log file holding `old`, when `event` runs once, then the file holds `old` followed by one new line.
  4. `fr05_log_inside_board_folder_is_ignored_with_warning`: given boards `[("work", <tmp>/work)]` and `env.log = <tmp>/work/bs.log`, when `guard_board_folders` runs, then it returns `Some("BRAIN_SWAP_LOG inside board work, ignored")`, `env.log` is `None`, and a following `event` creates nothing under `<tmp>/work`.
  5. `fr05_log_outside_board_folders_is_kept`: given `env.log = <tmp>/state/bs.log` and the same boards, when `guard_board_folders` runs, then it returns `None` and `env.log` is unchanged.
  6. `ts11_unwritable_log_does_not_fail`: given `env.log` pointing at an existing directory, when `event` runs, then it returns without panicking.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `event` takes no lock and contains no `unwrap` or `expect`.

### E1-F1-T9 Hermetic spawn helper

- Status: done
- Depends on: E1-F1-T1
- Covers: TECHSPEC 12.1 (hermetic tests, the one spawn helper)
- Size: M
- Scope: `tests/common/mod.rs` and `tests/common/spawn.rs`. `pub struct Scenario` built by `Scenario::new(tmp: &Path)` sets `HOME=<tmp>/home`, `XDG_CONFIG_HOME=<tmp>/home/.config`, `XDG_DATA_HOME=<tmp>/home/.local/share`, `XDG_STATE_HOME=<tmp>/home/.local/state`, `TZ=UTC`, `BRAIN_SWAP_NOW=2026-10-08T11:52:00+03:00`, `PATH=/usr/bin:/bin` and `HERDR_BIN_PATH=FAKE_HERDR` (`<CARGO_MANIFEST_DIR>/tests/fake_herdr/herdr`) always (A-E1-13). Setters: `herdr(pane, tab, workspace)` (adds `HERDR_ENV=1`, `HERDR_PANE_ID`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID`), `fake_scenario(name)` (`FAKE_HERDR_SCENARIO`), `failpoint(name)`, `session(id)`, `log(path)`, `visual(cmd)`, `editor(cmd)`, `now(stamp)`, and for the variables other epics override: `xdg(config, data, state)` replaces the three `XDG_*` values (E2-F1's `Sandbox`), `unset(name)` drops one of `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` and panics for any other name (so `HERDR_BIN_PATH` always stays the fake script, 12.1), and `path(value)` replaces `PATH` (E3-F1's `run_sh`). `vars(&self) -> Vec<(String, String)>` returns exactly the variables `command` sets, sorted by name (E4-F5's scripted editor). `Scenario::new` creates `<tmp>/home` (the fake herdr script logs to `$HOME/fake-herdr.log`) and no other folder. `pub fn command(s: &Scenario, program: &Path) -> std::process::Command` calls `env_clear()` and sets only those variables; `pub fn brain_swap(s: &Scenario) -> assert_cmd::Command` applies the same to `cargo_bin("brain-swap")`. Create `tests/fake_herdr/herdr`, an executable `sh` stub that prints `fake herdr: no scenario` to stderr and exits 1 until E5-F1 replaces it. Add a rule to `tests/layering.rs`: in `tests/`, `Command::new` and `cargo_bin` appear only in `tests/common/spawn.rs`; the rule skips `tests/layering.rs` itself, whose planted snippets hold those names as string literals.
- Not in scope: the fake herdr scenarios and argv log (E5-F1); per-subcommand CLI tests (E2).
- Tests first:
  1. `ts12_1_child_sees_only_listed_variables`: given a `Scenario` with no setters, when `/usr/bin/env` runs through `command`, then the printed variable names are exactly `PATH`, `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `TZ`, `BRAIN_SWAP_NOW` and `HERDR_BIN_PATH`, whatever the parent process environment holds.
  2. `ts12_1_herdr_setter_adds_herdr_variables`: given `.herdr("w4V:p9", "w4V:t1", "w4V")`, when `/usr/bin/env` runs, then the output also holds `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p9`, `HERDR_TAB_ID=w4V:t1` and `HERDR_WORKSPACE_ID=w4V`.
  3. `ts12_1_optional_variables_only_when_set`: given each of the setters `fake_scenario`, `failpoint`, `session`, `log`, `visual` and `editor` used alone, when `/usr/bin/env` runs, then exactly that one extra variable appears; given `xdg(<tmp>/c, <tmp>/d, <tmp>/s)`, then only the three `XDG_*` values differ from the defaults; given `unset("XDG_CONFIG_HOME")`, then only that variable is missing; given `path("/opt/bin:/usr/bin:/bin")`, then only `PATH` differs.
  4. `ts12_1_tests_spawn_only_through_helper`: given a temp tree with `Command::new("x")` in `tests/cli_new.rs`, when the `tests/` rule of the layering scanner runs, then it reports that file and line, and on the repository `tests/` (whose `tests/layering.rs` holds the planted snippets) it reports nothing.
  5. `ts12_1_scenario_creates_home`: given `Scenario::new(tmp)` on an empty temp dir, when it returns, then `<tmp>/home` exists and is the only new entry under `<tmp>`.
  6. `ts12_1_vars_match_child_environment`: given a `Scenario` with `.herdr("w4V:p9", "w4V:t1", "w4V")`, `.session("s1")` and `.unset("XDG_STATE_HOME")`, when `vars()` is compared with the pairs `/usr/bin/env` prints through `command`, then both hold the same pairs.
  7. `ts12_1_unset_refuses_non_xdg_names`: given `Scenario::new(tmp)`, when `unset("HERDR_BIN_PATH")` is called, then it panics (`#[should_panic]`).
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `tests/fake_herdr/herdr` is executable in git (`git ls-files -s` shows mode 100755).

### E1-F1-T10 Docs test

- Status: done
- Depends on: E1-F1-T1
- Covers: TECHSPEC 12.1 (docs), 12.2 item 5
- Size: S
- Scope: `tests/docs.rs` walks the repository from `CARGO_MANIFEST_DIR`, skipping `target/` and every directory whose name starts with `.` (`.git/`, and local folders such as `.serena/` or `.claude/` that a clean clone lacks, so the result is the same on every machine), reads every `*.md` file as bytes (including `backlog/`, `pack/`, `templates/` and test fixtures, among them the non-UTF-8 `tests/fixtures/cards/non_utf8.md`), scans `String::from_utf8_lossy` of each and fails naming `file:line` for each U+2014 or U+2013. Helpers `find_dashes(text: &str) -> Vec<usize>` (1-based line numbers) and `scan(root: &Path) -> Vec<(PathBuf, usize)>` let tests run on planted text and temp trees; planted dashes are written as `\u{2014}` and `\u{2013}` escapes.
- Not in scope: `--help` and README content (each task's 12.2 item 5).
- Tests first:
  1. `ts12_1_find_dashes_reports_em_dash_line`: given the text `a`, newline, `b` U+2014 `c`, newline, when `find_dashes` runs, then it returns `[2]`.
  2. `ts12_1_find_dashes_reports_en_dash_line`: given an en dash on line 1, when `find_dashes` runs, then it returns `[1]`.
  3. `ts12_1_hyphen_is_allowed`: given `a - b`, when `find_dashes` runs, then it returns `[]`.
  4. `ts12_1_repository_markdown_has_no_dashes`: given every markdown file of the repository, when checked, then none is reported.
  5. `ts12_1_non_utf8_markdown_is_scanned_lossily`: given a temp tree with `x.md` holding a 0xFF byte on line 1 and U+2014 on line 2, when `scan` runs, then it reports `x.md:2` and nothing else.
  6. `ts12_1_dot_directories_are_skipped`: given a temp tree with `.serena/m.md` holding U+2014, when `scan` runs, then nothing is reported.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The test fails on a planted dash in `backlog/E1-core.md` (tried locally, not committed).

## E1-F2 Card grammar

- Depends on: E1-F1
- Covers: FR-01, FR-07, FR-14, FR-16, FR-17, FR-18, NFR-06, NFR-07
- Spec: TECHSPEC 3 (Card, Note, Place, HerdrPlace, I2, I6, I7), 4.2, 4.4, 4.7, 4.8 (R1 to R5), 4.9, 12.1 (round trip)
- Scope: The hand-parsed flat frontmatter of 4.2 with reading and a one-line `set` splice, the card file grammar of 4.4 with its tolerant reader (bold labels, local stamps, bad stamps, quoted places, CRLF, missing frontmatter or timeline, non-UTF-8 and conflict markers), and the three writers every later write uses: rendering a new card (R4), appending a note (R3) and setting the column (R2), all as splices that keep every other byte (R1, R5, NFR-07). A round-trip suite over `tests/fixtures/cards/` pins each splice per fixture with insta snapshots. No file system access: every function takes and returns bytes.
- Provides: `core::frontmatter::read(&str) -> FmRead` (`None`, `Valid(Frontmatter)`, `Invalid { line: usize, problem: String }`), `Frontmatter::get(&self, key) -> Option<&str>`, `core::frontmatter::set(bytes: &[u8], key: &str, value: &str) -> Result<Vec<u8>>`, `core::frontmatter::line_ending(bytes: &[u8]) -> &'static str`; `core::model::{Card, Note, NewCard, NewNote}` with the section 3 fields plus `Card.mtime: Option<Stamp>` (A-E1-05); `core::card_file::parse(id: CardId, path: &Path, bytes: &[u8], default_column: &str, tz: &TimeZone) -> Parsed { card: Card, warnings: Vec<String> }`; `core::card_file::note_blocks(text: &str) -> Vec<Range<usize>>`; `core::card_file::check_writable(name: &str, bytes: &[u8]) -> Result<()>`; `core::card_file::render_new(&NewCard) -> (Vec<u8>, Vec<String>)`; `core::card_file::render_note(&NewNote, eol: &str) -> (String, Vec<String>)`; `core::card_file::append_raw(name: &str, bytes: &[u8], block: &str) -> Result<Appended>`; `core::card_file::append_note(name: &str, bytes: &[u8], note: &NewNote) -> Result<Appended>` with `Appended { bytes: Vec<u8>, inserted: Range<usize>, warnings: Vec<String> }`; `core::card_file::set_column(name: &str, bytes: &[u8], column: &str) -> Result<Vec<u8>>`; fixtures `tests/fixtures/cards/*.md`
- Requires: E1-F1 (`Error`, `CardId`, `Place`, `HerdrPlace`, `Stamp`, `parse_stamp`, `format_stamp`)
- Feature DoD:
  - [ ] Every fixture round-trips byte for byte: each splice changes only its line or its inserted block (E1-F2-T8).
  - [ ] Parsing never panics on any byte prefix of any fixture (E1-F2-T5).
  - [ ] A created card parses back to the rendered values (R4) (E1-F2-T6).

### E1-F2-T1 Frontmatter reader

- Status: todo
- Depends on: E1-F1-T2
- Covers: TECHSPEC 4.2
- Size: M
- Scope: `src/core/frontmatter.rs`: `pub struct Entry { pub key: String, pub value: String, pub line: usize }`, `pub struct Frontmatter { pub entries: Vec<Entry>, pub close_line: usize, pub end: usize }` (`end` is the byte offset after the closing `---` line), `pub enum FmRead { None, Valid(Frontmatter), Invalid { line: usize, problem: String } }`, `pub fn read(text: &str) -> FmRead` and `Frontmatter::get(&self, key: &str) -> Option<&str>` (first entry whose key matches ignoring case). A block exists only when the text starts at byte 0 with a `---` line; `key := [A-Za-z0-9_-]+`, the value is the rest after `:` and one space, with one pair of surrounding `"` or `'` stripped; an indented line or one starting with `- ` continues the previous key and is kept verbatim (not part of the value); `#` comment lines and blank lines are kept; any other line, or a missing closing `---`, makes the block `Invalid` with its 1-based line. LF and CRLF line ends are accepted.
- Not in scope: writing (E1-F2-T2); which keys a card or `board.md` uses (E1-F2-T3, E1-F5-T1); turning `Invalid` into warnings (E1-F2-T5).
- Tests first:
  1. `ts4_2_reads_key_value_pairs`: given `---\ncolumn: Doing\ntemplate: Feature\n---\n# t\n`, when `read` runs, then it is `Valid` and `get("column")` is `Doing` and `get("template")` is `Feature`.
  2. `ts4_2_keys_match_ignoring_case`: given a block with `Column: Doing`, when `get("column")` is called, then it returns `Doing`.
  3. `ts4_2_strips_one_pair_of_quotes`: given `column: "In review"` and `title: ""x""`, when read, then the values are `In review` and `"x"`.
  4. `ts4_2_key_without_value_is_empty`: given `template:`, when read, then `get("template")` is the empty string.
  5. `ts4_2_continuation_lines_belong_to_previous_key`: given `tags:`, `  - a` and `- b` inside a block, when read, then it is `Valid` with one entry `tags` whose value is empty.
  6. `ts4_2_comments_and_blank_lines_are_allowed`: given a block with `# note` and an empty line between keys, when read, then it is `Valid` with both keys.
  7. `ts4_2_block_only_at_byte_0`: given `\n---\ncolumn: x\n---\n`, when read, then it returns `FmRead::None`.
  8. `ts4_2_invalid_line_invalidates_block`: given `---\ncolumn Doing\n---\n`, when read, then it returns `Invalid { line: 2, .. }`.
  9. `ts4_2_unclosed_block_is_invalid`: given `---\ncolumn: Doing\n# t\n` with no closing line, when read, then it returns `Invalid`.
  10. `ts4_2_crlf_block_reads`: given test 1's text with CRLF line ends, when read, then the values equal test 1's and carry no `\r`.
  11. `ts4_2_text_without_block_is_none`: given `# title\n`, when read, then it returns `FmRead::None`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `read` returns without panicking on every byte prefix of each test input (checked inside test 1).

### E1-F2-T2 Frontmatter splice

- Status: todo
- Depends on: E1-F2-T1
- Covers: NFR-07; TECHSPEC 4.2 (setting a key), 4.8 R1, R2, R5
- Size: M
- Scope: `frontmatter::set(bytes, key, value) -> Result<Vec<u8>>`: the first line whose key matches ignoring case is replaced by `<key as spelled in the file>: <value>` (A-E1-15); an absent key gets a line inserted just before the closing `---`; text without a block gets `---`, `<key>: <value>`, `---` prepended; an `Invalid` block returns `Err(Error::Unreadable("<line>: <problem>"))` (callers prefix the file name). Inserted lines use `line_ending(bytes)`: CRLF when CRLF line ends outnumber bare LF ones, else LF (also on a tie and in a file without line ends, A-E1-15). Every other byte stays identical.
- Not in scope: card-specific use (`set_column`, E1-F2-T8) and `board.md` `next:` (E1-F6-T5); final newline before an appended note block (E1-F2-T7).
- Tests first:
  1. `r2_set_replaces_existing_line_only`: given `---\ncolumn: Todo\ntemplate: Feature\n---\nbody\n`, when `set(.., "column", "Doing")` runs, then the output equals the input with line 2 replaced by `column: Doing`.
  2. `r2_set_keeps_key_spelling`: given a block with `Column: Todo`, when `column` is set to `Doing`, then the line reads `Column: Doing`.
  3. `r2_set_inserts_before_closing_line`: given a block with `template: Feature` only, when `column` is set to `Doing`, then `column: Doing` is inserted as the line before the closing `---` and all other bytes are unchanged.
  4. `r2_set_prepends_three_line_block`: given `# t\nbody\n`, when `column` is set to `Doing`, then the output is `---\ncolumn: Doing\n---\n# t\nbody\n`.
  5. `r5_inserted_line_uses_crlf_in_crlf_file`: given test 3's input with CRLF line ends, when `column` is set, then the inserted line ends with `\r\n` and the rest is unchanged.
  6. `r5_prepended_block_uses_dominant_line_ending`: given `# t\r\nbody\r\n`, when `column` is set, then the three prepended lines end with `\r\n`.
  7. `nfr07_unknown_keys_comments_and_continuations_survive`: given a block with `owner: me`, `# c`, `tags:` and `  - x` around `column: Todo`, when `column` is set to `Done`, then only the column line differs.
  8. `ts4_2_set_on_invalid_block_is_unreadable`: given `---\ncolumn Doing\n---\n`, when `set` runs, then it returns `Err(Error::Unreadable(m))` with `m` starting `2: `.
  9. `ts4_2_line_ending_tie_is_lf`: given one CRLF and one LF line, then `line_ending` is `\n`; given text without line ends, then `\n`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Each test compares bytes outside the changed or inserted lines with a shared helper `tests/common/bytes.rs` `assert_only_lines_changed(before, after, expected_lines)`.

### E1-F2-T3 Card model and canonical parse

- Status: todo
- Depends on: E1-F2-T1, E1-F1-T3, E1-F1-T4
- Covers: FR-01, FR-07, FR-14, FR-17; TECHSPEC 3 (Card, Note, Place, HerdrPlace, I7), 4.4, 4.9, 7.6 (merge identity)
- Size: M
- Scope: Add to `src/core/model.rs`: `Card { id, path, title, column, template: Option<String>, created: Option<Stamp>, body, notes: Vec<Note>, writable: bool, mtime: Option<Stamp> }` (A-E1-05; `parse` sets `mtime: None`, E1-F5-T2 fills it), `Note { heading, at, auto, doing, next, watch_out, place, extra }` exactly as section 3 (with `Place` and `HerdrPlace` of E1-F1-T3). `src/core/card_file.rs` `parse(id, path, bytes, default_column, tz) -> Parsed`: keys `column` (missing: `default_column`), `template`, `created`; the title is the first `# ` line outside code fences (```` ``` ```` or `~~~`), else `(untitled)`; the body is the text between the title line and the `## Timeline` line; the timeline runs to the post (the next `# ` or `## ` heading outside fences); each `### <stamp>[ auto]` heading starts a note; parts are `["- "] label ":" [" " text]` with labels Doing, Next, Watch out in any case; two-space continuation lines join their part with `\n`; the place line is `<!-- where: k=v ... -->` with bare values; other lines go to `extra`; notes keep file order (I7). The card ID always comes from the `id` argument (FR-01, I2). `note_blocks(text) -> Vec<Range<usize>>` gives each note's byte range from its heading to its last non-blank line (A-E1-20). Create fixtures `tests/fixtures/cards/w12_4_9.md`, `w9_4_9.md`, `w11_4_9.md` (the three cards of 4.9) and `order_by_file.md` (two notes, the second with an earlier stamp).
- Not in scope: tolerant note forms (E1-F2-T4) and file-level tolerance (E1-F2-T5); writing (E1-F2-T6 to T8); column resolution against `board.md` and extra columns (E1-F5-T3).
- Tests first:
  1. `fr07_parse_reads_title_column_template_created`: given `w12_4_9.md`, when parsed with id `W-12` and default column `Todo`, then the title is `migrate invoices to v13`, column `Doing`, template `Some("Feature")` and `format_stamp(created)` is `2026-10-08T10:02:11+03:00`.
  2. `fr07_body_lies_between_title_and_timeline`: given `w12_4_9.md`, when parsed, then the body contains `## Goal`, `## Acceptance` and `The DBA applies the procedure on staging; we hand over the SQL only.` and contains neither the title line nor `## Timeline`.
  3. `fr14_note_has_parts_stamp_auto_and_place`: given `w12_4_9.md`, when parsed, then `notes[0]` has stamp `2026-10-08T10:31:05+03:00`, `auto` true, Doing `switched InvoiceRepository to v13, unit tests green`, Next `rerun the migration test against the staging copy`, Watch out `v13 needs a 10 s timeout entry, the default is 2 s`, cwd `/home/smilen/Work/ati.billing`, pane `w4V:p9`, tab `w4V:t1`, workspace `w4V` and session `55583127-a22a-49bc-a803-e777c377a595`.
  4. `fr14_note_without_auto_marker`: given `w12_4_9.md`, when parsed, then `notes[1].auto` is false and its heading is `### 2026-10-08T11:12:40+03:00`.
  5. `fr17_notes_keep_file_order_whatever_stamp`: given `order_by_file.md`, when parsed, then `notes[1]` is the second note in the file although its stamp is earlier than `notes[0]`'s.
  6. `fr01_id_comes_from_file_name_not_content`: given `w12_4_9.md` with its title changed to `# W-7 migrate`, when parsed with id `W-12`, then `card.id` is `W-12`.
  7. `ts4_4_place_with_cwd_only`: given `w9_4_9.md`, when parsed, then the note's place has cwd `/home/smilen/Work/private/brain-swap`, `herdr` `None` and `session` `None`.
  8. `ts4_4_continuation_joins_part`: given a note with `- Next: a` followed by `  b`, when parsed, then Next is `a\nb`.
  9. `ts4_4_missing_column_uses_default`: given `w12_4_9.md` without its `column:` line, when parsed with default column `Todo`, then the column is `Todo`.
  10. `ts4_4_title_skips_fenced_heading`: given a card whose first `# ` line is inside a fenced block and whose next `# real` line is outside, when parsed, then the title is `real`.
  11. `ts4_4_untitled_without_title_line`: given a card with frontmatter and body but no `# ` line, when parsed, then the title is `(untitled)`.
  12. `ts4_4_empty_timeline_has_no_notes`: given `w11_4_9.md`, when parsed, then the title is `invoice PDF shows the wrong VAT` and `notes` is empty.
  13. `ts7_6_note_blocks_span_heading_to_place_line`: given `w12_4_9.md`, when `note_blocks` runs, then it returns two ranges, the first slice starting `### 2026-10-08T10:31:05+03:00 auto` and ending with `-->`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] An insta snapshot of the parsed `Card` (Debug) of each 4.9 fixture is committed.

### E1-F2-T4 Tolerant note reading

- Status: todo
- Depends on: E1-F2-T3
- Covers: FR-14; TECHSPEC 4.4 (reader tolerance, place quoting), 4.7 (bad stamps, notes without place)
- Size: M
- Scope: Extend `card_file::parse` for: bold labels (`- **Doing:** x` and `- **Doing**: x`); stamps `YYYY-MM-DD HH:MM[:SS]` read in `tz`; a bad stamp gives `at: None` and the note is still listed; quoted place values with `\"` and `\\` escapes; unknown place keys skipped in the model (the bytes stay in the file); a note without a place line has `Place::default()`; `auto` only as the exact word after the stamp; other lines in `extra`. Create fixtures `bold_labels.md`, `local_stamps.md`, `bad_stamps.md` and `quoted_places.md` in `tests/fixtures/cards/`.
- Not in scope: writing quoted values (E1-F2-T7); rendering `?` ages (E1-F1-T5, E4-F2); jump target rules (E1-F5-T3).
- Tests first:
  1. `ts4_4_bold_labels_read_as_parts`: given `bold_labels.md` with `- **Doing:** a`, `- **Next**: b` and `**Watch out:** c`, when parsed, then the parts are `a`, `b` and `c`.
  2. `ts4_4_labels_match_any_case`: given `- doing: a`, `- NEXT: b` and `- WATCH OUT: c`, when parsed, then the parts are `a`, `b` and `c`.
  3. `ts4_4_local_stamp_without_offset_uses_given_zone`: given `local_stamps.md` with `### 2026-10-08 10:31` and `tz` fixed `+03:00`, when parsed, then `format_stamp(at)` is `2026-10-08T10:31:00+03:00`.
  4. `ts4_7_bad_stamp_note_has_no_time`: given `bad_stamps.md` with `### someday`, when parsed, then that note has `at: None` and its three parts are read.
  5. `ts4_4_quoted_place_value_unescapes`: given `quoted_places.md` with `cwd="/home/a b/\"q\"\\x"`, when parsed, then cwd is the path `/home/a b/"q"\x`.
  6. `ts4_4_unknown_place_key_is_skipped`: given `<!-- where: host=x cwd=/a -->`, when parsed, then cwd is `/a` and herdr and session are `None`.
  7. `ts4_7_note_without_place_has_empty_place`: given a note with three parts and no place line, when parsed, then its place equals `Place::default()`.
  8. `ts4_4_other_lines_go_to_extra`: given a note with the line `free text` between parts, when parsed, then `extra` holds `free text` and the parts are intact.
  9. `ts4_4_auto_marker_only_as_exact_word`: given the heading `### 2026-10-08T10:31:05+03:00 automatic`, when parsed, then `auto` is false and `at` is set.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] An insta snapshot of the parsed notes of each of the four new fixtures is committed.

### E1-F2-T5 Tolerant file reading

- Status: todo
- Depends on: E1-F2-T3
- Covers: FR-07; TECHSPEC 4.2 (invalid block), 4.4 (fences, post), 4.7 (no frontmatter, CRLF, missing timeline, non-UTF-8, conflict markers), 12.1 (parsing never panics)
- Size: M
- Scope: `card_file::parse` sets `writable` and warnings (texts A-E1-16): no frontmatter gives `default_column` and stays writable; an invalid block reads as none, warns `<file>:<line>: invalid frontmatter: <problem>` and sets `writable: false`; non-UTF-8 bytes are parsed from a lossy decode, warn `<file>: not UTF-8, read only` and set `writable: false` (A-E1-17); a line starting `<<<<<<<` warns `<file>:<line>: conflict marker, read only` and sets `writable: false`. CRLF files read like LF files (no `\r` in values); `##` headings inside fences neither start nor end the timeline; a card without `## Timeline` has no notes and its body runs to the end; the post section is outside the timeline. `card_file::check_writable(name, bytes) -> Result<()>` returns `Err(Error::Unreadable(<the warning text>))` for the three read-only cases and is what every splice calls first. `<file>` is the file name of `path`. Create fixtures `no_frontmatter.md`, `invalid_frontmatter.md`, `non_utf8.md`, `conflict_markers.md`, `fenced_headings.md`, `no_timeline.md`, `post_section.md`, `unknown_keys.md`, `hand_edited.md`, `no_final_newline.md` and `crlf.md` (the 4.9 `W-12.md` with CRLF line ends), and add `.gitattributes` with the line `tests/fixtures/cards/* -text`, so git never rewrites the CRLF and non-UTF-8 fixture bytes.
- Not in scope: refusing writes in the store (E1-F6-T3, T4, T7 call `check_writable`); showing warnings (E2, E4).
- Tests first:
  1. `ts4_7_card_without_frontmatter_is_in_default_column`: given `no_frontmatter.md`, when parsed with default column `Todo`, then the column is `Todo`, `writable` is true and there is no warning.
  2. `ts4_2_invalid_frontmatter_reads_as_none_and_is_read_only`: given `invalid_frontmatter.md` (line 2 `column Doing`), when parsed, then the column is the default, `writable` is false and the warnings hold `invalid_frontmatter.md:2: invalid frontmatter: ` followed by the problem.
  3. `ts4_7_non_utf8_card_is_read_only`: given `non_utf8.md` (a 0xFF byte in the body), when parsed, then the title is read, `writable` is false and the warnings hold `non_utf8.md: not UTF-8, read only`.
  4. `ts4_7_conflict_markers_make_card_read_only`: given `conflict_markers.md` with `<<<<<<< HEAD` on line 9, when parsed, then `writable` is false and the warnings hold `conflict_markers.md:9: conflict marker, read only`.
  5. `ts4_7_crlf_card_reads_like_lf`: given `w12_4_9.md` converted to CRLF in the test, when parsed, then title, column, body (with LF) and notes equal the LF parse.
  6. `ts4_4_fenced_headings_do_not_split_timeline`: given `fenced_headings.md` with a fenced `## Timeline` in the body and a fenced `## x` inside a note, when parsed, then there is one timeline with all its notes and the body holds the fenced lines.
  7. `ts4_7_card_without_timeline_has_no_notes`: given `no_timeline.md`, when parsed, then `notes` is empty and the body runs to the end of the file.
  8. `ts4_4_post_section_is_outside_timeline`: given `post_section.md` with `# Appendix` after two notes, when parsed, then there are two notes and no note's `extra` holds appendix lines.
  9. `ts4_2_check_writable_refuses_read_only_files`: given `invalid_frontmatter.md`, `non_utf8.md` and `conflict_markers.md`, when `check_writable` runs, then each returns `Err(Error::Unreadable(_))` whose message equals the parse warning, and given `w12_4_9.md`, then `Ok(())`.
  10. `ts12_1_parsing_never_panics_on_any_prefix`: given every byte prefix of every file in `tests/fixtures/cards/`, when parsed, then each call returns.
  11. `ts4_7_crlf_fixture_reads_like_w12`: given `crlf.md` read from disk, when its bytes are checked and it is parsed, then every line ends with `\r\n` and title, column, body (with LF) and notes equal the parse of `w12_4_9.md`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] An insta snapshot of `(card, warnings)` per new fixture is committed.
  - [ ] `git check-attr text tests/fixtures/cards/crlf.md` reports `unset`, and a fresh clone still holds `\r\n` in `crlf.md` and the 0xFF byte in `non_utf8.md`.

### E1-F2-T6 Render a new card

- Status: todo
- Depends on: E1-F2-T3
- Covers: FR-07, NFR-06; TECHSPEC 4.8 R4, 5.4 (new card shape), 6.3 (`## Timeline` in a supplied body), I2
- Size: S
- Scope: `core::model::NewCard { title: String, template: String, column: String, created: Stamp, body: String }` and `card_file::render_new(&NewCard) -> (Vec<u8>, Vec<String>)`: `---`, `column: <column>`, `template: <template>`, `created: <format_stamp(created)>`, `---`, `# <title>`, a blank line, the body with trailing blank lines trimmed (body and its blank line omitted when empty), a blank line, `## Timeline`, LF line ends and a final newline (A-E1-19). A body line that is `## Timeline` is dropped with the warning `## Timeline in body dropped` (A-E1-16), so the card holds exactly one title line and one timeline. The card ID is never written (I2).
- Not in scope: taking the title from a `# ` body line (E2-F2); choosing the skeleton and the column (E1-F6-T5); dropping `## Timeline` from templates (E1-F4-T3).
- Tests first:
  1. `r4_new_card_parses_back_to_rendered_values`: given `NewCard { title: "migrate invoices to v13", template: "Feature", column: "Todo", created: 2026-10-08T10:02:11+03:00, body: "## Goal\n\n## Acceptance\n\n## Context\n" }`, when rendered and parsed with id `W-12`, then title, column, template, created and body equal the input and `notes` is empty.
  2. `r4_new_card_bytes_snapshot`: given the same input, when rendered, then an insta snapshot of the text matches.
  3. `ts6_3_timeline_heading_in_body_dropped_with_warning`: given a body `## Goal\n## Timeline\nx\n`, when rendered, then the text holds exactly one `## Timeline` line (the last), keeps `x`, and the warnings are `["## Timeline in body dropped"]`.
  4. `r4_empty_body_renders_title_then_timeline`: given an empty body, when rendered, then the text after the frontmatter is `# <title>\n\n## Timeline\n`.
  5. `nfr06_new_card_is_utf8_markdown_with_one_title`: given test 1's input, when rendered, then the bytes are valid UTF-8, exactly one line starts with `# ` and none contains `W-12`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The snapshot shows the 4.9 `W-12.md` header shape (frontmatter keys in the order column, template, created).

### E1-F2-T7 Note writer and append

- Status: todo
- Depends on: E1-F2-T2, E1-F2-T4, E1-F2-T5
- Covers: FR-14, FR-16, FR-18, NFR-06, NFR-07; TECHSPEC 3 (I6), 4.4 (writer, place quoting), 4.7 (missing timeline), 4.8 R3, R5, 6.5 (`empty note`)
- Size: M
- Scope: `core::model::NewNote { at: Stamp, auto: bool, doing: String, next: String, watch_out: String, cwd: PathBuf, herdr: Option<HerdrPlace>, session: Option<String> }`, so I6's stamp and cwd hold by construction. `card_file::render_note(&NewNote, eol) -> (String, Vec<String>)`: a blank line, `### <format_stamp(at)>` plus ` auto` when marked, `- Doing: ...`, `- Next: ...`, `- Watch out: ...` (an empty part renders `- Next:`), further lines of a part as two-space continuations, then `<!-- where: cwd=.. pane=.. tab=.. workspace=.. session=.. -->` in that order with absent keys left out; a value with a space, `"` or `\` is quoted with `\"` and `\\` escapes; a value containing `-->` is dropped with the warning `place <key> contains '-->', dropped`. `append_raw(name, bytes, block)`: `check_writable` first; the block (converted to `line_ending(bytes)`) goes right after the last non-blank line of the timeline, before trailing blank lines and the post (A-E1-18); without `## Timeline`, `## Timeline` and the block are appended at the end; a missing final newline is added first (R5). `append_note(name, bytes, note)`: all three parts empty after trimming fails `Error::InvalidInput("empty note")` (FR-18), else `render_note` plus `append_raw`. Both return `Appended { bytes, inserted, warnings }`.
- Not in scope: locking, verify and the atomic write (E1-F6-T4); reading notes from stdin (E2-F3); merging raw blocks into an edited copy (E1-F6-T7 uses `append_raw`).
- Tests first:
  1. `fr16_append_only_adds_lines_inside_timeline`: given `w12_4_9.md` and a `NewNote`, when `append_note` runs, then `bytes[..inserted.start]` and `bytes[inserted.end..]` concatenated equal the input, and the parse shows three notes with column, title and body unchanged.
  2. `fr14_written_note_parses_back`: given a `NewNote` with `auto` true, all three parts, cwd, herdr pane, tab, workspace and session, when appended to `w11_4_9.md` and parsed, then the last note's fields equal the input.
  3. `ts4_4_writer_emits_all_three_parts_even_empty`: given Next empty, when rendered, then the block holds the line `- Next:` with nothing after the colon.
  4. `ts4_4_multiline_part_uses_two_space_continuation`: given Doing `a\nb`, when rendered, then the block holds `- Doing: a` followed by `  b`.
  5. `ts4_4_place_values_with_space_quote_backslash_are_quoted`: given cwd `/a b/"c"\d`, when rendered, then the place line holds `cwd="/a b/\"c\"\\d"` and parsing it gives the same path.
  6. `ts4_4_place_value_with_comment_end_is_dropped`: given session `x-->y`, when rendered, then the place line has no `session=` pair and the warnings hold `place session contains '-->', dropped`.
  7. `fr18_all_empty_parts_rejected`: given parts `""`, `" "` and `"\n"`, when `append_note` runs, then it returns `Err(Error::InvalidInput("empty note"))`.
  8. `r3_note_goes_before_post_section`: given `post_section.md`, when a note is appended, then the block sits after the last note's place line and the bytes from the blank line before `# Appendix` to the end are unchanged.
  9. `r3_missing_timeline_is_appended_at_end`: given `no_timeline.md`, when a note is appended, then the output is the input plus `\n## Timeline\n` plus the block.
  10. `r5_missing_final_newline_added_before_block`: given `no_final_newline.md`, when a note is appended, then the output is the input plus `\n` plus the block.
  11. `r5_block_uses_crlf_in_crlf_card`: given `w12_4_9.md` converted to CRLF, when a note is appended, then every inserted line ends with `\r\n`.
  12. `ts4_7_append_to_read_only_card_is_unreadable`: given `invalid_frontmatter.md`, `non_utf8.md` and `conflict_markers.md`, when `append_note` runs, then each returns `Err(Error::Unreadable(_))`.
  13. `nfr07_bytes_around_block_identical_with_unknown_keys`: given `unknown_keys.md` (unknown frontmatter keys, a comment, a YAML list), when a note is appended, then everything outside `inserted` equals the input.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] An insta snapshot of the block rendered for the 4.9 first note's values matches the 4.9 text byte for byte.

### E1-F2-T8 set_column and the round-trip suite

- Status: todo
- Depends on: E1-F2-T2, E1-F2-T4, E1-F2-T7
- Covers: FR-16, NFR-06, NFR-07; TECHSPEC 4.8 R1, R2, R3, 12.1 (round trip)
- Size: M
- Scope: `card_file::set_column(name, bytes, column) -> Result<Vec<u8>>`: `check_writable`, then `frontmatter::set(bytes, "column", column)`. `tests/roundtrip.rs` runs every fixture under `tests/fixtures/cards/` through both splices (append a fixed `NewNote`; set the column to `Doing`): writable fixtures get an insta snapshot per fixture and splice plus the R2 check (exactly one line changed or inserted, or three lines prepended) and the R3 check (output minus `inserted` equals input); read-only fixtures assert `Err(Error::Unreadable(_))`. R7 runs in the store: E1-F6-T3 and E1-F6-T4 run the move and park verifies over this same fixture set.
- Not in scope: verify before rename (E1-F6-T2 to T5); moves between columns and their rules (E1-F6-T3, E4-F2).
- Tests first:
  1. `r2_set_column_changes_one_line`: given `w12_4_9.md`, when `set_column(.., "Done")` runs, then exactly the `column:` line differs and reads `column: Done`.
  2. `r2_set_column_adds_frontmatter_to_card_without_one`: given `no_frontmatter.md`, when `set_column(.., "Doing")` runs, then the output is `---\ncolumn: Doing\n---\n` plus the input.
  3. `r2_set_column_on_read_only_card_is_unreadable`: given `conflict_markers.md`, when `set_column` runs, then it returns `Err(Error::Unreadable(_))`.
  4. `r1_every_fixture_round_trips_append_byte_for_byte`: given each writable fixture, when a note is appended, then the output minus the inserted block equals the input byte for byte.
  5. `r1_every_fixture_round_trips_set_column_byte_for_byte`: given each writable fixture, when the column is set, then the output differs from the input by one replaced or inserted line or a three-line prepend, and nothing else.
  6. `r3_append_snapshot_per_fixture`: given each writable fixture, when a note is appended, then the insta snapshot named after the fixture matches.
  7. `fr16_append_never_changes_column`: given each writable fixture, when a note is appended and the result parsed, then the column equals the column before.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every fixture under `tests/fixtures/cards/` is picked up by the suite through a directory listing, not a hand-kept list.

## E1-F3 Config and keys

- Depends on: E1-F1
- Covers: FR-02, FR-42, FR-43, FR-44, FR-45, FR-66
- Spec: TECHSPEC 5.1, 5.2, 5.3, 6.2 (unknown board), 10 (config), 13 (`preserve_order`)
- Scope: Parsing `config.toml` (5.2) into a `Config` with the board registry in file order, the editor string and the raw key entries; every 5.2 validation case as an `Error::Config` with file, line and column (FR-45); the first-run write with create-new semantics (FR-66). The key binding syntax of 5.3 matched against a terminal-free key press, the action and mode tables of 5.2, the merge of user entries over defaults with the iterative conflict check, the TitleInput rule and the template hotkey check. The TUI shows the warnings (E4-F1); this feature only returns them.
- Provides: `core::config::Config { path: PathBuf, default_board: String, editor: String, boards: Vec<(String, PathBuf)>, keys: Vec<(String, Vec<String>)> }`; `core::config::parse(text: &str, path: &Path, home: &Path) -> Result<(Config, Vec<String>)>`; `core::config::default_text(&Env) -> String`; `core::config::load(&Env) -> Result<Loaded>` with `Loaded { config: Config, warnings: Vec<String>, created: Option<PathBuf> }`; `Config::board(&self, name: &str) -> Result<(&str, &Path)>`; `core::keys::Key`, `core::keys::Binding`, `core::keys::KeyPress`, `core::keys::parse_binding(&str) -> Result<Binding, String>`, `Binding::matches(&self, &KeyPress) -> bool`, `core::keys::Action` (17 actions of 5.2, `Action::config_name()`), `core::keys::KeyContext` (`Board`, `Detail`, `Picker`, `TitleInput`), `core::keys::KeyMap` with `KeyMap::defaults()`, `KeyMap::build(&[(String, Vec<String>)]) -> (KeyMap, Vec<String>)`, `KeyMap::action(&self, KeyContext, &KeyPress) -> Option<Action>`, `KeyMap::keys(&self, Action) -> &[Binding]`, `KeyMap::check_template_hotkeys(&self, &[(String, char)]) -> (Vec<(String, char)>, Vec<String>)`; fixture `tests/fixtures/config/default.toml`
- Requires: E1-F1 (`Env` paths, `Error::Config`, `Error::NotFound`)
- Feature DoD:
  - [ ] Boards keep file order (E1-F3-T1).
  - [ ] A broken file reports line and column, and each 5.2 validation case (unknown `default_board`, empty `[boards]`, bad board name, wrong type, relative path) is a config error (E1-F3-T2).
  - [ ] A missing config is written once with create-new semantics and reported (E1-F3-T3).
  - [ ] A bad binding warns and keeps its default; a swap of two keys is accepted; a user binding colliding with another action's default reverts only the user-set action (E1-F3-T5).
  - [ ] TitleInput ignores printable `confirm` and `cancel` bindings, and colliding template hotkeys are dropped (E1-F3-T6).

### E1-F3-T1 Config parsing with defaults

- Status: todo
- Depends on: E1-F1-T2, E1-F1-T6
- Covers: FR-02, FR-42; TECHSPEC 5.2, 6.2 (unknown board), 13
- Size: M
- Scope: `src/core/config.rs` with `Config` and `parse(text, path, home)` through `toml` with `preserve_order` and `toml::Spanned` for positions: `default_board` (missing: `work`, A-E1-21), `editor` (missing: empty), `[boards]` in file order with a leading `~/` expanded against `home`, `[keys]` values read as a string or a list of strings into `keys`. `Config::board(name)` matches names ignoring case and fails `Error::NotFound("unknown board 'work2' (work, home, personal)")` listing names in file order.
- Not in scope: validation errors and warnings (E1-F3-T2); first run (E1-F3-T3); interpreting `[keys]` (E1-F3-T5).
- Tests first:
  1. `fr42_parses_default_file`: given the 5.2 text (`tests/fixtures/config/default.toml`) and home `/h`, when `parse` runs, then `default_board` is `work`, `editor` is empty, `boards` is work, home, personal under `/h/.local/share/brain-swap/`, and `keys` holds `("up", ["k", "up"])`.
  2. `fr02_boards_keep_file_order`: given `[boards]` with `zeta`, `alpha`, `mid` in that order, when parsed, then `boards` lists them in that order.
  3. `fr02_default_board_defaults_to_work`: given a file with `[boards]` `work = "/b/w"` and no `default_board`, when parsed, then `default_board` is `work`.
  4. `ts5_2_tilde_path_expands_to_home`: given `work = "~/b"` and `home = "/abs/h"` with home `/h`, when parsed, then the paths are `/h/b` and `/abs/h`.
  5. `ts6_2_board_lookup_ignores_case`: given the default config, when `board("WORK")` is called, then it returns `work` and its path.
  6. `ts6_2_unknown_board_lists_names`: given the default config, when `board("work2")` is called, then it returns `Err(Error::NotFound("unknown board 'work2' (work, home, personal)"))` with exit code 3.
  7. `fr42_key_value_is_string_or_list`: given `open = "o"` and `quit = ["q", "esc"]` in `[keys]`, when parsed, then `keys` holds `("open", ["o"])` and `("quit", ["q", "esc"])`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `tests/fixtures/config/default.toml` is the 5.2 block byte for byte (diffed in review).

### E1-F3-T2 Config errors and warnings

- Status: todo
- Depends on: E1-F3-T1
- Covers: FR-45; TECHSPEC 5.2 (validation, unknown keys), 10 (config)
- Size: M
- Scope: Each case fails `Error::Config { path, line, col, msg }` with the 1-based line and column of the offending key or of the TOML error span (messages A-E1-22): TOML syntax error (the parser's message); `default_board 'x' is not in [boards]`; `[boards] is empty` (at the `[boards]` header, or 1:1 when the table is missing, A-E1-21); `board name 'Work' must match [a-z][a-z0-9_-]*`; `<key>: expected <type>` for a known key of the wrong type, including a `[keys]` value that is neither a string nor a list of strings; `board <name>: path must be absolute or start with ~/`. An unknown top-level key warns `config: unknown key '<key>'` and parsing continues.
- Not in scope: unknown or bad entries inside `[keys]` (warnings of E1-F3-T5); showing the error in the TUI or `context` (E4-F1, E2-F3).
- Tests first:
  1. `fr45_syntax_error_names_file_line_col`: given a file whose line 4 is `default_board "work"`, when parsed, then the error is `Config` with line 4, the column of the parser's span, and its `Display` starts `config <path>:4:`.
  2. `ts5_2_default_board_not_in_boards_is_config_error`: given `default_board = "x"` on line 1 and boards work and home, when parsed, then the error is `Config { line: 1, col: 1, msg: "default_board 'x' is not in [boards]", .. }`.
  3. `ts5_2_empty_boards_is_config_error`: given an empty `[boards]` table on line 3, when parsed, then the error is `Config` at line 3 with `[boards] is empty`.
  4. `ts5_2_missing_boards_is_config_error`: given a file with only `default_board = "work"`, when parsed, then the error is `Config` with `[boards] is empty`.
  5. `ts5_2_bad_board_name_is_config_error`: given `Work = "/b"` and, separately, `1st = "/b"` in `[boards]`, when parsed, then each fails `Config` at that key's line and column.
  6. `ts5_2_wrong_type_is_config_error`: given separately `editor = 3`, `default_board = true`, `up = 5` and `up = [1]`, when parsed, then each fails `Config` at that key with `expected` in the message.
  7. `ts5_2_relative_board_path_is_config_error`: given separately `work = "boards/work"` and `work = "~user/x"`, when parsed, then each fails `Config` with `board work: path must be absolute or start with ~/`.
  8. `ts5_2_unknown_top_level_key_warns`: given the default file plus `colour = "red"`, when parsed, then it succeeds and the warnings are `["config: unknown key 'colour'"]`.
  9. `fr45_config_errors_exit_1`: given each error of tests 1 to 7, when mapped, then `exit_code()` is 1 and `code()` is `config`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every message of this task is listed in the doc comment of `config::parse`.

### E1-F3-T3 First run

- Status: todo
- Depends on: E1-F3-T2
- Covers: FR-66; TECHSPEC 5.1, 5.2 (create-new), 10 (missing config)
- Size: S
- Scope: `config::default_text(env)` returns the 5.2 file with each board path `<data_dir>/<name>` written as `~/...` when it lies under `HOME`, else absolute (A-E1-23). `config::load(env) -> Result<Loaded>` reads `env.config_file()`; when it is missing, it creates the parent folder and writes `default_text` with `OpenOptions::create_new`; `AlreadyExists` (a simultaneous first run) falls back to reading the file; `created` is `Some(path)` only when this process wrote it. No board folder is created (FR-04). Callers print `created config <path>` (E2-F1 on stderr, E4-F1 in the TUI).
- Not in scope: printing the notice (E2-F1, E4-F1); the AS-9 end-to-end run (E2-F1, E6-F1).
- Tests first:
  1. `fr66_missing_config_is_written_and_reported`: given a temp env without a config file, when `load` runs, then the file exists, `created` is `Some(env.config_file())` and the config parses with default board `work`.
  2. `fr66_default_text_equals_5_2_under_default_xdg`: given an env whose XDG values are the defaults under its HOME, when `default_text` runs, then it equals `tests/fixtures/config/default.toml` byte for byte.
  3. `fr66_existing_config_is_not_rewritten`: given a custom config file, when `load` runs, then its bytes are unchanged and `created` is `None`.
  4. `fr66_create_new_never_clobbers`: given a config file created between the existence check and the write (the test calls the internal `write_default` after creating the file), when it runs, then it reads the existing file, leaves its bytes unchanged and reports `created: None`.
  5. `fr66_first_run_creates_no_board_folder`: given a temp env, when `load` writes the default config, then `<data_dir>/work` does not exist.
  6. `ts5_1_custom_data_home_written_as_absolute`: given `XDG_DATA_HOME=/x/d` outside HOME, when `default_text` runs, then it holds `work = "/x/d/brain-swap/work"`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `load` never writes when the file exists, whatever its content (test 3 also covers a broken file).

### E1-F3-T4 Key binding syntax

- Status: doing
- Depends on: E1-F1-T1
- Covers: FR-43; TECHSPEC 5.3
- Size: S
- Scope: `src/core/keys.rs`: `pub enum Key { Char(char), Enter, Esc, Tab, BackTab, Space, Backspace, Delete, Up, Down, Left, Right, Home, End, PageUp, PageDown, F(u8) }`, `pub struct Binding { pub key: Key, pub ctrl: bool, pub alt: bool, pub shift: bool }`, `pub struct KeyPress { pub key: Key, pub ctrl: bool, pub alt: bool, pub shift: bool }` (terminal-free, A-E1-24), `parse_binding(&str) -> Result<Binding, String>`, `Binding::matches(&KeyPress) -> bool` and `Display` in config syntax. Grammar of 5.3: modifiers `ctrl`, `alt`, `shift` joined by `+`; a printable character is case-sensitive; `shift+h` normalises to `H` without the shift flag; named keys and `f1` to `f12`. Matching ignores SHIFT for printable characters, compares ctrl and alt exactly, and compares shift for named keys. Problems (A-E1-25): `empty binding`, `unknown key '<name>'`, `unknown modifier '<name>'`.
- Not in scope: converting crossterm events into `KeyPress` (E4-F1); actions, modes and conflicts (E1-F3-T5).
- Tests first:
  1. `ts5_3_parses_single_character`: given `o`, when parsed, then the binding is `Char('o')` without modifiers.
  2. `ts5_3_character_is_case_sensitive`: given `H` and `h`, when parsed, then they are `Char('H')` and `Char('h')`.
  3. `ts5_3_shift_h_normalises_to_upper_h`: given `shift+h`, when parsed, then it equals `parse_binding("H")`.
  4. `ts5_3_parses_named_keys`: given each of `enter`, `esc`, `tab`, `backtab`, `space`, `backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, when parsed, then each gives its `Key` variant.
  5. `ts5_3_parses_function_keys_1_to_12`: given `f1` and `f12`, then `F(1)` and `F(12)`; given `f0` and `f13`, then `Err`.
  6. `ts5_3_parses_modifier_combinations`: given `ctrl+d`, `alt+x` and `ctrl+alt+left`, when parsed, then the flags and keys match.
  7. `ts5_3_rejects_malformed_bindings`: given the empty string, `ctrl+`, `foo` and `hyper+x`, when parsed, then they fail with `empty binding`, `empty binding`, `unknown key 'foo'` and `unknown modifier 'hyper'`.
  8. `ts5_3_matches_upper_h_with_and_without_shift`: given the binding `H`, when matched against `KeyPress { Char('H'), shift: true }` and `KeyPress { Char('H'), shift: false }`, then both match.
  9. `ts5_3_ctrl_and_alt_must_match_exactly`: given `ctrl+d`, then it matches ctrl+d only, not `d` nor ctrl+alt+d; given `d`, then it does not match ctrl+d.
  10. `ts5_3_display_round_trips`: given `ctrl+d`, `H`, `enter` and `f5`, when parsed and displayed, then each text is unchanged.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/core/keys.rs` imports nothing from `ratatui` or `crossterm`.

### E1-F3-T5 Key map, merge and conflicts

- Status: todo
- Depends on: E1-F3-T1, E1-F3-T4
- Covers: FR-43, FR-44; TECHSPEC 5.2 (actions and modes), 5.3 (conflicts)
- Size: M
- Scope: `pub enum Action` with the 17 actions of 5.2 and `config_name()` (`up`, `down`, `left`, `right`, `move_left`, `move_right`, `open`, `jump`, `new`, `edit`, `switch_board`, `scroll_down`, `scroll_up`, `help`, `quit`, `confirm`, `cancel`); `pub enum KeyContext { Board, Detail, Picker, TitleInput }` with the 5.2 sets (Board: all but `scroll_*`, `confirm`, `cancel`; Detail: `up`, `down`, `scroll_*`, `jump`, `edit`, `help`, `quit`; Picker: `up`, `down`, `confirm`, `cancel`; TitleInput: E1-F3-T6). `KeyMap::defaults()` is the 5.2 table. `KeyMap::build(user)`: an unknown action warns `key config: <name>: unknown action`; an entry with an unparseable string is dropped whole with `key config: <action>: <problem>` and the action keeps its defaults (A-E1-25); a user entry replaces that action's defaults; then, per context on the merged map, while a key is bound to two actions, the user entry of every user-set action involved is dropped (both when both are user-set), with `key config: <action>: conflicts with <other> on '<key>'`, the action returns to its defaults in every context, and the check repeats. `KeyMap::action(ctx, press)` and `KeyMap::keys(action)` (for Help, FR-32).
- Not in scope: TitleInput and template hotkey rules (E1-F3-T6); the status line and Help screen (E4-F1).
- Tests first:
  1. `ts5_3_defaults_are_conflict_free`: given `KeyMap::defaults()`, when every context's bindings are checked, then no binding maps to two actions.
  2. `fr43_user_entry_replaces_defaults`: given `open = "x"`, when built, then `keys(Open)` is `[x]` and `o` maps to nothing in Board.
  3. `fr44_bad_binding_warns_and_keeps_default`: given `open = "foo"`, when built, then `keys(Open)` is `[o]` and the warnings are `["key config: open: unknown key 'foo'"]`.
  4. `fr44_unknown_action_warns`: given `jumpp = "x"`, when built, then the map equals the defaults and the warnings are `["key config: jumpp: unknown action"]`.
  5. `fr44_swap_of_two_keys_is_accepted`: given `open = "n"` and `new = "o"`, when built, then there is no warning, `n` gives `Open` and `o` gives `New` in Board.
  6. `fr44_user_binding_colliding_with_default_reverts_only_user_action`: given `new = "j"`, when built, then `keys(New)` is `[n]`, `keys(Down)` is `[j, down]` and one warning names `new`.
  7. `fr44_both_user_set_actions_revert_on_conflict`: given `open = "x"` and `edit = "x"`, when built, then `keys(Open)` is `[o]`, `keys(Edit)` is `[e]` and two warnings name `open` and `edit`.
  8. `ts5_3_conflict_is_checked_per_context`: given `scroll_down = "o"`, when built, then it is kept (Detail has no `open`); given `scroll_down = "e"`, then it reverts (Detail has `edit`).
  9. `ts5_3_reverted_action_returns_to_defaults_in_every_context`: given `up = "o"` (colliding with `open` in Board only; Detail and Picker have no `open`), when built, then `keys(Up)` is `[k, up]`, one warning names `up`, and in Detail and in Picker `o` gives `None` while `k` gives `Up`.
  10. `ts5_3_conflict_check_repeats_until_clean`: given `new = "o"`, `open = "x"` and `edit = "x"`, when built, then the first round reverts `open` and `edit`, the second reverts `new`, the map equals the defaults and there are three warnings.
  11. `fr43_action_lookup_by_context`: given the defaults, when `j` is looked up in Detail, `enter` in Picker and `enter` in Board, then they give `Down`, `Confirm` and `Jump`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Test 1 iterates every `KeyContext`, so a later default that collides fails it.

### E1-F3-T6 Mode-specific key checks

- Status: todo
- Depends on: E1-F3-T5
- Covers: FR-43, FR-44; TECHSPEC 5.2 (TitleInput, template hotkeys)
- Size: S
- Scope: In `KeyContext::TitleInput` only `confirm` and `cancel` resolve; a `confirm` or `cancel` binding that is a printable character without ctrl or alt is ignored there with `key config: <action>: printable key '<k>' ignored in title input` (A-E1-25) and still applies in Picker. `KeyMap::check_template_hotkeys(&self, hotkeys)` drops a hotkey equal to a final Picker binding of `up`, `down`, `confirm` or `cancel` with `template <name>: hotkey '<k>' collides with <action>, dropped`; it takes plain `(name, char)` pairs so this feature needs no template types.
- Not in scope: duplicate hotkeys among templates (E1-F4-T3); text entry, `backspace` and the empty-title rule in TitleInput (E4-F4).
- Tests first:
  1. `ts5_2_title_input_ignores_printable_confirm`: given `confirm = ["enter", "y"]`, when built, then in TitleInput `y` gives `None` and `enter` gives `Confirm`, in Picker `y` gives `Confirm`, and one warning names `confirm`.
  2. `ts5_2_title_input_resolves_only_confirm_and_cancel`: given the defaults, when `j`, `q` and `?` are looked up in TitleInput, then each gives `None`, and `enter` and `esc` give `Confirm` and `Cancel`.
  3. `ts5_2_template_hotkey_colliding_with_down_is_dropped`: given the defaults and hotkeys `[("Feature", 'f'), ("Junk", 'j')]`, when checked, then `('Feature', 'f')` is kept, `j` is dropped and the warning is `template Junk: hotkey 'j' collides with down, dropped`.
  4. `ts5_2_template_hotkey_check_uses_final_keys`: given `down = "x"` and hotkeys `j` and `x`, when checked, then `j` is kept and `x` is dropped.
  5. `ts5_2_builtin_hotkeys_survive_defaults`: given the defaults and hotkeys `f`, `b`, `r`, `c`, when checked, then all four are kept without warnings.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The doc comment of `check_template_hotkeys` states it must run after `KeyMap::build`.

## E1-F4 Templates

- Depends on: E1-F1
- Covers: FR-11, FR-12, FR-13
- Spec: TECHSPEC 2.1 (`templates/`), 5.4, 6.2 (unknown template)
- Scope: The four built-in templates as embedded markdown files with frontmatter `name` and `key` and a skeleton whose `##` headings are the fields. Loading the user template folder so a file adds a template or replaces a built-in of the same name, with the 5.4 rules for missing frontmatter, missing `key`, unreadable files and an unreadable folder. Dropping a `## Timeline` heading and duplicate hotkeys with warnings, and lookup by name with the 6.2 error. The task-level dependency on the E1-F2 frontmatter reader is recorded as A-E1-02.
- Provides: `templates/feature.md`, `templates/bug.md`, `templates/research.md`, `templates/chore.md`; `core::template::Template { name: String, key: Option<char>, skeleton: String }`, `Template::fields(&self) -> Vec<String>`; `core::template::parse_template(file_stem: &str, text: &str) -> Result<Template, String>`; `core::template::builtins() -> Vec<Template>`; `core::template::load(dir: &Path) -> (Vec<Template>, Vec<String>)`; `core::template::find<'a>(&'a [Template], name: &str) -> Result<&'a Template>`
- Requires: E1-F1 (`Error`), E1-F2-T1 (`frontmatter::read`)
- Feature DoD:
  - [ ] Built-ins Feature, Bug, Research and Chore have the 5.4 keys and fields (E1-F4-T1).
  - [ ] User files override and add; a file without frontmatter, one without `key` and an unreadable one load as 5.4 says (E1-F4-T2).
  - [ ] A `## Timeline` heading and duplicate hotkeys are dropped with warnings, and an unknown name lists the valid ones (E1-F4-T3).

### E1-F4-T1 Built-in templates

- Status: todo
- Depends on: E1-F1-T2, E1-F2-T1
- Covers: FR-11, FR-13; TECHSPEC 5.4 (built-ins, template file shape)
- Size: S
- Scope: Write `templates/feature.md` (`name: Feature`, `key: f`; `## Goal`, `## Acceptance`, `## Context`), `templates/bug.md` (`Bug`, `b`; `## Symptom`, `## Expected`, `## Repro`, `## Context`), `templates/research.md` (`Research`, `r`; `## Question`, `## Done when`, `## Context`) and `templates/chore.md` (`Chore`, `c`; `## Task`, `## Why`, `## Context`), each heading followed by a blank line. `src/core/template.rs`: `Template`, `Template::fields()` (the `## ` headings outside code fences), `parse_template(file_stem, text)` (name from frontmatter `name`, else the stem with its first letter upper-cased; `key` one character; the skeleton is the text after the frontmatter, or all of it without one; invalid frontmatter is `Err(problem)`) and `builtins()` embedding the four files with `include_str!` in the order Feature, Bug, Research, Chore.
- Not in scope: the user folder (E1-F4-T2); `## Timeline` and duplicate hotkey rules (E1-F4-T3); listing templates in the CLI (E2-F1).
- Tests first:
  1. `fr11_builtins_are_feature_bug_research_chore`: given `builtins()`, when names are listed, then they are `Feature`, `Bug`, `Research`, `Chore` in that order.
  2. `fr11_builtin_hotkeys_are_f_b_r_c`: given `builtins()`, when keys are listed, then they are `Some('f')`, `Some('b')`, `Some('r')`, `Some('c')`.
  3. `fr11_builtin_fields_follow_5_4`: given each built-in, when `fields()` runs, then Feature gives Goal, Acceptance, Context; Bug Symptom, Expected, Repro, Context; Research Question, Done when, Context; Chore Task, Why, Context.
  4. `fr13_skeleton_is_text_after_frontmatter`: given the Feature built-in, when its skeleton is read, then it starts with `## Goal` and contains neither `---` nor `name:`.
  5. `fr11_fields_skip_fenced_headings`: given `parse_template("x", "## A\n```\n## B\n```\n")`, when `fields()` runs, then it returns `["A"]`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The four template files pass the docs test and hold no `## Timeline`.

### E1-F4-T2 User template folder

- Status: todo
- Depends on: E1-F4-T1
- Covers: FR-12; TECHSPEC 5.4 (user files, unreadable cases)
- Size: M
- Scope: `template::load(dir) -> (Vec<Template>, Vec<String>)`: the built-ins, then every `*.md` file directly in `dir` in file name order; a template whose name matches a built-in ignoring case replaces it in its place, a new name is appended (A-E1-26); a file without `key` has no hotkey; a file without frontmatter is all skeleton; a non-UTF-8 file, an unreadable file or one with invalid frontmatter is skipped with `template <file>: <problem>`; an unreadable folder warns `templates <dir>: <os error>` and gives the built-ins; a missing folder gives the built-ins without a warning (A-E1-26).
- Not in scope: `## Timeline` and duplicate hotkeys (E1-F4-T3); the template folder path (`Env::templates_dir`, E1-F1-T6).
- Tests first:
  1. `fr12_user_file_adds_template`: given `spike.md` with `name: Spike`, `key: s` and `## Hypothesis`, when `load` runs, then there are five templates and the last is `Spike` with key `s` and field `Hypothesis`.
  2. `fr12_user_file_replaces_builtin_ignoring_case`: given `my-feature.md` with `name: feature` and `## Why`, when `load` runs, then index 0 is named `feature`, has fields `["Why"]` and no hotkey, and there are four templates.
  3. `fr12_name_defaults_to_capitalised_stem`: given `spike.md` with frontmatter `key: s` only, when loaded, then its name is `Spike`.
  4. `ts5_4_file_without_frontmatter_is_all_skeleton`: given `spike.md` holding `## Hypothesis\n`, when loaded, then its skeleton is `## Hypothesis\n` and its key is `None`.
  5. `ts5_4_file_without_key_has_no_hotkey`: given `spike.md` with `name: Spike` only, when loaded, then its key is `None`.
  6. `ts5_4_non_utf8_file_is_skipped_with_warning`: given `bad.md` holding a 0xFF byte, when loaded, then no template comes from it and one warning starts `template bad.md: `.
  7. `ts5_4_invalid_frontmatter_file_is_skipped_with_warning`: given `bad.md` starting `---\nname Spike\n---\n`, when loaded, then it is skipped with one warning.
  8. `ts5_4_unreadable_file_is_skipped_with_warning`: given `spike.md` with mode 000 (test skipped when running as root), when loaded, then it is skipped with one warning.
  9. `ts5_4_unreadable_folder_only_warns`: given the folder with mode 000, when loaded, then the four built-ins come back with one warning starting `templates `.
  10. `ts5_4_missing_folder_gives_builtins_silently`: given a path that does not exist, when loaded, then the four built-ins come back and the warnings are empty.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `load` never returns an error type: every problem is a warning (5.4: `templates` exits 0).

### E1-F4-T3 Template hygiene and lookup

- Status: todo
- Depends on: E1-F4-T2
- Covers: FR-11; TECHSPEC 5.4 (`## Timeline`, duplicate hotkeys), 6.2 (unknown template)
- Size: S
- Scope: In `load`: a `## Timeline` line in a skeleton is removed with `template <name>: ## Timeline dropped`; for duplicate hotkeys the first template in load order keeps the key and every later one loses it with `template <name>: hotkey '<k>' already used by <other>, dropped`; a `key` that is not exactly one character gives no hotkey with `template <name>: key must be one character` (A-E1-27). `template::find(templates, name)` matches ignoring case and fails `Error::NotFound("unknown template 'epic' (Feature, Bug, Research, Chore)")` listing the names in load order.
- Not in scope: hotkeys colliding with picker keys (E1-F3-T6); the TemplatePick screen (E4-F4).
- Tests first:
  1. `ts5_4_timeline_heading_in_template_is_dropped`: given `spike.md` with `## Hypothesis` and `## Timeline`, when loaded, then its fields are `["Hypothesis"]` and the warnings hold `template Spike: ## Timeline dropped`.
  2. `ts5_4_duplicate_hotkey_is_dropped_from_later_template`: given `spike.md` with `key: f`, when loaded, then Feature keeps `f`, Spike has `None` and the warning is `template Spike: hotkey 'f' already used by Feature, dropped`.
  3. `ts5_4_multi_character_key_is_ignored_with_warning`: given `spike.md` with `key: ff`, when loaded, then Spike has no hotkey and one warning names it.
  4. `ts6_2_find_template_ignores_case`: given the built-ins, when `find(.., "feature")` runs, then it returns the Feature template.
  5. `ts6_2_unknown_template_lists_names`: given the built-ins, when `find(.., "epic")` runs, then it fails `Error::NotFound("unknown template 'epic' (Feature, Bug, Research, Chore)")` with exit code 3.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] After `load`, no two templates share a hotkey (asserted in test 2 over the whole list).

## E1-F5 Board loading

- Depends on: E1-F2, E1-F3
- Covers: FR-01, FR-03, FR-04, FR-08, FR-09, FR-10
- Spec: TECHSPEC 3 (Board, BoardMeta, I1, I3, I5, `activity()`, `is_open()`, `jump_note()`), 4.1, 4.3, 4.5, 4.7, 4.9, 6.2 (unknown column), 10 (bad files, missing folder, duplicate letters)
- Scope: Reading `board.md` into letter, columns and next number with the 4.3 defaults and warnings; discovering card files in the board root and parsing them, with a missing folder reading as an empty board; placing cards in configured or extra unknown columns, deciding which are open and ordering each column by activity; looking up a card by ID across the configured boards by letter and a column by name. Acceptance tests load the 4.9 board and the AS-7 hand edits. Readers never create or lock anything.
- Provides: `core::model::BoardMeta { letter: char, columns: Vec<String>, next: u32 }`, `core::model::Board { name, path, meta, cards, warnings }`; `core::board_file::parse(board_name: &str, bytes: Option<&[u8]>) -> BoardFile` with `BoardFile { letter: char, columns: Vec<String>, next: Option<u32>, writable: bool, warnings: Vec<String> }`; `core::board::load(name: &str, path: &Path, env: &Env) -> Result<Board>`; `core::board::Column { name: String, unknown: bool }`; `Board::column_list(&self) -> Vec<Column>`; `Board::cards_in(&self, column_index: usize) -> Vec<&Card>`; `Board::is_open(&self, &Card) -> bool`; `Board::open_cards(&self) -> Vec<&Card>`; `Board::card(&self, CardId) -> Option<&Card>`; `Board::column(&self, name: &str) -> Result<&str>`; `Card::activity(&self) -> Option<&Stamp>`; `Card::latest_note(&self) -> Option<&Note>`; `Card::jump_note(&self) -> Option<usize>`; `core::board::board_letter(name: &str, path: &Path) -> (char, Vec<String>)`; `core::board::find_card(boards: &[(String, PathBuf)], id: CardId, env: &Env) -> Result<Board>`; fixture boards `tests/fixtures/boards/work_4_9/` and `tests/fixtures/boards/as7/`
- Requires: E1-F1 (`Env`, `Error`, `CardId`, stamps), E1-F2 (`frontmatter::read`, `card_file::parse`, fixtures), E1-F3 (`Config.boards` shape)
- Feature DoD:
  - [ ] `board.md` defaults, letter, columns and next follow 4.3 (E1-F5-T1).
  - [ ] A missing folder reads as empty, foreign and non-card files are ignored, bad files never stop loading (E1-F5-T2).
  - [ ] Unknown columns, open cards and activity order follow section 3 and 4.7 (E1-F5-T3).
  - [ ] Card IDs resolve case-insensitively by letter, duplicate letters warn (E1-F5-T4).
  - [ ] 4.9 and AS-7 load as described (E1-F5-T5).

### E1-F5-T1 board.md reader

- Status: todo
- Depends on: E1-F2-T1
- Covers: FR-03; TECHSPEC 3 (I3), 4.3
- Size: S
- Scope: `core::model::BoardMeta` and `src/core/board_file.rs` `parse(board_name, bytes) -> BoardFile`. Missing file or no frontmatter: letter = first character of the name upper-cased, columns `Todo`, `Doing`, `Done`, `next: None`. `letter`: one character `A` to `Z`, lower case upper-cased, any other value warns `board.md: invalid letter '<v>', using <L>` and keeps the derived one. `columns`: comma-separated, trimmed, empty items dropped; duplicates ignoring case are dropped with `board.md: duplicate column '<name>' dropped`, and an empty result falls back to the defaults with `board.md: no columns, using Todo, Doing, Done` (A-E1-29). `next`: a positive integer, else `board.md: invalid next '<v>'` and `None`. Invalid frontmatter: defaults, `writable: false`, warning `board.md:<line>: <problem>` (4.3). Text below the frontmatter is ignored.
- Not in scope: writing `next:` (E1-F6-T5); the floor from card numbers (E1-F5-T2).
- Tests first:
  1. `fr03_missing_board_md_gives_defaults`: given name `work` and no bytes, when parsed, then letter `W`, columns `Todo, Doing, Done`, `next` `None`, writable, no warning.
  2. `fr03_reads_letter_columns_next`: given the 4.9 `board.md`, when parsed, then letter `W`, columns `Todo, Doing, Done` and `next` `Some(13)`.
  3. `ts4_3_lower_case_letter_is_upper_cased`: given `letter: w`, when parsed, then letter `W` without a warning.
  4. `ts4_3_invalid_letter_warns_and_derives`: given separately `letter: 7` and `letter: WX` for board `work`, when parsed, then letter `W` and one warning each.
  5. `ts4_3_columns_are_a_trimmed_comma_list`: given `columns: Backlog ,Doing,  Review, Done`, when parsed, then columns are `Backlog`, `Doing`, `Review`, `Done`.
  6. `i3_duplicate_columns_dropped_with_warning`: given `columns: Todo, todo, Done`, when parsed, then columns are `Todo`, `Done` and one warning names `todo`.
  7. `i3_empty_columns_give_defaults`: given `columns:`, when parsed, then columns are the defaults and one warning is given.
  8. `ts4_3_board_md_without_frontmatter_gives_defaults`: given `# work\n`, when parsed, then the defaults, writable true and no warning.
  9. `ts4_3_invalid_frontmatter_warns_and_blocks_writes`: given `---\nletter W\n---\n`, when parsed, then the defaults, `writable` false and the warning `board.md:2: ` followed by the problem.
  10. `ts4_3_bad_next_warns`: given `next: x`, when parsed, then `next` is `None` with one warning.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every returned `columns` list is non-empty and unique ignoring case (asserted by a helper used in every test).

### E1-F5-T2 Card discovery and load

- Status: todo
- Depends on: E1-F5-T1, E1-F2-T5, E1-F1-T6
- Covers: FR-01, FR-04; TECHSPEC 4.1, 4.6 (next floor), 10 (bad files, missing folder)
- Size: M
- Scope: `core::model::Board` and `src/core/board.rs` `load(name, path, env) -> Result<Board>`. A missing folder gives an empty board with the `board.md` defaults and creates nothing (FR-04). A folder that cannot be listed fails `Error::Io("board <name>: <os error>")`. Only regular files in the root are considered: a name that `CardId::from_file_name` accepts with the board letter is parsed with `card_file::parse` (default column: the first column) and gets `mtime` from its metadata; one with another letter warns `<file>: letter <X> is not this board's <W>, ignored`; everything else (`README.md`, `.git/`, `.W-12.md.bs-tmp-4711`, subfolders, `w-12.md`) is ignored silently (A-E1-28); a card file that cannot be read warns `<file>: <os error>` and is skipped. `meta.next = max(board.md next, highest number + 1)`, 1 for an empty board. Parse warnings of `board.md` and every card go to `Board.warnings`. Create `tests/fixtures/boards/work_4_9/` with the 4.9 `board.md`, `W-9.md`, `W-11.md`, `W-12.md` and a `README.md`.
- Not in scope: column placement and order (E1-F5-T3); lookup (E1-F5-T4); reload polling (E4-F1).
- Tests first:
  1. `fr04_missing_folder_reads_as_empty_board`: given a path that does not exist, when `load` runs, then `cards` is empty, `meta.next` is 1 and the folder still does not exist.
  2. `fr01_loads_one_card_per_file`: given a copy of `work_4_9`, when loaded, then the card IDs are `W-9`, `W-11` and `W-12` and `README.md` is not a card.
  3. `ts4_1_other_letter_card_is_ignored_with_warning`: given the copy plus `H-3.md`, when loaded, then no card `H-3` exists and the warnings hold `H-3.md: letter H is not this board's W, ignored`.
  4. `ts4_1_subfolders_and_temp_files_are_ignored`: given the copy plus `sub/W-1.md` and `.W-12.md.bs-tmp-4711`, when loaded, then neither is a card and there is no new warning.
  5. `ts4_6_next_is_max_of_board_md_and_highest_plus_one`: given the copy with `next: 5`, then `meta.next` is 13; with `next: 20`, then 20.
  6. `ts10_bad_card_does_not_stop_loading`: given the copy plus a non-UTF-8 `W-5.md`, when loaded, then `W-5` and `W-12` are both cards, `W-5` is not writable and a warning names `W-5.md`.
  7. `ts10_unlistable_folder_is_io_error`: given the board folder with mode 000 (skipped as root), when loaded, then it fails `Error::Io` whose message starts `board work: `.
  8. `ts3_card_mtime_comes_from_file`: given the copy with `W-9.md` mtime set to `2026-10-06T17:10:00+03:00`, when loaded, then that card's `mtime` is that instant.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `load` opens no file for writing and creates no folder (asserted by test 1 and by a read-only copy in test 2).

### E1-F5-T3 Columns, open cards and order

- Status: todo
- Depends on: E1-F5-T2
- Covers: FR-08, FR-09; TECHSPEC 3 (I5, `activity()`, `is_open()`, `jump_note()`), 4.5, 4.7 (unknown columns, bad stamps, notes without place), 10 (clock skew)
- Size: M
- Scope: A card's column is matched to a configured column ignoring case and stored in the configured spelling; an unmatched column becomes an extra column `<name> (unknown)` after the configured ones, in order of first appearance by card number (A-E1-30), with one warning `<ID>: unknown column '<name>'` per card. `Board::column_list()`, `Board::cards_in(index)` ordered by `activity()` newest first (instants compared), ties to the higher number; `Board::is_open(card)` (not in the last configured column; extra columns are open, I5); `Board::open_cards()`; `Card::activity()` is the last note's `at`, else `created`, else `mtime` (a last note with a bad stamp falls through, A-E1-30); `Card::latest_note()` is the last note in file order; `Card::jump_note()` is the index of the last note with `place.cwd` or `place.herdr`.
- Not in scope: `H` and `L` on extra columns (E4-F2); rendering counts and headers (E4-F2); best guess tiers (E1-F8).
- Tests first:
  1. `fr08_card_in_last_column_is_not_open`: given the `work_4_9` copy, when `is_open` runs, then `W-9` (Done) is false and `W-12` (Doing) and `W-11` (Todo) are true.
  2. `fr08_custom_last_column_is_done`: given `columns: Backlog, Review, Shipped` and a card in `Shipped`, when `is_open` runs, then it is false.
  3. `i5_unknown_column_is_extra_and_open`: given a card with `column: Doign`, when loaded, then `column_list()` ends with `Column { name: "Doign (unknown)", unknown: true }`, the card is in it and open, and the warnings hold `W-13: unknown column 'Doign'`.
  4. `ts4_7_extra_columns_in_order_of_first_appearance`: given `W-3` in `Zeta` and `W-5` in `Alpha`, when loaded, then the extra columns are `Zeta (unknown)` then `Alpha (unknown)`.
  5. `ts4_7_column_matches_ignoring_case`: given `column: doing`, when loaded, then the card is in `Doing` and `card.column` is `Doing`.
  6. `fr09_cards_ordered_newest_activity_first`: given three Todo cards whose activities are 10:00 (note), 11:00 (note) and 10:30 (created only), when `cards_in(0)` runs, then the order is 11:00, 10:30, 10:00.
  7. `fr09_equal_activity_ties_to_higher_number`: given `W-7` and `W-12` with equal latest stamps, when ordered, then `W-12` comes first.
  8. `fr09_activity_falls_back_to_created_then_mtime`: given a card without notes or `created`, when `activity()` runs, then it returns its `mtime`.
  9. `fr09_activity_compares_instants_across_offsets`: given notes at `10:00+03:00` and `08:30+00:00`, when ordered, then the `08:30+00:00` card comes first.
  10. `fr09_activity_uses_last_note_not_newest_stamp`: given a card whose last note is stamped earlier than its previous note, when `activity()` runs, then it returns the last note's stamp.
  11. `ts3_jump_note_is_last_note_with_place`: given notes placed, placed, unplaced, when `jump_note()` runs, then it returns `Some(1)`.
  12. `ts4_7_bad_stamp_note_with_place_is_jump_target`: given a last note with `at: None` and a cwd, when `jump_note()` runs, then it returns that index.
  13. `ts3_jump_note_none_without_placed_notes`: given only unplaced notes, when `jump_note()` runs, then it returns `None`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Ordering never depends on file listing order (test 7 runs with files created in both orders).

### E1-F5-T4 Lookup by ID, board and column

- Status: todo
- Depends on: E1-F5-T3, E1-F3-T1
- Covers: FR-10; TECHSPEC 4.7 (no move targets an extra column), 6.2 (unknown column), 10 (duplicate letters)
- Size: S
- Scope: `Board::card(id)`; `Board::column(name)` matches configured columns ignoring case (never an extra column) and fails `Error::NotFound("unknown column 'Doign' on work (Todo, Doing, Done)")`; `board::board_letter(name, path)` reads only `board.md`; `board::find_card(boards, id, env)` takes the first board in file order whose letter equals `id.letter`, warns `boards <a> and <b> share letter <L>; using <a>` into the returned board's warnings for each later board with that letter, loads it, and fails `Error::NotFound("W-99 not found")` when no board has the letter or the card is missing (A-E1-31).
- Not in scope: the unknown board error (`Config::board`, E1-F3-T1); printing errors (E2-F1).
- Tests first:
  1. `fr10_lower_case_id_finds_card`: given boards work (the `work_4_9` copy) and home, when `find_card` runs with `CardId::parse("w-12")`, then it returns board `work` holding `W-12`.
  2. `fr10_id_resolves_to_board_by_letter`: given work and home with `H-3.md`, when `find_card` runs for `H-3`, then it returns board `home`.
  3. `fr10_letter_from_board_md_is_used`: given board `ops` whose `board.md` has `letter: X` and holds `X-1.md`, when `find_card` runs for `X-1`, then it returns `ops`.
  4. `ts10_duplicate_letter_first_board_wins_with_warning`: given boards `work` and `wiki` in that order, both letter `W`, when `find_card` runs for `W-12`, then it returns `work` and its warnings hold `boards work and wiki share letter W; using work`.
  5. `ts6_2_unknown_card_is_not_found`: given the boards of test 1, when `find_card` runs for `W-99` and for `Q-1`, then each fails `Error::NotFound` with `W-99 not found` and `Q-1 not found` and exit code 3.
  6. `ts6_2_column_lookup_ignores_case`: given the `work_4_9` board, when `column("doing")` runs, then it returns `Doing`.
  7. `ts6_2_unknown_column_lists_columns`: given the same board, when `column("Doign")` runs, then it fails `Error::NotFound("unknown column 'Doign' on work (Todo, Doing, Done)")`.
  8. `ts4_7_extra_column_is_not_a_move_target`: given a board with the extra column `Doign (unknown)`, when `column("Doign")` runs, then it fails `NotFound`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `find_card` loads only the board it returns (other boards are read for `board.md` only, asserted with an unlistable second board folder).

### E1-F5-T5 4.9 and AS-7 boards load as described

- Status: todo
- Depends on: E1-F5-T4
- Covers: FR-01, FR-03, FR-04, FR-08, FR-09; TECHSPEC 4.9; AS-7 (file side)
- Size: S
- Scope: Acceptance-level core tests on `tests/fixtures/boards/work_4_9/` and a new `tests/fixtures/boards/as7/`: the 4.9 board plus `W-20.md` holding only `# spike on caching` and `W-13.md` with `column: Doign`. These pin the loader on the worked example and on the AS-7 hand edits.
- Not in scope: the TUI's 2-second refresh keeping the selection (E4-F1); the next park only appending (E1-F6-T4); rendering at 80x24 (E4-F2).
- Tests first:
  1. `ts4_9_work_board_loads_as_described`: given `work_4_9`, when loaded, then Todo holds `W-11`, Doing `W-12`, Done `W-9`, `meta.next` is 13, there are no warnings, and `W-12`'s latest note has Next `open the MR, ask the DBA for the v13 deploy list`.
  2. `ts4_9_jump_note_of_w12_targets_pane_w4v_p9`: given `work_4_9`, when `jump_note()` of `W-12` is read, then it is note index 1 with pane `w4V:p9`, and `W-9`'s jump note has a cwd and no herdr place.
  3. `as7_hand_made_card_lands_in_first_column_without_notes`: given `as7`, when loaded, then `W-20` is in Todo with title `spike on caching` and no notes.
  4. `as7_next_card_skips_past_hand_made_card`: given `as7`, when loaded, then `meta.next` is 21.
  5. `as7_misspelt_column_shows_as_unknown_extra_column`: given `as7`, when loaded, then `W-13` is in `Doign (unknown)` and the warnings name it.
  6. `as7_hand_edit_shows_on_next_load`: given a copy of `as7` loaded once, when `W-12.md`'s title line is edited on disk and the board loaded again, then `W-12` has the new title and the same ID.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The `work_4_9` fixture files equal the 4.9 text byte for byte (diffed in review).

## E1-F6 Store

- Depends on: E1-F4, E1-F5
- Covers: FR-04, FR-05, FR-06, FR-13, FR-16, NFR-06, NFR-08
- Spec: TECHSPEC 4.3 (board.md writes), 4.6, 4.8 (R6, R7), 7.6 (merge), 10, 11 (lock waits, merges, verify failures), 12.1 (crash and concurrency, in-process layer)
- Scope: Every write to a board goes through `core::store`: the per-board lock in the state folder with its 2-second timeout, the verify-then-rename atomic write, and the four operations built on the E1-F2 splices (move, park, create with hard-link ID allocation, and the editor merge), each re-reading the file inside the lock. After each write the store cleans stale temp files and prunes old session files. The in-process crash tests (failpoints on `Env`) and concurrency tests (threads) of 12.1 close the feature. Store functions take a loaded `Board` only for its name, path and columns and always re-read the files on disk.
- Provides: `core::store::BoardLock`, `core::store::lock_path(env: &Env, board_dir: &Path) -> Result<PathBuf>`, `core::store::lock_board(env: &Env, board_dir: &Path) -> Result<BoardLock>`; `core::store::temp_path(target: &Path, pid: u32) -> PathBuf`; `core::store::write_atomic(env: &Env, target: &Path, bytes: &[u8], verify: &dyn Fn(&[u8]) -> bool, failpoint: Option<&str>) -> Result<()>`; `core::store::move_card(env: &Env, board: &Board, id: CardId, column: &str) -> Result<Moved>` with `Moved { card: Card, changed: bool }`; `core::store::park(env: &Env, board: &Board, id: CardId, note: &NewNote) -> Result<Parked>` with `Parked { card: Card, warnings: Vec<String> }`; `core::store::CreateRequest<'a> { title: &'a str, template: &'a Template, column: Option<&'a str>, body: Option<&'a str> }`, `core::store::create(env: &Env, board: &Board, req: &CreateRequest) -> Result<Created>` with `Created { card: Card, warnings: Vec<String> }`; `core::board_file::set_next(existing: Option<&[u8]>, n: u32) -> Result<Vec<u8>>`; `core::store::link_error(board: &str, e: &std::io::Error) -> Error`; `core::store::merge_edit(env: &Env, board: &Board, id: CardId, c0: &[u8], edited: &[u8]) -> Result<Merged>` with `Merged { written: bool, added_notes: usize }`; `core::store::cleanup(env: &Env, board_dir: &Path)`; `tests/crash_store.rs`, `tests/concurrency_store.rs`
- Requires: E1-F1 (`Env`, `Error`, `failpoint::check`, `log::event`, `encode_file_name`), E1-F2 (`card_file::*`, `frontmatter::set`), E1-F4 (`Template`), E1-F5 (`Board`, `board::load`, `board_file::parse`, `Board::column`), E1-F7-T4 (`session::prune`, A-E1-03)
- Feature DoD:
  - [ ] A lock wait over 100 ms is appended to the log file (E1-F6-T1).
  - [ ] Verify runs before every rename and a failure writes nothing (E1-F6-T2).
  - [ ] Move: only the `column:` line changes, re-read inside the lock (E1-F6-T3).
  - [ ] Park: only the timeline grows, the card never moves (E1-F6-T4).
  - [ ] Create: the number exceeds every number used before, the first write creates the folder, `board.md` follows 4.3 (E1-F6-T5).
  - [ ] Creation never overwrites and gives up cleanly after 100 collisions or on a filesystem without hard links (E1-F6-T6).
  - [ ] The editor merge keeps notes parked meanwhile and the file's column per 7.6 (E1-F6-T7).
  - [ ] Stale temp files and old session files are cleaned after writes (E1-F6-T8).
  - [ ] The in-process 12.1 crash tests pass (E1-F6-T9).
  - [ ] The in-process 12.1 concurrency tests pass (E1-F6-T10).

### E1-F6-T1 Board lock

- Status: todo
- Depends on: E1-F1-T6, E1-F1-T8
- Covers: FR-04, FR-05, NFR-08; TECHSPEC 10 (board lock), 11 (lock waits), T-07, T-17
- Size: M
- Scope: `src/core/store.rs`: `pub struct BoardLock` (holds the locked `File`; unlocks on drop), `lock_path(env, board_dir) -> Result<PathBuf>`, which canonicalizes `board_dir` (an error is `Error::Io` with the OS message), returns `<env.locks_dir()>/<encode_file_name(canonical)>.lock` and creates nothing, and `lock_board(env, board_dir) -> Result<BoardLock>`: `create_dir_all(board_dir)`, `lock_path`, open that file (creating the locks folder), then `File::try_lock` every 10 ms for up to 2 s, then `Error::Busy("board busy, try again")` (A-E1-32). When the lock was acquired after more than 100 ms, `log::event(env, "lock_wait", "<canonical path> <ms>")`. Readers never call it.
- Not in scope: holding the lock across an editor or herdr call (never done; E4-F5 and E5 call the store per transaction); the TUI status line text (E4-F1).
- Tests first:
  1. `fr05_lock_file_lives_in_state_folder`: given a board folder in a temp dir, when `lock_board` runs, then `<state>/brain-swap/locks/<encoded path>.lock` exists and the board folder holds no new file.
  2. `fr04_lock_creates_missing_board_folder`: given a board path that does not exist, when `lock_board` runs, then the folder exists.
  3. `ts10_second_locker_waits_until_release`: given thread A holding the lock for 150 ms, when thread B calls `lock_board`, then B gets `Ok` after at least 100 ms and before 2 s.
  4. `ts10_lock_wait_over_100_ms_is_logged`: given test 3's setup and `env.log` set for B, when B acquires the lock, then the log holds one line with kind `lock_wait` naming the canonical board path.
  5. `ts10_short_lock_wait_is_not_logged`: given an uncontended lock and `env.log` set, when `lock_board` runs, then the log file is absent or empty.
  6. `ts10_lock_times_out_busy_after_2_seconds`: given a lock held for the whole test by another `File` handle, when `lock_board` runs, then it fails `Error::Busy("board busy, try again")` with exit code 5 after between 2 and 2.5 seconds.
  7. `ts10_lock_is_released_on_drop`: given a `BoardLock` dropped, when `lock_board` runs again, then it succeeds within 100 ms.
  8. `ts10_lock_path_is_encoded_canonical_board_path`: given a board folder `<tmp>/b` and a symlink `<tmp>/link` to it, when `lock_path(env, <tmp>/link)` runs, then it returns `<state>/brain-swap/locks/<encode_file_name(canonical <tmp>/b)>.lock`, the locks folder is not created, and a following `lock_board(env, <tmp>/b)` creates exactly that file.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Test 6 is the only core store test that waits the full 2 seconds; every other lock test stays under 200 ms.

### E1-F6-T2 Atomic write with verify

- Status: todo
- Depends on: E1-F6-T1, E1-F1-T7
- Covers: NFR-08; TECHSPEC 4.8 R7, 10 (atomic writes), 11 (verify failures), T-20
- Size: S
- Scope: `temp_path(target, pid)` gives `.<file name>.bs-tmp-<pid>` beside the target. `write_atomic(env, target, bytes, verify, failpoint)`: `verify(bytes)` first; false fails `Error::Verify("<file name>: verify failed")`, logs `verify_failed <path>` and writes nothing; else it writes the temp file, `sync_all`, runs `failpoint::check` for the given name, renames over the target and fsyncs the folder. Any error removes the temp, except a failpoint error, which leaves it as a crash would.
- Not in scope: the postconditions each operation verifies (E1-F6-T3 to T5, T7); hard-linking new cards (E1-F6-T5).
- Tests first:
  1. `ts10_write_atomic_replaces_content`: given a file holding `a`, when `write_atomic` writes `b` with a verify that accepts, then the file holds `b`.
  2. `ts10_temp_file_is_dot_name_bs_tmp_pid`: given target `/b/W-12.md` and pid 4711, when `temp_path` runs, then it returns `/b/.W-12.md.bs-tmp-4711`.
  3. `r7_verify_failure_writes_nothing`: given a file holding `a` and a verify that rejects, when `write_atomic` writes `b`, then it fails `Error::Verify(_)` with code `verify_failed` and exit 1, the file still holds `a` and no temp file exists.
  4. `r7_verify_failure_is_logged`: given test 3's setup with `env.log` set, when it fails, then the log holds one `verify_failed` line naming the target.
  5. `ts10_no_temp_left_after_success`: given test 1's run, when the folder is listed, then it holds only the target.
  6. `ts12_1_failpoint_after_tmp_leaves_temp_and_target`: given feature `failpoints` and `env.failpoint` `park:after_tmp` with `abort: false`, when `write_atomic` runs with failpoint `Some("park:after_tmp")`, then it fails `Error::Io`, the target is unchanged and the temp file holds the new bytes.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `write_atomic` is the only function in `src/core/` that calls `rename` (grep).

### E1-F6-T3 Move

- Status: todo
- Depends on: E1-F6-T2, E1-F5-T4, E1-F2-T8
- Covers: NFR-06, NFR-08; TECHSPEC 4.8 R2, R7, 6.2 (move to the current column), 10 (fresh reads), 12.1 (R7 per fixture)
- Size: S
- Scope: `move_card(env, board, id, column) -> Result<Moved>`: resolve `column` with `Board::column` (fails `NotFound`); lock; read `<board.path>/<id>.md` from disk (missing: `NotFound("<ID> not found")`); `check_writable` (fails `Unreadable`); if the parsed column equals the target ignoring case, write nothing and return `changed: false`; else `card_file::set_column`, verify with a named function `verify_move(before, after, column) -> bool` that the new bytes parse to the target column and differ from the read bytes only as R2 allows, and `write_atomic`. Returns the re-parsed card.
- Not in scope: `H` and `L` from extra columns (E4-F2); the `move` subcommand and `moved W-12 to Doing` (E2-F2); cleanup (E1-F6-T8 adds the call).
- Tests first:
  1. `nfr06_move_changes_only_column_line`: given a copy of `work_4_9`, when `W-12` is moved to `Done`, then `W-12.md` differs from before in exactly the `column:` line and `changed` is true.
  2. `ts6_2_move_to_current_column_writes_nothing`: given `W-12` in `Doing`, when moved to `doing`, then `changed` is false and the file bytes and mtime are unchanged.
  3. `ts6_2_move_to_unknown_column_is_not_found`: given the copy, when `W-12` is moved to `Doign`, then it fails `Error::NotFound("unknown column 'Doign' on work (Todo, Doing, Done)")` and nothing is written.
  4. `ts10_move_reads_file_fresh_inside_lock`: given a `Board` loaded before a note was appended to `W-12.md` on disk, when `W-12` is moved with that stale `Board`, then the file holds the appended note and the new column.
  5. `ts4_7_move_of_read_only_card_is_unreadable`: given `W-12.md` replaced by the `conflict_markers.md` fixture, when moved, then it fails `Error::Unreadable(_)` and the bytes are unchanged.
  6. `nfr06_move_adds_frontmatter_to_card_without_one`: given `W-20.md` holding only `# spike on caching`, when moved to `Doing`, then the file is `---\ncolumn: Doing\n---\n# spike on caching\n`.
  7. `ts6_2_move_of_missing_card_is_not_found`: given the copy, when `W-99` is moved, then it fails `Error::NotFound("W-99 not found")`.
  8. `ts6_2_move_to_current_column_in_other_case_writes_nothing`: given `W-12.md` with `column: doing`, when moved to `Doing`, then `changed` is false and the bytes are unchanged.
  9. `r7_verify_move_accepts_every_writable_fixture` (round trip fixture): given each writable fixture under `tests/fixtures/cards/`, when `set_column(.., "Doing")` runs and `verify_move(before, after, "Doing")` checks the result, then every one is accepted.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `move_card` never reads `board.cards` bytes or titles (only `board.path` and `Board::column`), checked in review.

### E1-F6-T4 Park

- Status: todo
- Depends on: E1-F6-T2, E1-F5-T4, E1-F2-T8
- Covers: FR-16, NFR-06, NFR-08; TECHSPEC 4.7 (missing timeline), 4.8 R3, R7, 12.1 (`park:after_tmp`, R7 per fixture)
- Size: M
- Scope: `park(env, board, id, note) -> Result<Parked>`: all three parts empty fails `InvalidInput("empty note")` before locking; lock; read the card file (missing: `NotFound("<ID> not found")`); `card_file::append_note` (fails `Unreadable` for read-only files); verify (R7) with `verify_park(before, after, inserted)` that the new bytes parse with exactly one more note and that the bytes before `inserted.start` and after the block equal the read bytes; `write_atomic` with failpoint `park:after_tmp`; return the re-parsed card and the splice warnings.
- Not in scope: reading the note from stdin or prompts, capturing the place, the reference (E2-F3); `parked to <ID>: <title>` (E2-F3); cleanup (E1-F6-T8 adds the call).
- Tests first:
  1. `fr16_park_appends_only_to_timeline`: given a copy of `work_4_9`, when a note is parked on `W-12`, then the new file equals the old with one block inserted after the last note, and title, column and body are unchanged.
  2. `fr16_park_never_moves_card`: given `W-11` in Todo, when a note is parked on it, then `W-11` is still in Todo.
  3. `ts4_7_park_readds_missing_timeline`: given `W-11.md` with its `## Timeline` line removed, when a note is parked, then the file ends with `## Timeline` followed by the note block.
  4. `fr18_park_with_empty_note_writes_nothing`: given three empty parts, when `park` runs, then it fails `Error::InvalidInput("empty note")` and the card bytes are unchanged.
  5. `ts6_2_park_on_missing_card_is_not_found`: given the copy, when `park` runs for `W-99`, then it fails `Error::NotFound("W-99 not found")`.
  6. `ts4_7_park_on_read_only_card_is_unreadable`: given `W-12.md` replaced by the `invalid_frontmatter.md` fixture, when `park` runs, then it fails `Error::Unreadable(_)` and nothing is written.
  7. `nfr08_two_parks_on_one_card_append_both`: given two `NewNote`s with the same stamp, when parked one after the other on `W-12`, then `W-12` has two more notes in park order.
  8. `r7_verify_park_accepts_every_writable_fixture` (round trip fixture): given each writable fixture under `tests/fixtures/cards/`, when `append_note` runs and `verify_park(before, after, inserted)` checks the result, then every one is accepted.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The verify closure is a named function `verify_park(before, after, inserted)` with its own unit test that rejects a changed byte outside the block.

### E1-F6-T5 Create

- Status: todo
- Depends on: E1-F6-T2, E1-F5-T4, E1-F4-T3, E1-F2-T6
- Covers: FR-04, FR-06, FR-13, NFR-06; TECHSPEC 3 (I3, I4, new stamps in the local zone), 4.3 (board.md writes), 4.6 steps 1 to 3, 4.8 R4, R7, 12.1 (`create:*` failpoints)
- Size: M
- Scope: `create(env, board, req) -> Result<Created>`: an empty trimmed title fails `InvalidInput("no title")` (A-E1-33); the column is `req.column` resolved with `Board::column` (fails `NotFound`), else the first column (I3); lock (which creates a missing folder, FR-04); re-read `board.md` with `board_file::parse` (not writable: `Unreadable("board.md:<line>: <problem>")`) and list card files; `n = max(next, highest number + 1)`; `board_file::set_next(existing, n + 1)` (new file exactly `---\nnext: <n>\n---\n`; prepend those three lines to a `board.md` without frontmatter; splice the `next:` line otherwise) written with `write_atomic`; `failpoint::check("create:after_next")`; render with `render_new` (body `req.body` or the template skeleton, template name, `created` = `env.now`, which E1-F1-T6 keeps in the local zone, section 3), verify that it parses back to those values (R4, R7), write it to `temp_path`, `sync_all`, `failpoint::check("create:after_tmp")`, `std::fs::hard_link(temp, "<L>-<n>.md")`, `failpoint::check("create:after_link")`, remove the temp. Returns the parsed card and the render warnings.
- Not in scope: retry on a link collision and unsupported links (E1-F6-T6); the title from a `# ` body line and `--body -` (E2-F2); the session reference (E2-F3); cleanup (E1-F6-T8).
- Tests first:
  1. `fr06_create_takes_next_number`: given a copy of `work_4_9`, when a Feature card titled `new task` is created, then `W-13.md` exists and `board.md` holds `next: 14`.
  2. `fr13_create_uses_template_skeleton`: given no body, when a Feature card is created, then its body holds `## Goal`, `## Acceptance` and `## Context` and its template is `Feature`.
  3. `fr13_create_uses_supplied_body`: given body `## Notes\nx\n`, when created, then the body is that text and holds no `## Goal`.
  4. `i3_create_enters_first_column_unless_given`: given no column, then the card is in Todo; given `doing`, then in Doing; given `nope`, then it fails `NotFound` and no card file appears.
  5. `fr04_create_makes_missing_board_folder`: given a board path that does not exist, when a card is created, then the folder holds `W-1.md` and `board.md` equal to `---\nnext: 2\n---\n`.
  6. `nfr06_create_changes_only_next_line_of_board_md`: given the 4.9 `board.md`, when a card is created, then `board.md` differs only in its `next:` line and exactly one new card file exists.
  7. `ts4_3_create_prepends_frontmatter_to_plain_board_md`: given `board.md` holding `# work\n` and no cards, when a card is created, then `board.md` is `---\nnext: 2\n---\n# work\n`.
  8. `ts4_3_create_with_invalid_board_md_is_unreadable`: given `board.md` `---\nletter W\n---\n`, when `create` runs, then it fails `Error::Unreadable` whose message starts `board.md:2: `, with exit 1, and no card file appears.
  9. `i4_next_exceeds_every_number_after_create`: given `board.md` `next: 3` and `W-12.md` present, when a card is created, then it is `W-13` and `board.md` holds `next: 14`.
  10. `ts4_6_deleting_cards_never_lowers_next`: given `W-13` created and its file then deleted, when another card is created, then it is `W-14`.
  11. `ts6_3_create_with_empty_title_is_invalid_input`: given title `"  "`, when `create` runs, then it fails `Error::InvalidInput("no title")` and nothing is written.
  12. `r4_created_card_parses_to_request_values`: given title `migrate invoices to v13`, template Feature and column Doing, when created, then the returned card has that title, column `Doing`, template `Feature` and `created` equal to `env.now`.
  13. `ts3_created_is_written_in_local_zone`: given an env built with `Env::from_vars` from `HOME=<tmp>` and `BRAIN_SWAP_NOW=2026-10-08T11:52:00+03:00` with tz UTC, when a card is created, then its file holds the line `created: 2026-10-08T08:52:00+00:00`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `set_next` has unit tests for the three `board.md` shapes (absent, no frontmatter, frontmatter) in `board_file.rs`.

### E1-F6-T6 Create collisions and unsupported links

- Status: todo
- Depends on: E1-F6-T5
- Covers: FR-06; TECHSPEC 4.6 (collision retry, hard links), T-05, T-13
- Size: S
- Scope: In `create`, a `hard_link` failing with `AlreadyExists` (a file made by hand meanwhile, or by a writer that skips the lock) sets `n = n + 1` and repeats from step 2 (`board.md` gets `next: n + 1` again), at most 100 attempts in all, then fails `Error::Io("board <name>: no free card number after 100 tries")` (A-E1-33). `link_error(board, e)` maps any other link error: kind `Unsupported` or `PermissionDenied` to `Error::Io("board <name>: hard links not supported")`, anything else to `Error::Io("board <name>: <os error>")`. The temp file is removed on every failure path. Linking never overwrites.
- Not in scope: network filesystems (unsupported, T-13); offline collisions across machines (accepted, T-05).
- Tests first:
  1. `fr06_create_skips_name_that_appears_inside_lock`: given a copy of `work_4_9` plus a directory named `W-13.md` (not listed as a card, but `hard_link` to that name fails with `AlreadyExists`), when a card is created, then it is `W-14`, `board.md` holds `next: 15` and the directory is untouched.
  2. `fr06_create_never_overwrites_existing_file`: given test 1's setup, when the card is created, then no existing path changed content and no temp file is left.
  3. `ts4_6_create_gives_up_after_100_collisions`: given directories `W-13.md` to `W-112.md`, when `create` runs, then it fails `Error::Io("board work: no free card number after 100 tries")`, exit 1, and no temp file is left.
  4. `ts4_6_unsupported_link_maps_to_hard_links_not_supported`: given `io::Error::from(ErrorKind::Unsupported)` and `io::Error::from(ErrorKind::PermissionDenied)`, when `link_error("work", e)` runs, then each gives `Error::Io("board work: hard links not supported")`.
  5. `ts4_6_other_link_error_keeps_os_message`: given `io::Error::other("disk on fire")`, when `link_error("work", e)` runs, then it gives `Error::Io("board work: disk on fire")`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The retry limit is a named constant `MAX_LINK_ATTEMPTS = 100` in `store.rs`.

### E1-F6-T7 Editor merge

- Status: todo
- Depends on: E1-F6-T4
- Covers: NFR-08; TECHSPEC 4.7 (missing timeline), 4.8 R6, 7.6 (merge inside the lock), 11 (merges), 12.1 (`edit:after_tmp`)
- Size: M
- Scope: `merge_edit(env, board, id, c0, edited) -> Result<Merged>`: `edited == c0` writes nothing (`written: false`); otherwise lock and read the file `F` (missing: `NotFound("<ID> not found")`). If `F == c0`, the result is `edited`. Else the result is `edited` plus every note block of `F` (`card_file::note_blocks`) whose exact text is not among `c0`'s blocks, appended in `F`'s order with `card_file::append_raw` (a missing `## Timeline` is re-added); then, if `edited`'s column equals `c0`'s and `F`'s differs, the result's column line is set to `F`'s with `frontmatter::set`, else `edited`'s column stays. When blocks or a column must be merged into an `edited` that is read only, it fails `Unreadable` and writes nothing (A-E1-34). Verify: every added block is present in the result (A-E1-34). `write_atomic` with failpoint `edit:after_tmp`; `log::event(env, "merge", "<ID> +<n> notes")`.
- Not in scope: the edit copy in `$XDG_STATE_HOME/brain-swap/edit/`, launching the editor, the non-zero exit rule and deleting the copy (E4-F5); cleanup (E1-F6-T8).
- Tests first:
  1. `r6_unchanged_file_gets_users_bytes`: given `F == c0` (the 4.9 `W-12.md`) and `edited` with a new title, when merged, then the file equals `edited` and `written` is true.
  2. `r6_note_parked_meanwhile_survives_edit`: given `F` = `c0` plus one parked note and `edited` = `c0` with a new title, when merged, then the file is `edited` with that note block at the end of its timeline and `added_notes` is 1.
  3. `r6_two_notes_in_one_second_are_both_kept`: given `F` with two new blocks sharing a heading but differing in Doing, when merged, then both blocks are appended in `F`'s order.
  4. `r6_note_deleted_by_user_stays_deleted`: given `edited` without `c0`'s first note and `F` = `c0` plus one new note, when merged, then the result lacks the deleted note and holds the new one.
  5. `r6_file_column_kept_when_user_left_column`: given `c0` in Doing, `F` moved to Done and `edited` still Doing with a body change, when merged, then the result has `column: Done` and the body change.
  6. `r6_user_column_change_wins`: given `c0` Doing, `F` Done and `edited` Todo, when merged, then the result has `column: Todo`.
  7. `r6_missing_timeline_in_edit_is_readded_for_merged_notes`: given `edited` without `## Timeline` and `F` with one new note, when merged, then the result ends with `## Timeline` and that block.
  8. `r6_no_change_writes_nothing`: given `edited == c0` and `F` different, when merged, then `written` is false and the file still equals `F`.
  9. `ts11_merge_is_logged`: given test 2's setup and `env.log` set, when merged, then the log holds one line `... merge W-12 +1 notes`.
  10. `ts7_6_read_only_edit_with_notes_to_merge_writes_nothing`: given `edited` with a conflict marker and `F` with one new note, when merged, then it fails `Error::Unreadable(_)` and the file still equals `F`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `merge_edit` is the only store function that writes bytes it did not splice itself (doc comment says so, R6).

### E1-F6-T8 Cleanup after writes

- Status: todo
- Depends on: E1-F6-T3, E1-F6-T4, E1-F6-T5, E1-F6-T7, E1-F7-T4
- Covers: FR-05; TECHSPEC 4.6 (stale temp removal), 10 (cleanup), T-17
- Size: S
- Scope: `cleanup(env, board_dir)`: calls `session::prune(env)` (E1-F7-T4, A-E1-03) and calls `std::fs::remove_file` on every entry in `board_dir`, whatever its type, whose name starts with `.` and contains `.bs-tmp-` and whose mtime is more than 1 hour before `env.now` (A-E1-35); every error is ignored. `create`, `park`, `merge_edit` (when written) and `move_card` (when changed) call it after the lock is released.
- Not in scope: the pruning rules of session and pane files (E1-F7-T4); editor copies (E4-F5).
- Tests first:
  1. `ts10_stale_temp_older_than_an_hour_is_removed_after_park`: given `.W-5.md.bs-tmp-1` with mtime 61 min before `env.now` and `.W-6.md.bs-tmp-1` at 59 min, when a note is parked on `W-12`, then the first is gone and the second remains.
  2. `ts10_old_session_files_are_pruned_after_a_write`: given `sessions/a.toml` with mtime 31 days before `env.now` and `sessions/b.toml` at 29 days, when `W-12` is moved, then `a.toml` is gone and `b.toml` remains.
  3. `ts10_cleanup_never_touches_other_files`: given `README.md` and `notes.bs-tmp-1` (no leading dot) with mtimes a year old, when a card is created, then both remain.
  4. `ts10_no_op_move_does_not_clean`: given a stale temp file and a move to the card's current column, when `move_card` returns `changed: false`, then the temp file remains.
  5. `ts10_cleanup_failure_never_fails_the_write`: given `.W-5.md.bs-tmp-1`, a non-empty directory whose mtime is set to 2 hours before `env.now` with `File::set_modified`, so `remove_file` on it fails, when a note is parked on `W-12`, then `park` returns `Ok`, the note is written and the directory still exists.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Each of the four write functions calls `cleanup` only after its `BoardLock` is dropped (checked in review; the lock is never held during cleanup).

### E1-F6-T9 In-process crash tests

- Status: todo
- Depends on: E1-F6-T6, E1-F6-T7, E1-F6-T8
- Covers: NFR-08; TECHSPEC 3 (I1 to I4), 4.6 (crash safety), 12.1 (crash, in-process layer)
- Size: M
- Scope: `tests/crash_store.rs` with `#![cfg(feature = "failpoints")]`. For each of `create:after_next`, `create:after_tmp`, `create:after_link`, `park:after_tmp` and `edit:after_tmp`, on a fresh copy of `work_4_9`, set `env.failpoint = Some(Failpoint { name, abort: false })`, run the operation, expect `Error::Io`, then reload the board with an env without failpoint and run `assert_board_sound(board, used_numbers)`: every card file parses without new warnings, I1 (letters `W`, numbers at least 1), I2 (file names `<id>.md`), I3 (columns non-empty and unique) and I4 (`meta.next` exceeds every number), and a following create gets a number greater than every number seen so far, including a skipped one.
- Not in scope: process-level crash tests through `new`, `park` and the TUI editor (E2-F3, E4-F5).
- Tests first:
  1. `nfr08_crash_after_next_only_skips_a_number`: given `create:after_next`, when `create` fails, then `board.md` holds `next: 14`, no `W-13.md` exists, the board is sound and the next create gives `W-14`.
  2. `nfr08_crash_after_tmp_leaves_ignored_temp`: given `create:after_tmp`, when `create` fails, then `.W-13.md.bs-tmp-<pid>` exists, is not loaded as a card, the board is sound and the next create gives `W-14`.
  3. `nfr08_crash_after_link_leaves_card_and_temp`: given `create:after_link`, when `create` fails, then `W-13.md` parses, a temp file remains, the board is sound and the next create gives `W-14`.
  4. `nfr08_crash_during_park_keeps_old_card`: given `park:after_tmp`, when `park` on `W-12` fails, then `W-12.md` equals its bytes before and parses.
  5. `nfr08_crash_during_edit_merge_keeps_file`: given `edit:after_tmp`, `F` = the 4.9 `W-12.md` plus one parked note and `edited` = the 4.9 bytes with a new title, when `merge_edit` fails, then the card file equals `F` and parses.
  6. `nfr08_crash_temp_removed_after_an_hour`: given test 2's leftover `.W-13.md.bs-tmp-<pid>` with its mtime set by `File::set_modified` to test 2's `env.now`, and a second temp `.W-11.md.bs-tmp-1` whose mtime is set 2 minutes after that stamp, when a note is parked on `W-11` with `env.now` 61 minutes after test 2's `env.now`, then the first temp is gone and the second (59 minutes old) remains.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `assert_board_sound` lives in `tests/common/sound.rs` so E2-F3 and E4-F5 reuse it for the process-level layer.

### E1-F6-T10 In-process concurrency tests

- Status: todo
- Depends on: E1-F6-T3, E1-F6-T4, E1-F6-T5
- Covers: NFR-08; TECHSPEC 10 (fresh reads), 12.1 (concurrency, threads); AS-8 (store half)
- Size: M
- Scope: `tests/concurrency_store.rs`: threads that call `core::store` on one copy of `work_4_9` plus a card `W-7` in Doing, each thread with its own `Env` and a distinct `pid` so temp names never clash, started together through a `std::sync::Barrier`. Round one: 20 parks (10 on `W-12`, 10 on `W-7`) and 5 moves alternating `W-12` and `W-7` between Doing and Done. Round two: 10 creates.
- Not in scope: the same load as processes through `park`, `move` and `new` (E2-F3); the TUI editor case of AS-8 (E4-F5).
- Tests first:
  1. `nfr08_parallel_parks_and_moves_lose_nothing`: given round one, when every thread has joined, then all 25 calls returned `Ok`, both files parse, `W-12` and `W-7` hold 10 more notes each, and each card's column is one its moves targeted.
  2. `nfr08_parallel_creates_get_distinct_ids`: given round two after round one, when every thread has joined, then 10 calls returned `Ok` with 10 distinct IDs, `board.md`'s `next` exceeds the highest, and every card file parses.
  3. `as8_parks_on_two_cards_in_one_second_both_land`: given two threads with the same `env.now` parking on `W-12` and `W-7`, when both join, then each card has exactly one more note and no other byte of either file changed.
  4. `as8_two_parks_on_one_card_both_append`: given two threads parking on `W-12` at once, when both join, then `W-12` has two more notes and both blocks are intact.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The suite passed 20 consecutive local runs, stated in the commit message.

## E1-F7 Session store

- Depends on: E1-F1
- Covers: FR-05, FR-41, FR-57
- Spec: TECHSPEC 2.2 (state folder), 3 (`SessionRef`, I8), 5.1 (`sessions/`, `panes/`), 6.4 (steps 1, 2 and 4, warnings, reference store, pane hints), 10 (cleanup of `sessions/` and `panes/`), T-09, T-17, T-19
- Scope: The core half of session identity and the reference store of 6.4. Checking a session ID against I8 and resolving the steps that need no process: a valid `--session`, else a valid `BRAIN_SWAP_SESSION`, else none, with the warnings `session id not substituted` and `invalid session id` and never a failure; the CLI inserts step 3 (herdr's pane session) between these and none. Reading and writing `$XDG_STATE_HOME/brain-swap/sessions/<id>.toml` with `card`, `board`, `pane`, `cwd` and `set_at`, atomically, an unparsable file reading as none; the pane hint `panes/<pane>` (card and session) written with every reference that has a pane, which E1-F8 reads as `linked here`; and pruning both folders after 30 days, which the store's cleanup calls (E1-F6-T8). Nothing here runs a process or reads the environment; core unit tests in `tests/session_store.rs` use `core_env` with a temp HOME.
- Provides: `core::model::SessionRef { card: CardId, board: String, pane: Option<String>, cwd: Option<PathBuf>, set_at: Stamp }` (section 3); `core::session::IdCheck` (`Valid`, `Placeholder`, `Invalid`), `core::session::check_id(&str) -> IdCheck`; `core::session::WARN_NOT_SUBSTITUTED` (`session id not substituted`), `core::session::WARN_INVALID` (`invalid session id`); `core::session::Resolution { session: Option<String>, warnings: Vec<String> }`, `core::session::resolve(flag: Option<&str>, env: &Env) -> Resolution`; `core::session::reference_path(env: &Env, session: &str) -> Option<PathBuf>`, `core::session::write_reference(env: &Env, session: &str, r: &SessionRef) -> Result<()>`, `core::session::read_reference(env: &Env, session: &str) -> Option<SessionRef>`, `core::session::stamp_serde` (serde `with` module for `set_at`); `core::session::PaneHint { card: CardId, session: String }`, `core::session::valid_pane_id(&str) -> bool`, `core::session::hint_path(env: &Env, pane: &str) -> Option<PathBuf>`, `core::session::read_pane_hint(env: &Env, pane: &str) -> Option<PaneHint>`; `core::session::PRUNE_AFTER` (30 days), `core::session::prune(env: &Env)`; `tests/session_store.rs`
- Requires: E1-F1 (`Env` with `session`, `pid`, `now`, `sessions_dir()`, `panes_dir()`; `encode_file_name`; `Error`; `CardId`; `Stamp`, `parse_stamp`, `format_stamp`; `tests/common/env.rs` `core_env`)
- Feature DoD:
  - [ ] Empty, placeholder and invalid IDs warn and fall through (E1-F7-T1).
  - [ ] A reference round-trips through `sessions/<id>.toml`, is written atomically, lives outside every board folder, and an unparsable file reads as none (E1-F7-T2).
  - [ ] A reference with a pane writes the pane hint, an ID outside the 6.4 pattern writes none, and the hint recovers the card for a new session that never parked (E1-F7-T3).
  - [ ] Files in `sessions/` and `panes/` older than 30 days are pruned and nothing else is touched (E1-F7-T4).

### E1-F7-T1 Session ID check and resolution steps 1, 2 and 4

- Status: todo
- Depends on: E1-F1-T6
- Covers: FR-41; TECHSPEC 3 (I8), 6.4 (steps 1, 2 and 4, missing flag, warnings), T-09
- Size: S
- Scope: `src/core/session.rs`: `pub enum IdCheck { Valid, Placeholder, Invalid }` and `check_id(s)`: `Placeholder` when `s` is empty or contains `$`, `{` or `}`; else `Valid` when it matches `[A-Za-z0-9._-]{1,128}`; else `Invalid` (I8). Constants `WARN_NOT_SUBSTITUTED = "session id not substituted"` and `WARN_INVALID = "invalid session id"`. `resolve(flag, env) -> Resolution`: step 1 returns a `Valid` flag; a `Placeholder` flag warns `WARN_NOT_SUBSTITUTED`, an `Invalid` one `WARN_INVALID`, and both fall through; a missing flag (`None`) falls through without a warning; step 2 applies the same rule to `env.session` (`BRAIN_SWAP_SESSION`), an empty value counting as unset, and each warning text appears at most once (A-E1-04); otherwise `session: None` (step 4). A `None` session tells the caller that step 3 may still apply inside herdr (E2-F3-T4); `resolve` itself never fails, reads nothing but `env.session` and calls no herdr.
- Not in scope: step 3, herdr's pane session and its call only inside herdr (E2-F3-T4, E5-F1); printing the warnings (E2-F3-T4 on stderr, E2-F3-T10 as `context` lines); `link` failing without a session (E2-F3-T7).
- Tests first:
  1. `fr41_valid_flag_resolves_without_warning`: given flag `Some("55583127-a22a-49bc-a803-e777c377a595")` and `env.session` `None`, when `resolve` runs, then `session` is that ID and `warnings` is empty.
  2. `i8_valid_ids_match_pattern`: given `a`, `A.b_c-1` and an ID of 128 `x` characters, when `check_id` runs, then each is `Valid`.
  3. `i8_empty_dollar_or_brace_is_placeholder`: given the empty string, `${CLAUDE_SESSION_ID}`, `$X`, `{x` and `x}`, when `check_id` runs, then each is `Placeholder`.
  4. `i8_other_bad_ids_are_invalid`: given `a b`, `a/b`, `é` and an ID of 129 `x` characters, when `check_id` runs, then each is `Invalid`.
  5. `fr41_empty_flag_warns_and_falls_back_to_env`: given flag `Some("")` and `env.session` `Some("b")`, when `resolve` runs, then `session` is `b` and `warnings` is `["session id not substituted"]`.
  6. `fr41_placeholder_flag_without_env_resolves_none`: given flag `Some("${CLAUDE_SESSION_ID}")` and `env.session` `None`, when `resolve` runs, then `session` is `None` and `warnings` is `["session id not substituted"]`.
  7. `fr41_invalid_flag_resolves_none_with_warning`: given flag `Some("a b")` and `env.session` `None`, when `resolve` runs, then `session` is `None` and `warnings` is `["invalid session id"]`.
  8. `ts6_4_missing_flag_uses_env_without_warning`: given flag `None` and `env.session` `Some("b")`, when `resolve` runs, then `session` is `b` and `warnings` is empty.
  9. `ts6_4_valid_flag_wins_over_env`: given flag `Some("a")` and `env.session` `Some("b")`, when `resolve` runs, then `session` is `a`.
  10. `ts6_4_nothing_resolves_silently`: given flag `None` and `env.session` `None`, when `resolve` runs, then `session` is `None` and `warnings` is empty.
  11. `ts6_4_invalid_env_session_warns_and_falls_through`: given flag `None` and `env.session` `Some("${X}")`, then `None` with `["session id not substituted"]`; given `Some("a b")`, then `None` with `["invalid session id"]`; given `Some("")`, then `None` with no warning (A-E1-04).
  12. `ts6_4_same_warning_given_once`: given flag `Some("")` and `env.session` `Some("${X}")`, when `resolve` runs, then `session` is `None` and `warnings` is exactly `["session id not substituted"]`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The doc comment of `resolve` states that a `None` session means steps 1 and 2 gave no valid ID and that only the caller may try step 3; `session.rs` imports nothing from `adapters` (layering test).

### E1-F7-T2 Reference file

- Status: todo
- Depends on: E1-F7-T1, E1-F1-T3
- Covers: FR-05, FR-57; TECHSPEC 3 (`SessionRef`), 5.1 (`sessions/`), 6.4 (reference store, `set_at`, atomic writes, unparsable file)
- Size: M
- Scope: `core::model::SessionRef` exactly as section 3. In `session.rs`: `reference_path(env, session)` is `Some(<env.sessions_dir()>/<session>.toml)` for a session `check_id` calls `Valid`, else `None`. `write_reference(env, session, r)`: an ID that is not `Valid` fails `Error::InvalidInput("invalid session id")` and writes nothing (A-E1-36); else it creates `sessions/` and writes TOML through a private serde struct with keys `card` (`W-12`), `board`, `pane`, `cwd`, `set_at` in that order, `pane` and `cwd` left out when `None`, and `set_at` through the `with` module `stamp_serde` (written with `format_stamp`, read with `parse_stamp` in UTC, A-E1-38), so jiff needs no `serde` feature. The bytes go to `.<session>.toml.bs-tmp-<env.pid>` beside the target, `sync_all`, then `rename` (A-E1-37); an I/O failure is `Error::Io("<path>: <os error>")` and removes the temp. `read_reference(env, session)` returns `None` for an invalid ID, a missing file, a non-UTF-8 or unparsable file, a missing key, a card `CardId::parse` rejects or a bad `set_at` (A-E1-38), and creates nothing. A later write for the same session replaces its reference (FR-57). E1-F7-T3 extends `write_reference` with the pane hint, so the file checks below use `pane: None` unless they read only `sessions/`.
- Not in scope: the pane hint (E1-F7-T3); pruning (E1-F7-T4); I9, a reference resolving only when its board is configured and its card file exists (E2-F3-T5, E2-F3-T9); which commands set a reference (E2-F3-T5, E2-F3-T7).
- Tests first:
  1. `fr57_reference_round_trips`: given `SessionRef { card: W-12, board: "work", pane: Some("w4V:p9"), cwd: Some("/home/smilen/Work/ati.billing"), set_at: 2026-10-08T11:52:00+03:00 }`, when `write_reference(env, "s1", ..)` and then `read_reference(env, "s1")` run, then the result equals the input.
  2. `ts6_4_reference_file_format`: given test 1's write, when `<state>/brain-swap/sessions/s1.toml` is read, then its text is exactly the lines `card = "W-12"`, `board = "work"`, `pane = "w4V:p9"`, `cwd = "/home/smilen/Work/ati.billing"` and `set_at = "2026-10-08T11:52:00+03:00"` (insta snapshot).
  3. `ts6_4_absent_pane_and_cwd_are_left_out`: given `pane: None` and `cwd: None`, when written and read back, then the file holds only `card`, `board` and `set_at` lines and the read value has `None` for both.
  4. `fr57_new_session_id_starts_unlinked`: given a reference written for `s1`, when `read_reference(env, "s2")` runs, then it returns `None`.
  5. `fr57_setting_again_replaces_reference`: given `s1` written with W-12, when `s1` is written again with W-7 and read, then its card is W-7.
  6. `ts6_4_unparsable_file_reads_as_none`: given files in `sessions/` named `a.toml` holding `card = `, `b.toml` with `card = "W12"`, `c.toml` with `set_at = "yesterday"`, `d.toml` without `board` and `e.toml` holding a 0xFF byte, the other keys valid, when each is read by its session ID, then each returns `None`.
  7. `ts6_4_set_at_without_offset_reads_as_utc`: given `sessions/s1.toml` with valid keys and `set_at = "2026-10-08 08:52:00"`, when read, then `format_stamp(&r.set_at)` is `2026-10-08T08:52:00+00:00` (A-E1-38).
  8. `i8_invalid_session_id_is_neither_written_nor_read`: given session `../x`, when `write_reference` runs, then it fails `Error::InvalidInput("invalid session id")` and the state folder holds no file; given a valid reference file `sessions/a b.toml` made by hand, when `read_reference(env, "a b")` runs, then it returns `None`.
  9. `fr05_reference_lives_in_state_folder`: given HOME, the cwd and a board folder under one temp dir and `pane: None`, when `write_reference(env, "s1", ..)` runs, then the only new file under the temp dir is `<state>/brain-swap/sessions/s1.toml`: no temp file is left and nothing appears in the cwd or the board folder.
  10. `ts6_4_reference_write_is_atomic`: given one thread rewriting `s1` 200 times, alternating W-12 and W-7 with `pane: None`, and a second thread reading `s1` 200 times after the first write, when both join, then every read returned `Some` with card W-12 or W-7.
  11. `ts6_4_read_creates_nothing`: given a temp HOME without a state folder, when `read_reference(env, "s1")` runs, then it returns `None` and `<state>/brain-swap` does not exist.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `read_reference` contains no `unwrap` or `expect`, and `Cargo.toml` still declares `jiff` without the `serde` feature (TECHSPEC 13).

### E1-F7-T3 Pane hints

- Status: todo
- Depends on: E1-F7-T2
- Covers: FR-05, FR-57; TECHSPEC 5.1 (`panes/`), 6.4 (pane hint, file name encoding, pane ID pattern), 10 (name encoding), T-19
- Size: S
- Scope: `PaneHint { card, session }`; `valid_pane_id(p)` is true when `p` matches `[A-Za-z0-9:._-]{1,128}` and is neither `.` nor `..` (A-E1-40); `hint_path(env, pane)` is `Some(<env.panes_dir()>/<encode_file_name(pane)>)` for a valid pane ID, else `None`. `write_reference` now writes, after the reference file and when `r.pane` is `Some` with a valid ID, the hint TOML with the lines `card = "<ID>"` and `session = "<session>"` through the same temp-and-rename write (A-E1-39); a pane ID that is not valid writes the reference and no hint. `read_pane_hint(env, pane)` returns `None` for an invalid pane ID, a missing file or anything unparsable, and creates nothing. The last reference set in a pane wins its hint, and replacing a session's reference elsewhere leaves the old pane's hint alone; this is how the best guess recovers the card after `/clear` even when the old session never parked (6.4).
- Not in scope: reading the current pane's hint and ranking it `linked here` (E1-F8-T3); capturing the pane (E5-F1); pruning hints (E1-F7-T4).
- Tests first:
  1. `ts6_4_reference_with_pane_writes_hint`: given `write_reference(env, "s1", r)` with card W-12 and pane `w4V:p9`, when `read_pane_hint(env, "w4V:p9")` runs, then it returns `PaneHint { card: W-12, session: "s1" }` and the hint file is `<state>/brain-swap/panes/w4V:p9`.
  2. `ts6_4_pane_hint_file_format`: given test 1's write, when the hint file is read, then its text is exactly the lines `card = "W-12"` and `session = "s1"` (insta snapshot).
  3. `ts6_4_reference_without_pane_writes_no_hint`: given a reference with `pane: None`, when written, then `panes/` does not exist or holds no file.
  4. `ts6_4_last_reference_in_pane_wins`: given `s1` written with W-12 in pane `w4V:p9`, when `s2` is written with W-7 in the same pane, then the hint names W-7 and `s2`, and `read_reference(env, "s1")` still names W-12.
  5. `ts6_4_pane_id_outside_pattern_writes_no_hint`: given pane IDs `w4V p9`, `a/b`, `%` and an ID of 129 `x` characters, when a reference with each is written, then each reference file exists, `panes/` holds no file and `read_pane_hint` returns `None` for each.
  6. `ts6_4_dot_pane_ids_write_no_hint`: given pane IDs `.` and `..`, when references with them are written, then `valid_pane_id` is false for both, `write_reference` returns `Ok` and `panes/` holds no file (A-E1-40).
  7. `ts6_4_unparsable_hint_reads_as_none`: given `panes/w4V:p9` holding `card = 3`, and separately `card = "W12"` with `session = "s1"`, when `read_pane_hint(env, "w4V:p9")` runs, then it returns `None` both times.
  8. `fr57_hint_outlives_session_change_without_park`: given only `write_reference(env, "old", ..)` with W-11 in pane `w4V:p11` and no card file or note anywhere, when `read_reference(env, "new")` and `read_pane_hint(env, "w4V:p11")` run, then the first returns `None` and the second names W-11.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `hint_path` builds the file name with `encode_file_name` (review), and no temp file remains in `panes/` after tests 1 to 6 (asserted).

### E1-F7-T4 Pruning

- Status: todo
- Depends on: E1-F1-T6
- Covers: FR-05; TECHSPEC 10 (cleanup of `sessions/` and `panes/`), T-17
- Size: S
- Scope: `pub const PRUNE_AFTER: jiff::SignedDuration` of 30 days (720 hours, T-17) and `prune(env)`: removes every regular file in `env.sessions_dir()` and `env.panes_dir()` whose mtime lies more than `PRUNE_AFTER` before `env.now` (A-E1-41); missing folders, unreadable entries and failed removals are ignored, and nothing else in the state folder is touched (`locks/`, `edit/`). It needs only the two folder paths, so it does not wait for E1-F7-T1 to E1-F7-T3 and can start right after E1-F1-T6; its tests create the files directly and set their mtime with `File::set_modified`. E1-F6-T8 calls it after every store write (A-E1-03).
- Not in scope: calling it after store writes, and the board folder's temp files (E1-F6-T8); editor copies (E4-F5).
- Tests first:
  1. `ts10_prune_removes_session_files_older_than_30_days`: given `sessions/a.toml` with mtime 31 days before `env.now` and `sessions/b.toml` at 29 days, when `prune` runs, then `a.toml` is gone and `b.toml` remains.
  2. `ts10_prune_removes_pane_hints_older_than_30_days`: given `panes/w4V:p9` at 31 days and `panes/w4V:p11` at 29 days, when `prune` runs, then the first is gone and the second remains.
  3. `ts10_prune_boundary_is_strictly_over_30_days`: given `sessions/x.toml` exactly 720 hours before `env.now` and `sessions/y.toml` 720 hours and 1 second before, when `prune` runs, then `x.toml` remains and `y.toml` is gone.
  4. `ts10_prune_measures_age_from_env_now`: given `sessions/a.toml` with mtime `2026-09-01T12:00:00+00:00`, when `prune` runs with `env.now` `2026-09-20T12:00:00+00:00`, then it remains, and with `env.now` `2026-10-08T12:00:00+00:00`, then it is gone.
  5. `ts10_prune_removes_old_crash_leftovers`: given `sessions/.s1.toml.bs-tmp-4711` at 31 days, when `prune` runs, then it is gone (A-E1-41).
  6. `ts10_prune_touches_nothing_else`: given `locks/x.lock` and `edit/work-W-12.md` with mtimes a year before `env.now`, when `prune` runs, then both remain.
  7. `ts10_prune_without_folders_creates_nothing`: given a temp HOME without a state folder, when `prune` runs, then it returns and `<state>/brain-swap` does not exist.
  8. `ts10_prune_ignores_errors`: given `sessions/` with mode 555 holding a 31-day-old `a.toml` and `panes/` holding a 31-day-old `w4V:p9` (test skipped as root), when `prune` runs, then it returns without panicking, `a.toml` remains and `panes/w4V:p9` is gone.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `PRUNE_AFTER` is the only place the 30 days appear in `src/`, and `prune` returns `()` with no `unwrap` or `expect`.

## E1-F8 Best guess

- Depends on: E1-F5, E1-F7
- Covers: FR-38
- Spec: TECHSPEC 3 (`activity()`, `is_open()`, I5), 6.4 (pane hint read as `linked here`), 6.6, 6.7 (`reason` values), 9.1 (`HERDR_ENV` detection)
- Scope: The 6.6 order of a board's open cards as one pure function. Each open card takes its best tier: 0 `linked here` (the current pane's hint names it), 1 `same pane` (its latest note with a pane is from the current pane), 2 `same folder` (its latest note with a cwd has the current cwd, lies inside it, or contains it when that cwd is neither `/` nor `$HOME`), 3 a middle column, 4 the rest; within a tier newest activity first, then the higher number. Each candidate carries its tier, whose JSON `reason` and text label 6.6 fixes. A small reader builds the input from `Env`: the current pane only when `HERDR_ENV=1`, the cwd, `$HOME` and the pane's hint read through E1-F7. Core unit tests in `tests/guess.rs` build boards in memory; the CLI prints the result (E2-F2-T8, E2-F3-T9).
- Provides: `core::guess::Tier` (`LinkedHere`, `SamePane`, `SameFolder`, `MiddleColumn`, `Other`, ordered by tier number) with `Tier::reason(&self) -> &'static str` and `Tier::label(&self) -> Option<&'static str>`; `core::guess::Candidate<'a> { card: &'a Card, tier: Tier }`; `core::guess::GuessInput { pane: Option<String>, cwd: PathBuf, home: PathBuf, hint: Option<CardId> }` with `GuessInput::from_env(env: &Env) -> GuessInput`; `core::guess::best_guess<'a>(board: &'a Board, input: &GuessInput) -> Vec<Candidate<'a>>`; `tests/common/mem_board.rs` (`mem_board`, `mem_card`, `mem_note`); `tests/guess.rs`
- Requires: E1-F5 (`Board`, `BoardMeta`, `Board::open_cards`, `Card::activity`, `board::load`, fixture `tests/fixtures/boards/work_4_9/`), E1-F2 (`Card`, `Note`, through E1-F5), E1-F7 (`session::read_pane_hint`, `session::write_reference`, `session::read_reference`, `SessionRef`), E1-F1 (`Env`, `Env::herdr_active`, `CardId`, `Place`, `HerdrPlace`, `parse_stamp`, `core_env`)
- Feature DoD:
  - [ ] Tiers 3 and 4 each have a test, closed cards never appear, and both tie rules (newest activity, then higher number) have a test (E1-F8-T1).
  - [ ] Tiers 1 and 2 each have a test, including the `/` and `$HOME` rule, the best-tier rule and a tie inside tier 1 (E1-F8-T2).
  - [ ] Tier 0 has a test, including recovery by pane hint for a new session whose predecessor never parked, and the pane and hint count only inside herdr (E1-F8-T3).

### E1-F8-T1 Middle columns, the rest and the tie rules

- Status: todo
- Depends on: E1-F5-T3
- Covers: FR-38; TECHSPEC 3 (`activity()`, `is_open()`), 6.6 (tiers 3 and 4, order within a tier, `reason`, labels), 6.7 (`reason`)
- Size: S
- Scope: `src/core/guess.rs`: `Tier`, `Candidate`, `GuessInput` (a plain struct here; `from_env` arrives in E1-F8-T3) and `best_guess(board, input)`: it takes `board.open_cards()`, so closed cards never appear; a card is tier 3 when its column is one of `board.meta.columns` other than the first and the last, else tier 4, a card in an extra column included (A-E1-42); the result is sorted by tier, then `activity()` newest first comparing instants, then the higher number, a card without any activity sorting after those with one (A-E1-43). `Tier::reason()` gives `linked_here`, `same_pane`, `same_folder`, `middle_column` and `other`; `Tier::label()` gives `linked here`, `same pane`, `same folder`, and `None` for tiers 3 and 4. `best_guess` is pure: no `Env`, no I/O, no clock. Create `tests/common/mem_board.rs`: `mem_board(columns: &[&str], cards: Vec<Card>) -> Board` (board `work`, letter `W`), `mem_card(id: &str, column: &str, created: Option<&str>, notes: Vec<Note>) -> Card` and `mem_note(at: &str, pane: Option<&str>, cwd: Option<&str>) -> Note`; `at` is a full stamp, or a bare `HH:MM` meaning that time on 2026-10-08 at `+03:00`, as in the tests below.
- Not in scope: tiers 0 to 2 (E1-F8-T2, E1-F8-T3); picker lines, `rank`, labels as printed and JSON (E2-F2-T8, E2-F3-T9).
- Tests first:
  1. `fr38_middle_column_card_comes_before_the_rest`: given columns Todo, Doing, Done, `W-11` in Todo with a note at `2026-10-08T11:00:00+03:00` and `W-12` in Doing with a note at `2026-10-08T10:00:00+03:00`, and an input with no pane, no hint and cwd `/elsewhere`, when `best_guess` runs, then the candidates are W-12 (`MiddleColumn`) then W-11 (`Other`).
  2. `fr38_closed_cards_are_never_candidates`: given test 1's board plus `W-9` in Done with the newest note, when `best_guess` runs, then W-9 is absent and there are two candidates.
  3. `ts6_6_tier_ties_to_newest_activity`: given `W-3` and `W-4` in Doing with notes at 11:00 and 10:00, when `best_guess` runs, then W-3 comes first.
  4. `ts6_6_equal_activity_ties_to_higher_number`: given `W-7` and `W-12` in Doing with notes at `2026-10-08T10:00:00+03:00` and `2026-10-08T07:00:00+00:00` (the same instant), when `best_guess` runs, then W-12 comes first.
  5. `ts6_6_every_middle_column_is_tier_3`: given columns Backlog, Doing, Review, Done and one card each in Backlog, Doing and Review, when `best_guess` runs, then the Doing and Review cards are `MiddleColumn` and the Backlog card is `Other`.
  6. `ts6_6_two_column_board_has_no_middle`: given columns Todo, Done and two Todo cards, when `best_guess` runs, then both are `Other`.
  7. `ts6_6_extra_column_card_is_other`: given a card with column `Doign` on a Todo, Doing, Done board, when `best_guess` runs, then it is a candidate with tier `Other` (A-E1-42).
  8. `ts6_6_card_without_activity_sorts_last_in_tier`: given Todo cards `W-2` without notes, `created` or mtime and `W-1` with one note, when `best_guess` runs, then W-1 comes first (A-E1-43).
  9. `ts6_6_reason_and_label_per_tier`: given each `Tier`, when `reason()` and `label()` run, then they give `linked_here` and `linked here`, `same_pane` and `same pane`, `same_folder` and `same folder`, `middle_column` and `None`, `other` and `None`.
  10. `ts4_9_work_board_guess_order`: given `tests/fixtures/boards/work_4_9/` loaded with `board::load` and an input with cwd `/elsewhere`, no pane and no hint, when `best_guess` runs, then the candidates are W-12 (`MiddleColumn`) then W-11 (`Other`).
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `best_guess` takes no `Env` and performs no I/O (its signature, and review).

### E1-F8-T2 Same pane and same folder

- Status: todo
- Depends on: E1-F8-T1
- Covers: FR-38; TECHSPEC 6.6 (tiers 1 and 2, best tier, ties within a tier)
- Size: M
- Scope: In `best_guess`: tier 1 when `input.pane` is `Some` and the card's last note with `place.herdr` has that pane; tier 2 when the card's last note with `place.cwd` has a cwd `c` with `c == input.cwd`, or `c` inside `input.cwd`, or `input.cwd` inside `c` while `c` is neither `/` nor `input.home`. Paths are compared lexically and component-wise (`Path::starts_with`, no canonicalization), and the `/` and `$HOME` exclusion applies to the note's cwd only, as 6.6 reads (A-E1-44). Each card takes its best (lowest-numbered) tier, and the E1-F8-T1 order applies inside every tier.
- Not in scope: tier 0 and the hint (E1-F8-T3); where the current pane, cwd and home come from (E1-F8-T3).
- Tests first:
  1. `fr38_same_pane_card_comes_first`: given pane `w4V:p9`, `W-7` in Doing whose only note (11:00) has pane `w4V:p11` and `W-11` in Todo whose only note (09:00) has pane `w4V:p9`, when `best_guess` runs, then W-11 (`SamePane`) precedes W-7 (`MiddleColumn`).
  2. `ts6_6_same_pane_uses_latest_note_with_a_pane`: given pane `w4V:p9` and a Todo card with notes from `w4V:p9` then `w4V:p11`, then it is not `SamePane`; given notes from `w4V:p9` then one with only a cwd, then it is `SamePane`.
  3. `ts6_6_no_current_pane_means_no_same_pane`: given `input.pane` `None` and a Todo card whose note has pane `w4V:p9`, when `best_guess` runs, then its tier is `Other`.
  4. `fr38_same_folder_equal_cwd`: given cwd `/work/billing` and a Todo card whose note cwd is `/work/billing`, when `best_guess` runs, then it is `SameFolder`.
  5. `ts6_6_note_cwd_inside_current_cwd_is_same_folder`: given cwd `/work/billing` and a note cwd `/work/billing/src`, then `SameFolder`.
  6. `ts6_6_current_cwd_inside_note_cwd_is_same_folder`: given cwd `/work/billing/src/x` and a note cwd `/work/billing`, then `SameFolder`.
  7. `ts6_6_root_or_home_note_cwd_does_not_contain_current`: given home `/home/u`, cwd `/home/u/proj` and a note cwd `/home/u`, then `Other`; given cwd `/tmp/x` and a note cwd `/`, then `Other`.
  8. `ts6_6_inside_is_component_wise`: given cwd `/work/billing` and Todo cards with note cwds `/work/bill` and `/work/billing2`, then neither is `SameFolder`.
  9. `ts6_6_same_folder_uses_latest_note_with_a_cwd`: given cwd `/work/billing` and notes with cwd `/work/billing` then `/other`, then the card is not `SameFolder`; given notes with cwd `/work/billing` then one without a place, then it is `SameFolder`.
  10. `ts6_6_current_home_contains_note_cwds_below_it`: given home and cwd both `/home/u` and a note cwd `/home/u/proj`, then `SameFolder` (A-E1-44).
  11. `ts6_6_card_takes_its_best_tier`: given pane `w4V:p9`, cwd `/work/billing` and a Doing card whose note has both, then `SamePane`; given a Doing card whose note has only that cwd, then `SameFolder`.
  12. `ts6_6_tier_beats_activity`: given pane `w4V:p9`, cwd `/work/billing`, `W-1` in Todo with a note at 08:00 from pane `w4V:p9`, `W-2` in Todo with a note at 09:00 with cwd `/work/billing`, `W-3` in Doing and `W-4` in Todo with notes at 10:00 and 11:00 with cwd `/other`, when `best_guess` runs, then the order is W-1, W-2, W-3, W-4.
  13. `ts6_6_ties_within_same_pane_tier`: given two `SamePane` cards `W-5` and `W-8` with notes at the same instant, then W-8 comes first; with W-5's note one minute newer, W-5 comes first.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Each clause of tiers 1 and 2 in 6.6 has a test where it holds and one where it does not (tests 1 to 10).

### E1-F8-T3 Linked here and recovery by pane hint

- Status: todo
- Depends on: E1-F8-T2, E1-F7-T3
- Covers: FR-38; TECHSPEC 6.4 (the hint read as `linked here`), 6.6 (tier 0, current pane only when `HERDR_ENV=1`), 9.1 (detection); AS-3 (best guess part)
- Size: S
- Scope: Tier 0 in `best_guess`: the open card whose ID equals `input.hint`; a hint naming a closed card, a card of another board or a missing card gives no tier 0. `GuessInput::from_env(env)`: `pane` is `env.herdr_pane_id` only when `env.herdr_active()`, else `None`; `cwd` and `home` come from `env`; `hint` is the card of `session::read_pane_hint(env, pane)` when a pane is set, else `None`. It reads one state file at most and writes nothing.
- Not in scope: writing hints (E1-F7-T3, called by E2-F3-T5 and E2-F3-T7); the picker text and the `o. other board:` line (E2-F2-T8, E2-F3-T9).
- Tests first:
  1. `fr38_hint_card_is_linked_here_and_first`: given input pane `w4V:p9`, hint `W-11` (Todo, no note) and `W-12` in Doing whose note has pane `w4V:p9`, when `best_guess` runs, then W-11 (`LinkedHere`) precedes W-12 (`SamePane`).
  2. `ts6_6_hint_to_card_not_open_here_gives_no_tier_0`: given the `work_4_9` board and, in turn, hints `W-9` (in Done), `H-3` and `W-99`, when `best_guess` runs, then no candidate is `LinkedHere` and the order equals the order without a hint.
  3. `ts6_6_from_env_reads_hint_of_current_pane`: given `env.herdr_env` `Some("1")`, `env.herdr_pane_id` `Some("w4V:p9")`, `env.cwd` `/work/billing` and `write_reference(env, "old", ..)` with W-11 and pane `w4V:p9`, when `GuessInput::from_env` runs, then `pane` is `w4V:p9`, `hint` is W-11, `cwd` is `/work/billing` and `home` is `env.home`.
  4. `ts6_6_outside_herdr_no_pane_and_no_hint`: given test 3's state with `env.herdr_env` `None`, and separately `Some("0")`, when `from_env` runs, then `pane` and `hint` are `None`.
  5. `ts6_6_from_env_without_hint_file`: given herdr active in pane `w4V:p9` and no state folder, when `from_env` runs, then `hint` is `None` and `<state>/brain-swap` does not exist.
  6. `as3_new_session_finds_previous_card_linked_here`: given the `work_4_9` board loaded, an env inside herdr in pane `w4V:p11` with cwd `/elsewhere`, and session `old` linked to W-11 in that pane through `write_reference` without any park, when `read_reference(env, "new")` returns `None` and `best_guess(&board, &GuessInput::from_env(&env))` runs, then the first candidate is W-11 with label `linked here` and W-12 follows.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `from_env` is the only function in `guess.rs` that touches the file system (review).

## Assumptions

- A-E1-01 Do `CardId` (with `parse`, `file_name` and `from_file_name`) and the plain structs `Place` and `HerdrPlace` live in E1-F1 rather than with the other section 3 types of E1-F2, so that E1-F2, E1-F7 and E5-F1 can start in parallel right after E1-F1? Affects E1-F1-T3, E1-F2-T3, E1-F7-T2, E1-F7-T3, E5-F1-T7.
- A-E1-02 Do templates read their `name` and `key` frontmatter with E1-F2's flat frontmatter reader (4.2), so that E1-F4-T1 depends on E1-F2-T1 although TECHSPEC 15 lists only E1-F1 as E1-F4's dependency? If yes, the next TECHSPEC revision adds E1-F2 to E1-F4's Dep line; if not, E1-F4 needs a reader of its own. Affects E1-F4-T1.
- A-E1-03 Does the store's cleanup (section 10: every write through `core::store` prunes `sessions/` and `panes/`) call E1-F7's `session::prune`, so that E1-F6-T8 depends on E1-F7-T4 although TECHSPEC 15 lists only E1-F4 and E1-F5 as E1-F6's dependencies? If yes, the next TECHSPEC revision adds E1-F7 to E1-F6's Dep line; E1-F7-T4 depends only on E1-F1-T6, so nothing is delayed meanwhile. If not, pruning moves out of the store. Affects E1-F6-T8, E1-F7-T4.
- A-E1-04 Does an invalid `BRAIN_SWAP_SESSION` warn like an invalid `--session` (`session id not substituted` for a placeholder, `invalid session id` otherwise) while an empty one counts as unset, with each warning text given at most once per resolution, since 6.4 states the warnings without naming their source? Affects E1-F7-T1.
- A-E1-05 Does `Card` carry an extra field `mtime: Option<Stamp>`, filled by `board::load` from the file's metadata and `None` from `card_file::parse`, since section 3's `activity()` falls back to the mtime but section 3's `Card` has no such field? Affects E1-F2-T3, E1-F5-T2, E1-F5-T3.
- A-E1-06 Does every `Error` variant carry its one-line message as a `String`, and does `Config` add a `msg` field to section 11's `{ path, line, col }` for the problem text? Affects E1-F1-T2.
- A-E1-07 Is exit 6 (jump not performed) a constant `EXIT_JUMP_NOT_PERFORMED` outside `Error`, since 6.2 gives exit 6 no error object and section 11 makes jump outcomes `PaneStatus` values? Affects E1-F1-T2.
- A-E1-08 May the `BRAIN_SWAP_LOG` sink live in a module `core::log` (`src/core/log.rs`) that the 2.1 module map does not list? Affects E1-F1-T8.
- A-E1-09 Is a `BRAIN_SWAP_LOG` path inside a board folder detected by a component-wise path prefix, once by `cli::cmd::Ctx::load` (E2-F1-T4) and `tui::prepare` (E4-F1-T8) after the config loads, with the warning `BRAIN_SWAP_LOG inside board <name>, ignored`? Affects E1-F1-T8.
- A-E1-10 Does an unparsable `BRAIN_SWAP_NOW` fail with `Error::InvalidInput("BRAIN_SWAP_NOW: invalid stamp '<v>'")` (exit 1) rather than fall back to the clock? This is the chosen rule: E2-F1-T1 test `ts6_1_invalid_brain_swap_now_exits_1` pins it at process level, and A-E2-33 is withdrawn. Affects E1-F1-T6.
- A-E1-11 Does an unset or empty `HOME` fail every invocation with `Error::Io("HOME not set")`, since every 5.1 path derives from it? Affects E1-F1-T6.
- A-E1-12 Is NFR-03 (no background process) enforced by the layering test allowing `Command::new` only in `src/adapters/runner.rs` and `src/tui/editor.rs`, the two places section 2 gives a reason to spawn? Affects E1-F1-T1.
- A-E1-13 Does the spawn helper's default scenario set `PATH=/usr/bin:/bin`, `BRAIN_SWAP_NOW=2026-10-08T11:52:00+03:00` (the 4.9 time) and `HERDR_BIN_PATH` to `tests/fake_herdr/herdr` even in scenarios outside herdr, with `xdg`, `unset` (only for the three `XDG_*` names) and `path` as the only setters that change or drop a default? Affects E1-F1-T9.
- A-E1-14 Is a `Z` offset read as `+00:00` and written back as `+00:00`, so such a stamp does not round-trip byte for byte (brain-swap never writes `Z`)? Affects E1-F1-T4.
- A-E1-15 Does `frontmatter::set` keep the key's spelling found in the file (`Column: Doing`), and is the dominant line ending CRLF only when CRLF line ends outnumber bare LF ones, a tie or a file without line ends giving LF? Affects E1-F2-T2.
- A-E1-16 Are the card reader's and renderer's warning texts `<file>:<line>: invalid frontmatter: <problem>`, `<file>: not UTF-8, read only`, `<file>:<line>: conflict marker, read only` and `## Timeline in body dropped`? Affects E1-F2-T5, E1-F2-T6.
- A-E1-17 Is a non-UTF-8 card shown from a lossy decode (title, column, notes) and marked read only, rather than left out of the board? Affects E1-F2-T5.
- A-E1-18 Does an appended note go right after the last non-blank line of the timeline, before trailing blank lines and any post section, rather than at the very end of the file? Affects E1-F2-T7.
- A-E1-19 Is a new card rendered with LF line ends, a blank line between the title and the body, the body's trailing blank lines trimmed, a blank line before `## Timeline` and a final newline (the 4.9 shape)? Affects E1-F2-T6.
- A-E1-20 Does a note's merge identity (7.6) span the bytes from its `###` heading to its last non-blank line, so the blank lines between notes belong to no block? Affects E1-F2-T3, E1-F6-T7.
- A-E1-21 Does a config without `default_board` default to `work`, and is a config without a `[boards]` table the `[boards] is empty` error at line 1, column 1? Affects E1-F3-T1, E1-F3-T2.
- A-E1-22 Are the config errors `default_board '<x>' is not in [boards]`, `[boards] is empty`, `board name '<n>' must match [a-z][a-z0-9_-]*`, `<key>: expected <type>`, `board <name>: path must be absolute or start with ~/` and the TOML parser's own message, and is an unknown top-level key the warning `config: unknown key '<key>'`? Affects E1-F3-T2.
- A-E1-23 Does the first-run config write each default board path as `~/...` when it lies under `HOME` and as an absolute path otherwise (a custom `XDG_DATA_HOME`)? Affects E1-F3-T3.
- A-E1-24 Does `core::keys` match a terminal-free `KeyPress` type, leaving the conversion from crossterm events to E4-F1, so that the core imports no TUI crate? Affects E1-F3-T4.
- A-E1-25 Are the key warnings `key config: <action>: <problem>` with the problems `empty binding`, `unknown key '<name>'`, `unknown modifier '<name>'`, `unknown action`, `conflicts with <other> on '<key>'` and `printable key '<k>' ignored in title input`, and is a user entry holding one unparseable string dropped whole? Affects E1-F3-T4, E1-F3-T5, E1-F3-T6.
- A-E1-26 Are user template files read in file name order, does a replacement keep the built-in's place in the list while a new name is appended, and does a missing template folder give the built-ins without a warning? Affects E1-F4-T2.
- A-E1-27 Does a template `key` that is not exactly one character give no hotkey with the warning `template <name>: key must be one character`, and does the first template in load order keep a duplicated hotkey? Affects E1-F4-T3.
- A-E1-28 Are files in a board folder that do not match the card pattern (including a lower-case `w-12.md`) ignored silently, while a card file with another board's letter warns? Affects E1-F5-T2.
- A-E1-29 Are duplicate `columns` entries (ignoring case) dropped with a warning, and does an empty `columns:` fall back to Todo, Doing, Done with a warning, keeping I3? Affects E1-F5-T1.
- A-E1-30 Are extra columns ordered by first appearance in card number order, and does `activity()` fall through to `created`, then the mtime, when the last note has a bad stamp? Affects E1-F5-T3.
- A-E1-31 Does a card ID resolve to the first board in config order with that letter (each later one warned as `boards <a> and <b> share letter <L>; using <a>`), and is a missing card `Error::NotFound("<ID> not found")` in the core, which every E2 subcommand passes through unchanged; only `context` keeps the 6.3 literal `reference: none (<ID> not found)` (A-E2-07), so every surface (CLI, `locate`, `jump`, the TUI) reports an unknown card alike? Affects E1-F5-T4, E1-F6-T3, E1-F6-T4, E1-F6-T7.
- A-E1-32 Is the `Busy` message `board busy, try again`, the TUI status text of section 10, also on the CLI? Affects E1-F6-T1.
- A-E1-33 Does `create` fail `Error::InvalidInput("no title")` for an empty trimmed title and `Error::Io("board <name>: no free card number after 100 tries")` after 100 link collisions? Affects E1-F6-T5, E1-F6-T6.
- A-E1-34 Does the editor merge refuse (`Unreadable`, nothing written) when notes or a column must be spliced into an edited copy that is itself read only, and is its R7 postcondition that every added block is present in the result? Affects E1-F6-T7.
- A-E1-35 Does the store's temp file cleanup match names that start with `.` and contain `.bs-tmp-`, compare their mtime with `env.now` (so `BRAIN_SWAP_NOW` drives it in tests) and ignore every error? Affects E1-F6-T8.
- A-E1-36 Do `read_reference` and `write_reference` accept only session IDs that I8 calls valid (an invalid one reads as none and fails the write with `Error::InvalidInput("invalid session id")`), so that a session ID never names a path outside `sessions/`? Affects E1-F7-T2.
- A-E1-37 Does `core::session` write references and hints with its own temp file, `sync_all` and `rename` (temp `.<name>.bs-tmp-<pid>` beside the target, no verify step), since TECHSPEC 15 lets E1-F7 depend only on E1-F1 and so not on E1-F6's `write_atomic`, and is E1-F6-T2's DoD grep ("`write_atomic` is the only function in `src/core/` that calls `rename`") read as excluding `src/core/session.rs`? Affects E1-F7-T2, E1-F7-T3, E1-F6-T2.
- A-E1-38 Is the reference file TOML with the keys `card`, `board`, `pane`, `cwd` and `set_at` in that order, `pane` and `cwd` left out when absent, does a file with a missing key, a malformed card ID or a bad `set_at` read as none, and is a `set_at` without offset read in UTC, since the serde `with` helper 6.4 asks for has no local zone? Affects E1-F7-T2.
- A-E1-39 Is a pane hint the TOML file `panes/<pane>` with the keys `card` and `session`, written by `write_reference` right after the reference whenever the reference has a pane, the last writer winning, and does a failure of either write return `Error::Io` to the caller? Affects E1-F7-T3.
- A-E1-40 Do the pane IDs `.` and `..`, which match the 6.4 pattern but name folders as file names under `panes/`, write no hint and read as none? Affects E1-F7-T3.
- A-E1-41 Does `prune` remove every regular file in `sessions/` and `panes/` (so also a temp file a crash left in `sessions/`, which `sessions/*.toml` would miss) whose mtime lies more than 30 times 24 hours before `env.now`, ignoring missing folders and every error? Affects E1-F7-T4, E1-F6-T8.
- A-E1-42 Is a card in an extra (unknown) column in tier 4 rather than tier 3, since extra columns render after the last configured column and so are not middle columns? Affects E1-F8-T1.
- A-E1-43 Within a best guess tier, does a card without any activity (no stamped note, no `created`, no mtime) sort after every card with one, then by the higher number? Affects E1-F8-T1.
- A-E1-44 For `same folder`, are paths compared lexically and component-wise (no canonicalization, so `/work/bill` is not inside `/work/billing`), is `$HOME` taken from `Env.home`, and does the `/` and `$HOME` exclusion apply only to the note's cwd as 6.6 reads, so that a current cwd of `$HOME` makes every card whose note cwd lies below it `same folder`? Affects E1-F8-T2.
