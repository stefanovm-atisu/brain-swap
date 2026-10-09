# E6 Release

Milestone: M5. This epic is the release check of v0.1: it does not add behaviour, it proves that the Core requirements already built in E1 to E5 hold together on the installed product. It finishes the README (install, the three install steps of TECHSPEC 13, the Claude Code pack, the herdr key, upgrade), measures NFR-01 and NFR-02 on a generated board of 300 cards with 20 notes each (an `#[ignore]` release test plus `hyperfine`), checks NFR-05 end to end, installs the binary with `cargo install` from a fresh checkout (FR-68), runs the acceptance scenarios AS-1 to AS-11 by hand in real herdr and Claude Code on that installed build, and makes the PRD section 9 success metrics measurable with what already exists. It sits last because TECHSPEC 15 makes it depend on every Core feature; the README task and the automated performance and parity tests can start as soon as their own dependencies are done, before the final manual runs. The manual Claude steps of AS-1, AS-2 and AS-3 are written once, in E3-F1-T8, E3-F1-T9 and E3-F1-T10; this epic repeats them on the installed pack by reference instead of copying them. Spec: TECHSPEC 12.1 (Performance and Manual bullets), 12.2, 13, 15 (E6 entry), 15.2; PRD FR-68, NFR-01, NFR-02, NFR-05, section 6 (AS-1 to AS-11), section 9 (SM-1 to SM-5), section 10.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E6-F1 | Release check | E0-F3, E1-F1, E1-F2, E1-F3, E1-F4, E1-F5, E1-F6, E1-F7, E1-F8, E2-F1, E2-F2, E2-F3, E3-F1, E3-F2, E4-F1, E4-F2, E4-F3, E4-F4, E4-F5, E5-F1, E5-F2, E5-F3, E5-F4 | 10 | L |

## E6-F1 Release check

- Depends on: E0-F3, E1-F1, E1-F2, E1-F3, E1-F4, E1-F5, E1-F6, E1-F7, E1-F8, E2-F1, E2-F2, E2-F3, E3-F1, E3-F2, E4-F1, E4-F2, E4-F3, E4-F4, E4-F5, E5-F1, E5-F2, E5-F3, E5-F4 (TECHSPEC 15: all Core features; E0-F3 added by A-E6-02)
- Covers: FR-68, NFR-01, NFR-02, NFR-05
- Spec: TECHSPEC 12.1 (Performance, Manual), 12.2, 13, 15 (E6-F1), 15.2; PRD 6 (AS-1 to AS-11), 8 (NFR-01, NFR-02, NFR-05), 9 (SM-1 to SM-5), 10
- Scope: The README gains its final shape: what brain-swap is, install with `cargo install`, the three install steps of TECHSPEC 13 (binary, `brain-swap install claude`, `brain-swap install herdr` with paste and `herdr server reload-config`), the pack, the herdr key and upgrade, checked by `tests/readme.rs`. E4-F1-T10's `write_big_board` generates the board of 300 cards with 20 notes each, changed in place to the A-E6-03 layout; `#[ignore]` release tests in `tests/perf.rs` time every NFR-01 subcommand and the NFR-02 Detail keys, move key and jump overhead, E4-F1-T10's `#[ignore]` first frame and key reaction tests in `tests/tui_perf.rs` are rerun on the finished screens, and the author records `hyperfine` numbers on the same board. A parity test runs the Core CLI flow inside and outside a fake herdr pane (NFR-05). The release build is installed from a fresh clone (FR-68), AS-1 to AS-11 are run by hand on it in three grouped runs, and `docs/metrics.md` states how each of SM-1 to SM-5 is measured with the product as built, the transcripts Claude Code already keeps and a stopwatch, adding no product feature.
- Provides: `README.md` sections `## Install`, `## Claude Code pack`, `## herdr key binding` (E5-F4-T3's section, kept as written), `## Upgrade` (A-E6-14); `tests/readme.rs` with `readme_section(text: &str, heading: &str) -> Option<String>`; the A-E6-03 layout of E4-F1-T10's `write_big_board(dir: &Path, cards: usize, notes: usize)` (changed in place, signature kept); `tests/common/timing.rs` with `p95_ms(samples: &mut [f64]) -> f64`, `time_runs(warmup: usize, runs: usize, f: impl FnMut()) -> Vec<f64>` and `herdr_ms(log: &str) -> f64`; `tests/perf.rs` (`#[ignore]` tests run with `cargo test --release --test perf -- --ignored --nocapture`); `<CARGO_TARGET_TMPDIR>/perf-board/` (written by `sm5_write_hyperfine_board`: `config/`, `data/work/`, `data/work.pristine/`, `state/`, `note.txt`, `env.sh`) (A-E6-20); `tests/nfr05_parity.rs`; `docs/metrics.md` (A-E6-19)
- Requires: E1-F1 `tests/common/spawn.rs` (`Scenario`, `brain_swap(&Scenario)`, `Scenario::herdr`, `Scenario::fake_scenario`, `Scenario::log`, `Scenario::session`), `tests/common/env.rs` `core_env(home, now)`, `tests/docs.rs`, `core::log::event` line format; E1-F2 `core::card_file::render_new`, `core::card_file::render_note`, `core::model::{NewCard, NewNote}`; E1-F3 `core::config::load`; E1-F5 `core::board::load`; E1-F6 `core::store::move_card`; E2-F1 `cli::args::Cli` (`Cli::try_parse_from`) and `tests/common/cli.rs` (`Sandbox::new()`, `write_config()`, `copy_board`, `cmd`, `scenario_mut`, `redact`); E2-F2 `tests/fixtures/boards/work/`; E2-F3 `park`, `link`, `context`; E3-F1 `tests/common/skill_md.rs` (`commands`, `substitute`, `shell_words`) and the procedures of E3-F1-T8, E3-F1-T9, E3-F1-T10; E3-F2 `install claude` (`--link`, `--force`, `--remove`) and its README paragraphs; E4-F1 `tui::prepare` (the start path `tui::run` uses), `tui::app::update`, `Input`, `Effect`, `Outcome`, `tui::view_frame::draw` (A-E6-07), and from E4-F1-T10 `tests/common/big_board.rs` `write_big_board` and the `#[ignore]` tests `nfr02_first_frame_within_150_ms` and `nfr02_key_reaction_within_50_ms` in `tests/tui_perf.rs`; E4-F2 and E4-F3 the board and detail views; E5-F1 the fake herdr script (`HERDR_BIN_PATH`, `FAKE_HERDR_SCENARIO`, argv log) and its scenario `pane-session`, and the `herdr` log lines (argv, exit, ms); E5-F2 `locate`, `jump` and the fake herdr scenario `live-agent`; E5-F3 the TUI jump; E5-F4 `install herdr` and its README section; E0-F3 the final step 3 of `bs-back/SKILL.md`
- Feature DoD:
  - [ ] README covers install, the three install steps, the pack, the herdr key and upgrade, and every `brain-swap` command in it parses (E6-F1-T1).
  - [ ] The `#[ignore]` release test on 300 cards with 20 notes passes for every NFR-01 subcommand at p95 50 ms or less (E6-F1-T2).
  - [ ] The `#[ignore]` NFR-02 tests pass: E4-F1-T10's first frame (150 ms) and key reaction (50 ms) tests rerun on the finished screens, and the Detail keys, move key and jump overhead tests of `tests/perf.rs` at p95 50 ms or less (E6-F1-T3).
  - [ ] `hyperfine` numbers for NFR-01 and NFR-02 (SM-5), plus the idle scan count, are recorded with the machine and versions (E6-F1-T4).
  - [ ] Without herdr, the Core CLI flow gives the same exit codes and output as inside a herdr pane, and calls no herdr command (E6-F1-T5).
  - [ ] `cargo install` from a fresh checkout yields the whole product; nothing else is needed at runtime; the same checkout type-checks for macOS (E6-F1-T6).
  - [ ] Manual AS-1, AS-2, AS-3 recorded on the installed build with the installed pack and herdr key (E6-F1-T7).
  - [ ] Manual AS-4, AS-5, AS-10, AS-11 recorded (E6-F1-T8).
  - [ ] Manual AS-6, AS-7, AS-8, AS-9 recorded (E6-F1-T9).
  - [ ] PRD section 9 measurable: `docs/metrics.md` gives each of SM-1 to SM-5 an instrument, a record and a pass rule, and each instrument was tried once (E6-F1-T10).

### E6-F1-T1 README: install, pack, herdr key and upgrade

- Status: todo
- Depends on: E2-F1, E3-F2, E5-F4
- Covers: FR-68; TECHSPEC 13 (install, upgrade), 12.2 item 5 (README for FR-64 and FR-67)
- Size: M
- Scope: Rewrite `README.md` from its idea-stage text into the v0.1 README, keeping the paragraphs earlier tasks added (the exit code table of E2-F1-T3, the `install claude` paragraphs of E3-F2, the herdr section of E5-F4) and putting them under fixed headings. `## Install`: requirements (Rust 1.89 or later as `rust-version` says, Linux; macOS should build, NFR-09), then three numbered steps in TECHSPEC 13 order: `cargo install --path .` from a checkout (the `--git <url>` form only once a public URL exists, A-E6-14) into `~/.cargo/bin`, optionally `alias bs=brain-swap`; `brain-swap install claude`; `brain-swap install herdr`, paste its snippet into herdr's config and run `herdr server reload-config`. `## Claude Code pack`: the four commands `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link` with one line each, `--link <repo>`, `--force`, `--remove`, the `/reload-skills` hint and the allow rule `Bash(brain-swap:*)`; settings are never edited. `## herdr key binding`: E5-F4-T3's section (the snippet, the default key as SP-7 settled it, a second binding with `--board home`), kept as written and placed between `## Claude Code pack` and `## Upgrade`. `## Upgrade`: repeat the three steps. Add `tests/readme.rs` with `readme_section` (the text from a `## ` heading to the next `## `) and the tests below, reusing `commands`, `substitute` and `shell_words` from E3-F1's `tests/common/skill_md.rs`.
- Not in scope: the exit code table and its agreement with `--help` (E2-F1-T3); the behaviour of `install claude` and `install herdr` (E3-F2, E5-F4); the content of `## herdr key binding` and its snippet test (E5-F4-T3); running the steps on a clean machine (E6-F1-T6); the success metrics (E6-F1-T10).
- Tests first: docs layer, except test 4 (CLI).
  1. `fr68_readme_install_lists_three_steps_in_order`: given `README.md`, when `readme_section(text, "## Install")` is read, then it holds a fenced line starting `cargo install`, then one `brain-swap install claude`, then one `brain-swap install herdr`, in that order, and the words `herdr server reload-config`.
  2. `ts13_readme_upgrade_repeats_the_three_steps`: given `README.md`, when the `## Upgrade` section is read, then it holds the same three command prefixes in the same order.
  3. `ts13_readme_brain_swap_commands_parse`: given every command `commands` finds in `README.md` (fenced lines and code spans starting `brain-swap `), when each is passed through `substitute` with a fixed session and `shell_words`, then `Cli::try_parse_from` accepts every one.
  4. `fr64_readme_herdr_key_matches_install_herdr`: given the sandbox, when `brain-swap install herdr` runs, then the value of its `key = "..."` line equals the value of the `key = "..."` line in the `## herdr key binding` section.
  5. `fr67_readme_pack_section_names_commands_and_hints`: given the `## Claude Code pack` section, then it names `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`, `--link`, `--force`, `--remove`, `/reload-skills` and `Bash(brain-swap:*)`.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the README turned it green (12.2 item 1).
  - [ ] `tests/docs.rs` passes on the new README (no U+2014 or U+2013).
  - [ ] The README no longer says `idea stage, no code yet`, and links `PRD.md`, `TECHSPEC.md` and `docs/metrics.md` once that file exists.

### E6-F1-T2 Performance board and NFR-01 test

- Status: todo
- Depends on: E2-F3, E4-F1-T10
- Covers: NFR-01; TECHSPEC 12.1 (Performance); PRD SM-5
- Size: M
- Scope: Change E4-F1-T10's `write_big_board(dir, cards, notes)` in place, keeping its signature and canonical files, so that `write_big_board(dir, 300, 20)` writes the board of A-E6-03 (its columns, titles, place lines and stamps); E4-F1-T10's `tests/tui_perf.rs` keeps passing. `tests/common/timing.rs`: `p95_ms` sorts the samples and returns the value at rank `ceil(0.95 * n)` (A-E6-04); `time_runs(warmup, runs, f)` calls `f` `warmup` times untimed, then `runs` times, returning wall milliseconds from `std::time::Instant`. "The perf sandbox" is built by each `#[ignore]` test for itself, so no test sees another's writes: E2-F1 `Sandbox::new()` plus `write_config()`, `write_big_board(<sandbox work board folder>, 300, 20)`, then `link W-296 --session perf-s1` once. `tests/perf.rs` times each NFR-01 subcommand through the 12.1 spawn helper outside herdr (no `HERDR_*`, so no herdr wait is included), 3 warm-ups and 20 timed runs, printing `nfr01 <command>: p95 <x> ms`. Every `#[ignore]` test first asserts `!cfg!(debug_assertions)` with the message `run with cargo test --release`. Write commands let the board grow (A-E6-05). `sm5_write_hyperfine_board` writes the same board with `write_big_board` for the `hyperfine` runs to `env!("CARGO_TARGET_TMPDIR")/perf-board/` (A-E6-20).
- Not in scope: TUI timings and the jump overhead (E4-F1-T10, E6-F1-T3); the `hyperfine` runs and the recorded numbers (E6-F1-T4); `locate`, `jump` and `install`, which NFR-01 excludes.
- Tests first: core unit for 1, round trip for 2, performance (`#[ignore]`, CLI spawn) for 3 to 12.
  1. `nfr01_p95_takes_rank_ceil_of_095_n`: given the samples 1 to 20 ms in shuffled order, when `p95_ms` runs, then it returns 19.0; and given 1 to 50, then 48.0.
  2. `nfr01_perf_board_loads_300_cards_with_20_notes`: given a temp folder written by `write_big_board(dir, 300, 20)`, when `core::board::load("work", dir, &env)` runs, then it has 300 cards, every card has 20 notes, 10 cards are open (5 Todo, 5 Doing), `meta.next` is 301 and `warnings` is empty.
  3. `nfr01_templates_p95_under_50ms`: given the perf sandbox, when `brain-swap templates` is timed, then p95 is at most 50 ms.
  4. `nfr01_context_p95_under_50ms`: given the perf sandbox, when `brain-swap context --session perf-s1` is timed, then every run exits 0 with `reference: W-296` and p95 is at most 50 ms.
  5. `nfr01_show_all_p95_under_50ms`: given the perf sandbox, when `brain-swap show W-150 --all` is timed, then p95 is at most 50 ms.
  6. `nfr01_ls_p95_under_50ms`: given the perf sandbox, when `brain-swap ls` is timed, then p95 is at most 50 ms.
  7. `nfr01_ls_open_guess_p95_under_50ms`: given the perf sandbox, when `brain-swap ls --open --guess` is timed, then p95 is at most 50 ms.
  8. `nfr01_link_p95_under_50ms`: given the perf sandbox, when `brain-swap link W-297 --session perf-s2` is timed, then p95 is at most 50 ms.
  9. `nfr01_move_p95_under_50ms`: given the perf sandbox, when `brain-swap move W-291 Doing` and `brain-swap move W-291 Todo` are timed alternately, then p95 is at most 50 ms.
  10. `nfr01_park_p95_under_50ms`: given the perf sandbox, when `brain-swap park --card W-298 --session perf-s3` with three part lines on stdin is timed, then every run prints `parked to W-298: ` and p95 is at most 50 ms.
  11. `nfr01_new_p95_under_50ms`: given the perf sandbox, when `brain-swap new perf card` is timed, then every run exits 0 and p95 is at most 50 ms.
  12. `sm5_write_hyperfine_board`: given `CARGO_TARGET_TMPDIR`, when the test runs, then `perf-board/` holds a config whose `work` board is `perf-board/data/work`, that board loads with 300 cards, `data/work.pristine/` is a byte copy of it, `note.txt` holds `Doing:`, `Next:` and `Watch out:` lines, and `env.sh` exports `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `BRAIN_SWAP_NOW` and unsets `HERDR_ENV`, `BRAIN_SWAP_SESSION` and `BRAIN_SWAP_LOG`.
- DoD:
  - [ ] The tests above pass (3 to 12 with `cargo test --release --test perf -- --ignored`); the first test was committed red before `p95_ms` turned it green (12.2 item 1).
  - [ ] Tests 1 and 2 are not ignored and run in the normal `cargo test --all-features` suite.
  - [ ] No second board generator exists; `cargo test --release --test tui_perf -- --ignored` still passes with the changed `write_big_board`.
  - [ ] Running the ignored tests in a debug build fails at once with `run with cargo test --release`.
  - [ ] No test spawns a process except through `tests/common/spawn.rs` (the `tests/layering.rs` rule passes).

### E6-F1-T3 NFR-02 tests: first frame, keys and jump overhead

- Status: todo
- Depends on: E6-F1-T2, E4-F1-T10, E4-F2, E4-F3, E5-F2
- Covers: NFR-02; TECHSPEC 12.1 (Performance), 7.1, 9.3
- Size: M
- Scope: First frame and Board navigation key reaction are E4-F1-T10's `#[ignore]` tests `nfr02_first_frame_within_150_ms` and `nfr02_key_reaction_within_50_ms` in `tests/tui_perf.rs`, on the `write_big_board(dir, 300, 20)` board of E6-F1-T2; this task reruns them once E4-F2 and E4-F3 have finished the screens and adds no second version of them. It adds only what they do not measure: more `#[ignore]` tests in `tests/perf.rs` on the perf sandbox, each with the release guard of E6-F1-T2, 3 warm-ups and 20 timed runs, printing `nfr02 <what>: p95 <x> ms`. Key reactions are timed in process, since no pty crate exists (A-E6-06): the TUI start path is `tui::prepare` as E4-F1-T10 uses it, and a key reaction is `tui::app::update` with that key, running any emitted effect the way the E4 runtime does, then one `tui::view_frame::draw` on a ratatui `TestBackend` of 120x40 (A-E6-07). The jump overhead runs `brain-swap jump` through `Sandbox::cmd` inside a fake herdr pane, the `HERDR_*`, `FAKE_HERDR_SCENARIO` and `BRAIN_SWAP_LOG` variables set through `Sandbox::scenario_mut()` before `cmd(args)`, and subtracts the herdr call durations of the log from the wall time (A-E6-08). `herdr_ms(log)` in `tests/common/timing.rs` sums the duration field of every `herdr` line in the E5-F1 format. The jump uses E5-F2's fake herdr scenario `live-agent` (`pane get w4V:p9` answers a live agent pane with tab `w4V:t1` and workspace `w4V`; `agent focus w4V:p9` exits 0).
- Not in scope: the first frame and Board navigation key tests and `write_big_board` itself (E4-F1-T10, layout changed in E6-F1-T2); idle directory scans (manual in E6-F1-T4); the jump's behaviour per `PaneStatus` (E5-F2, E5-F3); CLI subcommand timings (E6-F1-T2).
- Tests first: core unit for 1, performance (`#[ignore]`; TUI `update` and `TestBackend`, or CLI with fake herdr) for 2 to 4.
  1. `nfr02_herdr_log_durations_are_summed`: given a log text with two `herdr` lines of 12 ms and 30 ms and one `reload` line, when `herdr_ms` runs, then it returns 42.0.
  2. `nfr02_detail_keys_p95_under_50ms`: given an `App` from `tui::prepare` on the perf sandbox with W-300 (20 notes) selected, when `o`, then `j` and `k` in Detail are each fed and drawn, then p95 over all samples is at most 50 ms and the last frame shows `note 20/20`.
  3. `nfr02_move_key_p95_under_50ms`: given an `App` with W-291 selected in Todo, when `L` is fed and its `Effect::Move` is run through `core::store::move_card`, the board reloaded with `core::board::load`, the `Input::Done(Outcome::Moved(..))` fed and the frame drawn (alternating `L` and `H`), then p95 is at most 50 ms.
  4. `nfr02_jump_overhead_p95_under_50ms`: given the perf sandbox inside fake herdr pane `w4V:p9` with `FAKE_HERDR_SCENARIO=live-agent` and `BRAIN_SWAP_LOG=<sandbox>/state/bs.log` truncated before each run, when `brain-swap jump W-300` runs, then every run prints `jumped to W-300 (pane w4V:p9)` and exits 0, and p95 of wall ms minus `herdr_ms(log)` is at most 50 ms.
- DoD:
  - [ ] The tests above pass (2 to 4 with `--ignored` in release); the first test was committed red before `herdr_ms` turned it green (12.2 item 1).
  - [ ] Test 1 is not ignored.
  - [ ] `cargo test --release --test tui_perf -- --ignored --nocapture` passes on the finished screens (E4-F2, E4-F3 done), and `tests/perf.rs` holds no first frame or Board navigation key test.
  - [ ] No TUI test needs a terminal: every draw goes to `TestBackend`.

### E6-F1-T4 hyperfine runs for NFR-01 and NFR-02

- Status: todo
- Depends on: E6-F1-T2, E6-F1-T3
- Covers: NFR-01, NFR-02; PRD SM-5; TECHSPEC 12.1 (Performance)
- Size: S
- Scope: The author measures the release build on the generated board with `hyperfine` (SM-5), outside herdr and on the author's machine as NFR-01 states, derives p95 from the exported samples (A-E6-04), reruns the `#[ignore]` tests for the in-process NFR-02 numbers, and counts idle directory scans with `strace` (A-E6-09). Writes run against a board restored before each sample (`--prepare`, A-E6-05). Nothing in the repository changes unless a target is missed, in which case a bug task is opened and this task repeats after the fix.
- Not in scope: writing the tests or the generator (E4-F1-T10, E6-F1-T2, E6-F1-T3); the two-week SM-5 rerun after v0.1 (E6-F1-T10 says how).
- Procedure:
  1. On a quiet machine on AC power, record CPU model, kernel, `rustc --version`, `hyperfine --version` (1.17 or later, for `--input`) and the commit hash; `cargo build --release`.
  2. Run `cargo test --release --test perf -- --ignored --nocapture` and `cargo test --release --test tui_perf -- --ignored --nocapture`; record every printed `nfr01` and `nfr02` line; pass when all tests pass.
  3. Run `cargo test --release --test perf -- --ignored sm5_write_hyperfine_board`, then `. target/tmp/perf-board/env.sh`, set `B=$PWD/target/release/brain-swap` and `D=$PWD/target/tmp/perf-board/data`, and run `$B link W-296 --session perf-s1` once.
  4. Reads: `hyperfine -N --warmup 3 --runs 50 --export-json target/tmp/hf-read.json "$B templates" "$B context --session perf-s1" "$B show W-150 --all" "$B ls" "$B ls --open --guess"`.
  5. Writes, each with `--prepare "rm -rf $D/work && cp -a $D/work.pristine $D/work"`, `--warmup 3 --runs 50` and an export to `target/tmp/hf-write.json`: `$B move W-291 Doing`, `$B link W-297 --session perf-s2`, `$B new perf card`, and `$B park --card W-298 --session perf-s3 --auto` with `--input target/tmp/perf-board/note.txt`.
  6. Compute p95 per command: `jq -r '.results[] | "\(.command) \((.times | sort | .[((length * 0.95) | ceil) - 1]) * 1000) ms"' target/tmp/hf-*.json`; pass when every value is at most 50 ms (NFR-01).
  7. Process start: `hyperfine -N --warmup 3 --runs 50 "$B templates"` gives the start-up floor that the in-process first frame of step 2 excludes; pass when that floor plus the first-frame time that E4-F1-T10's `nfr02_first_frame_within_150_ms` printed in step 2 is at most 150 ms (A-E6-06).
  8. Idle scans: run `$B` (the TUI) on the perf board in one terminal, wait 5 seconds without a key, then in another run `timeout 10 strace -f -e trace=openat -p "$(pgrep -n -x brain-swap)" 2>&1 | grep -c "$D/work\""`; pass when the count is 9 to 11 (one scan per second, NFR-02).
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the step 1 machine and versions, every value of steps 2, 6 and 7, the step 8 count, and pass or fail per NFR-01 and NFR-02 target; this replaces 12.2 items 1 and 3 (TECHSPEC 12.2).
  - [ ] A missed target is filed as a bug against the owning feature (E2 for a subcommand, E4 for a frame or key, E5-F2 for the jump) and this task is rerun after the fix.
  - [ ] The `target/tmp/` files are not committed.

### E6-F1-T5 NFR-05 parity without adapters

- Status: todo
- Depends on: E2-F3, E5-F2
- Covers: NFR-05; FR-58, FR-62; TECHSPEC 9.5
- Size: S
- Scope: `tests/nfr05_parity.rs` runs one Core CLI flow in two sandboxes built from E2-F2's `tests/fixtures/boards/work/` (A-E6-10): outside herdr (no `HERDR_*`) and inside fake herdr pane `w4V:p9` (tab `w4V:t1`, workspace `w4V`, `FAKE_HERDR_SCENARIO=pane-session`), both with the fake herdr argv log enabled; the inside sandbox sets its `HERDR_*` and `FAKE_HERDR_SCENARIO` variables through `Sandbox::scenario_mut()` before `cmd(args)`. The flow, in order: `new parity check --session parity-s1`, `move W-13 Doing`, `park --card W-13 --session parity-s1` with `Doing: a`, `Next: b`, `Watch out: c` on stdin, `link W-13 --session parity-s1`, `context --session parity-s1`, `show W-13 --all`, `ls`, `ls --open --guess`, `templates`. Each step is run as a CLI test through `Sandbox::cmd`; stdout is passed through each sandbox's `redact` (so both roots read `<sandbox>`) and compared after removing every ` (pane w4V:p9)` suffix and the labels `, linked here`, `, same pane` and `, same folder`, because a pane is the only thing an adapter adds; the `ls --open --guess` output is compared as the set of card IDs, since the herdr tiers of 6.6 may reorder it.
- Not in scope: `locate` and `jump` per `PaneStatus` (E5-F2); the pack, which NFR-05 exempts (E3-F1); the TUI, checked by hand outside herdr in E6-F1-T8 (AS-5).
- Tests first: CLI (`assert_cmd`) with the fake herdr script.
  1. `nfr05_cli_flow_exit_codes_match_inside_and_outside_herdr`: given the two sandboxes, when the flow runs in each, then every step exits 0 in both.
  2. `nfr05_cli_flow_output_matches_after_masking_the_pane`: given the two sandboxes, when the flow runs in each, then each step's masked stdout is equal between them, and `ls --open --guess` lists the same card IDs.
  3. `nfr05_outside_herdr_runs_no_herdr_command`: given the outside sandbox, when the flow runs and then `locate W-13` and `jump W-13`, then the fake herdr argv log is empty, `locate` prints `not in herdr` and `Where: <sandbox>/elsewhere` with exit 0, and `jump` prints the same lines with exit 6.
- DoD:
  - [ ] The tests above pass; the first test was committed red before the parity harness turned it green (12.2 item 1).
  - [ ] Every sandbox passes a valid `--session` where the flow takes one, so no step depends on `pane_session` (T-15).

### E6-F1-T6 Clean cargo install from a fresh checkout

- Status: todo
- Depends on: E6-F1-T1, E3-F2, E4-F5, E5-F3, E5-F4
- Covers: FR-68, NFR-09 (macOS build); TECHSPEC 13 (install, upgrade), 8.8, 9.4
- Size: S
- Scope: Prove FR-68 by hand: a fresh clone at the release commit, built with `cargo install` and nothing else, yields the TUI, every subcommand, the templates and the pack, and runs with no checkout, no herdr and no Claude Code present. The same clone is type-checked for macOS (NFR-09 "macOS should build"), and the test suite is run on a macOS machine when one is available (A-E6-21). Then follow the README's three install steps for the author's real setup and repeat them as an upgrade, which prepares E6-F1-T7 to T9 (A-E6-13).
- Not in scope: the acceptance scenarios (E6-F1-T7 to T9); `cargo package` contents (E3-F1-T6 DoD).
- Procedure:
  1. `T=$(mktemp -d)`; `git clone <this repository> "$T/src"` and check out the release commit; record its hash.
  2. `cargo install --locked --path "$T/src" --root "$T/root"`; pass when it exits 0 and `ls "$T/root/bin"` shows only `brain-swap`.
  3. `ldd "$T/root/bin/brain-swap"`; pass when it lists only system C runtime libraries (`linux-vdso`, `libc`, `libm`, `libgcc_s`, the loader); record the list.
  4. macOS build: `rustup target add aarch64-apple-darwin`, then `cargo check --locked --all-targets --all-features --target aarch64-apple-darwin --manifest-path "$T/src/Cargo.toml"`; pass when it exits 0 (NFR-09).
  5. macOS tests: if a macOS machine is available, run `cargo test --all-features` on it and record the result (date, pass or fail, macOS and toolchain versions) in the commit message (NFR-09); otherwise record that no macOS machine was available.
  6. `rm -rf "$T/src"`, then `cd /` and run with a cleared environment: `env -i HOME="$T/home" PATH="$T/root/bin" brain-swap ls`; pass when stderr holds `created config $T/home/.config/brain-swap/config.toml` and it exits 0.
  7. With the same `env -i` prefix: `brain-swap new hello`, `printf 'Doing: a\nNext: b\nWatch out: c\n' | brain-swap park --card W-1`, `brain-swap show W-1`, `brain-swap templates`; pass when all exit 0 and `templates` lists Feature, Bug, Research and Chore with their keys.
  8. With the same prefix: `brain-swap install claude`; pass when `$T/home/.local/share/brain-swap/pack/0.1.0/skills/` holds the four `SKILL.md` files, `$T/home/.claude/skills/bs-*` are symlinks to them, and each file equals `git -C <the author's repository> show <hash>:pack/claude/skills/<name>/SKILL.md` (or differs only by the SP-3 absolute path rewrite).
  9. With the same prefix: `brain-swap install herdr`; pass when the snippet's `command` is `$T/root/bin/brain-swap`.
  10. `rm -rf "$T"`. Then, for the author's setup, follow `## Install` of the README from a fresh clone exactly as written: `cargo install --path .` into `~/.cargo/bin`, `brain-swap install claude`, `brain-swap install herdr` pasted into herdr's config and `herdr server reload-config`; pass when each step works as the README says, with no step missing.
  11. Run the `## Upgrade` steps once more on the same build; pass when `install claude` replaces its own links without `--force` and the herdr key still opens the board.
- DoD:
  - [ ] One commit (empty unless the README needed a fix) whose message records the date, the commit hash, the `rustc` version, the step 3 library list, the step 4 result, the step 5 macOS test result (or that no macOS machine was available) and each step as passed or failed; this replaces 12.2 items 1 and 3.
  - [ ] A README step that did not work as written is fixed in the same commit, with `tests/readme.rs` kept green, and steps 10 and 11 rerun.

### E6-F1-T7 Manual release run: AS-1, AS-2, AS-3

- Status: todo
- Depends on: E6-F1-T6, E0-F3
- Covers: FR-19, FR-20, FR-22, FR-26, FR-29, FR-59; AS-1 (all steps), AS-2, AS-3 (manual, installed build)
- Size: M
- Scope: Run AS-1 to AS-3 end to end on the build, pack and herdr key installed by E6-F1-T6, in real herdr and Claude Code (A-E6-01). The Claude steps reuse the procedures and pass criteria of E3-F1-T8, E3-F1-T9 and E3-F1-T10 by reference, now against the installed pack (symlinks into `~/.local/share/brain-swap/pack/`, not the checkout) and the final `bs-back` step 3 of E0-F3; the board steps of AS-1 (2, 5, 6, 7) are written here. Scratch XDG folders keep the author's boards untouched, and a temporary herdr key opens the board on them (A-E6-11).
- Not in scope: the jump fallbacks (E6-F1-T8); AS-6 to AS-9 (E6-F1-T9); scoring staleness (E0-F3).
- Procedure:
  1. Check `ls -l ~/.claude/skills/bs-*` points into `~/.local/share/brain-swap/pack/0.1.0/skills/`, and the installed `prefix+alt+b` key opens the author's real board in a popup (`q` closes it).
  2. Set up the scratch environment `$S` as E3-F1-T8 step 2 (W-1 to W-11, a note on W-7) and run `git init` plus a commit in its work board folder. Add a temporary `[[keys.command]]` popup binding (key recorded) whose `command` is `env XDG_CONFIG_HOME=$S/config XDG_DATA_HOME=$S/data XDG_STATE_HOME=$S/state <installed brain-swap path>`, then `herdr server reload-config`.
  3. AS-1 step 1: E3-F1-T8 step 3 in herdr pane A (ID recorded).
  4. AS-1 step 2: open the board with the temporary key; pass when W-12 is selected in Todo (newest activity, FR-19). Press `L`; pass when W-12 sits in Doing still selected and `git diff` shows only its `column:` line changed. Quit, then in pane A type `/bs-park` once (the older note).
  5. AS-1 step 3: in pane A type `/bs-park next: rerun migration test, watch timeout`; pass criteria as E3-F1-T8 steps 5 and 6 (reply `parked to W-12: migrate invoices to v13`, Next and Watch out verbatim, `auto`, column unchanged).
  6. AS-1 step 4: E3-F1-T8 step 7 in pane B (ID recorded), without its closing `/bs-park` in pane A so that W-12 keeps two notes; pass instead when `brain-swap context --session <pane A's agent_session.value from herdr pane get>` still prints `reference: W-12`.
  7. AS-1 step 5: press the temporary key in pane B; pass when Doing shows `W-12 migrate invoices to v13` over `<age>  rerun migration test`, and the preview panel shows `W-12 latest: <age> ago, auto, pane <pane A>, <cwd>` and the first line of each part.
  8. AS-1 step 6: press `o`; pass when the detail view shows `note 2/2`; `j` shows `note 1/2` with the older note, `k` shows `note 2/2` again.
  9. AS-1 step 7: press Enter; pass when the popup closes and herdr focuses pane A.
  10. AS-1 step 8: E3-F1-T8 step 9 in pane A.
  11. AS-2: E3-F1-T9 steps 2 to 7 in the same scratch environment.
  12. AS-3: E3-F1-T10 steps 2 to 9 in the same scratch environment.
  13. Remove the temporary binding and run `herdr server reload-config`.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the commit hash, the Claude Code and herdr versions, the pane IDs, each procedure step as passed or failed with the observed text, and every question or extra text Claude produced despite "ask nothing" (TECHSPEC 14 unspiked risk); this replaces 12.2 items 1 and 3.
  - [ ] A failed step is filed as a bug against the owning feature, fixed there, and the failed scenario rerun before this task closes.
  - [ ] The temporary herdr binding is gone and the author's real boards are unchanged.

### E6-F1-T8 Manual release run: AS-4, AS-5, AS-10, AS-11

- Status: todo
- Depends on: E6-F1-T6
- Covers: NFR-05, FR-58, FR-59, FR-60, FR-61, FR-62, FR-63; AS-4, AS-5, AS-10, AS-11 (manual, installed build)
- Size: M
- Scope: Run the jump fallbacks by hand on the installed build: a closed pane, a moved pane, herdr not responding, and use outside herdr, including the pack and the TUI in a plain terminal (NFR-05). Scratch environment and temporary key as E6-F1-T7 steps 2 and 13. herdr 0.8.2 has no documented pane move command, so AS-10 uses whatever herdr offers and records it (A-E6-12); AS-11 freezes the herdr server with `SIGSTOP` and watches the TUI from a terminal outside herdr (A-E6-11).
- Not in scope: the `PaneStatus` logic and its fake herdr tests (E5-F2, E5-F3); AS-1 step 7, the live jump (E6-F1-T7).
- Procedure:
  1. Set up as E6-F1-T7 step 2; in herdr pane A start Claude Code with `$S` exported, `/bs-link W-12`, `/bs-park`; record pane A's ID and the note's `cwd`.
  2. AS-4: quit Claude Code in pane A and close pane A. Open the board with the temporary key and press Enter on W-12; pass when the TUI stays open with a message showing `pane closed`, the `cwd` and `open a session there and run /bs-link W-12`. In a shell, `brain-swap locate W-12` prints `closed <pane A>` and `Where: <cwd>` with exit 0, and `brain-swap jump W-12` exits 6.
  3. AS-4 continued: open pane C in that `cwd`, start Claude Code, `/bs-link W-12`, `/bs-park`; pass when the new note's place line names pane C and Enter on W-12 in the popup now focuses pane C.
  4. AS-10: move pane C to another herdr workspace while its Claude Code session runs (record how, and the new ID from `herdr pane list`). Press Enter on W-12 in the popup; pass when herdr focuses the moved pane and the TUI exits. From another pane run `brain-swap jump W-12`; pass when it prints `pane moved: now <new ID>` and exits 0.
  5. AS-5 outside: open a terminal emulator outside herdr, check `env | grep ^HERDR_` prints nothing, export `$S`. Run `brain-swap`; pass when the board, `o`, `j`, `k`, `L`, `H` and `?` behave as inside herdr, and Enter on W-12 shows `not in herdr` and the `cwd` with the TUI still open.
  6. AS-5 pack outside: in that terminal start Claude Code, `/bs-link W-12`, `/bs-park`, `/bs-back`; pass when the replies match AS-1 (`linked to W-12: ...`, `parked to W-12: ...`, the note printed) and the new note's place line holds `cwd=` and `session=` but no `pane=`.
  7. AS-5 inside: open the popup in herdr and press Enter on W-12; pass when it shows `no pane recorded` and the `cwd`; `brain-swap locate W-12` prints `no pane recorded`.
  8. AS-11 setup: in herdr pane D (a plain shell with `$S` exported) park W-12 with `brain-swap park --card W-12` (three parts), so the jump note has pane D; save `env | grep ^HERDR_` from pane D to `$S/herdr.env`. In a terminal outside herdr export `$S/herdr.env` and `$S`, and run `brain-swap`.
  9. AS-11: find the herdr server process (record the command used) and `kill -STOP` it. Press Enter on W-12 with a stopwatch running; pass when `herdr not responding` and the `cwd` appear within 3 seconds and `Esc`, `j`, `k` and `o` still work.
  10. AS-11 CLI: in a second shell of that outside terminal, `time brain-swap jump W-12`; pass when it prints `herdr not responding` and `Where: <cwd>`, exits 6, and real time is at most 3.5 seconds (A-E6-15).
  11. `kill -CONT` the server; pass when herdr recovers and Enter on W-12 in the popup focuses pane D. Remove the temporary binding.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the commit hash, the herdr and Claude Code versions, pane IDs, how the pane was moved (step 4) and how the server was found (step 9), the measured seconds of steps 9 and 10, and each step as passed or failed with the observed text; this replaces 12.2 items 1 and 3.
  - [ ] A failed step is filed as a bug against E5-F2 or E5-F3 (or E3-F1 for step 6), fixed there, and rerun before this task closes.
  - [ ] The herdr server is running again (`SIGCONT`), the temporary binding is gone and the author's real boards are unchanged.

### E6-F1-T9 Manual release run: AS-6, AS-7, AS-8, AS-9

- Status: todo
- Depends on: E6-F1-T6
- Covers: NFR-05, FR-04, FR-06, FR-23, FR-29, FR-30, FR-31, FR-34, FR-59, FR-66, NFR-06, NFR-08; AS-6, AS-7, AS-8, AS-9 (manual, installed build)
- Size: M
- Scope: Run by hand the scenarios that need no Claude Code or concern the files: the first run with no config (AS-9, with one `/bs-card` on day one), TUI and CLI only (AS-6, including the real-terminal prompts of `park`, which E2-F3-T6 also runs once under util-linux `script` (A-E2-35)), hand edits (AS-7) and simultaneous parks (AS-8). Each board folder is a git repository so `git diff` shows exactly what changed. Scratch XDG folders as in E6-F1-T7.
- Not in scope: the automated halves (E2-F3-T12 for the CLI half of AS-8, E4-F5 for the editor merge, E1-F5 for the AS-7 board load); the jump fallbacks (E6-F1-T8).
- Procedure:
  1. AS-9: make empty `$F` and export `XDG_CONFIG_HOME=$F/config`, `XDG_DATA_HOME=$F/data`, `XDG_STATE_HOME=$F/state`. Run `brain-swap`; pass when it shows `created config $F/config/brain-swap/config.toml` and an empty work board with Todo, Doing and Done, the file holds TECHSPEC 5.2 with boards under `$F/data/brain-swap/`, and no `work` folder exists yet.
  2. AS-9 continued: press `n`, `f`, type `first card`, Enter; pass when `W-1 first card` is selected in Todo, and `$F/data/brain-swap/work/` now holds `W-1.md` and a `board.md` of exactly `---`, `next: 2`, `---`.
  3. AS-9 subcommand: with a new empty `$G` exported the same way, run `brain-swap ls`; pass when stderr holds `created config <path>` and it exits 0. With a new empty `$H`, start Claude Code in a herdr pane and type `/bs-card try brain-swap on day one`; pass when it replies `created W-1: try brain-swap on day one (Todo, work)`.
  4. AS-6 setup: new scratch `$K` with W-1 to W-11 made by `brain-swap new`, `git init` and a commit in the board folder; export `$K`'s XDG folders in every shell and pane used in steps 5 to 12.
  5. AS-6 TUI: in a herdr pane running a plain shell (no Claude Code), run `brain-swap`, keep Todo focused, press `n`, `f`, type `tui only card`, Enter; pass when `W-12 tui only card` appears in Todo, selected. Press `L`; pass when it moves to Doing, still selected. Quit.
  6. AS-6 park: in that shell run `brain-swap park --card W-12`; pass when it prompts `Doing: `, `Next: ` and `Watch out: ` one at a time, replies `parked to W-12: tui only card`, and the note heading has no `auto` and the place line names this pane.
  7. AS-6 jump: from a pane in another tab, run `brain-swap` and press Enter on W-12; pass when herdr shows the tab holding the shell pane and the TUI exits. Run `brain-swap jump W-12`; pass when it prints `jumped to W-12 (tab <tab>; pane not focusable)`.
  8. AS-7: with the TUI open on W-12, change W-12's title line in another pane with an editor; pass when the TUI shows the new title within 2 seconds (stopwatch) with W-12 still selected.
  9. AS-7 continued: `printf '# spike on caching\n' > W-20.md` in the board folder; pass when W-20 appears in Todo with `no note` within 2 seconds, and `brain-swap new next card` replies `created W-21: next card (Todo, work)`. Set `column: Doign` in `W-11.md`; pass when a column `Doign (unknown)` appears after Done holding W-11, and `?` lists the warning.
  10. AS-7 append: commit, then park W-20 and W-11 from stdin; pass when `git diff` shows only added lines (a `## Timeline` heading and the note for W-20, the note alone for W-11).
  11. AS-8: commit; with the TUI open, run `( printf 'Doing: a\nNext: b\nWatch out: c\n' | brain-swap park --card W-12 --session as8-a & printf 'Doing: d\nNext: e\nWatch out: f\n' | brain-swap park --card W-7 --session as8-b & wait )`; pass when both print `parked to ...`, `git diff --stat` lists only `W-12.md` and `W-7.md` with additions only, and the TUI shows both at age `now` within 2 seconds. Repeat with both parks on W-12; pass when W-12 gains both notes.
  12. AS-8 editor: select W-12, press `e`; while the editor is open, park a note on W-12 from another pane; then change a body line, save and quit; pass when `W-12.md` holds the body change and the parked note.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the commit hash, the herdr and Claude Code versions, the measured seconds of steps 8, 9 and 11, and each step as passed or failed with the observed text; this replaces 12.2 items 1 and 3.
  - [ ] A failed step is filed as a bug against the owning feature, fixed there, and rerun before this task closes.
  - [ ] The author's real config and boards are unchanged; the scratch folders are removed.

### E6-F1-T10 Success metrics made measurable

- Status: todo
- Depends on: E6-F1-T4, E6-F1-T7
- Covers: PRD 9 (SM-1, SM-2, SM-3, SM-4, SM-5); TECHSPEC 15 (E6-F1 DoD)
- Size: S
- Scope: Write `docs/metrics.md`: for each of SM-1 to SM-5, the PRD definition quoted, the instrument, how one sample is taken, the record (a results table with its columns) and the pass rule, using only what exists (A-E6-19). SM-1: a stopwatch from the first key of the herdr board key to the first character typed in the target pane, and a key count (A-E6-16), over 10 timed returns, median of each. SM-2 and SM-3: counts from the Claude Code transcripts under `~/.claude/projects/` (the author reads them; brain-swap never does, P3, T-09): per day, `/bs-park` invocations against `Park to which card?` questions, and over the two weeks, `/bs-back` invocations against replies starting `no note since` or `no note yet` (A-E6-17). SM-4: a yes or no noted right after each sampled return (A-E6-18). SM-5: rerun the E6-F1-T4 procedure on the build in use. Each instrument is tried once on data the release runs produced, and the exact search patterns that worked go into the file. No product code, flag or log event is added.
- Not in scope: the two weeks of measurement after v0.1 and their results (the author fills the tables later); the hyperfine procedure itself (E6-F1-T4).
- Procedure:
  1. Write `docs/metrics.md` with one `## SM-n` section per metric holding `Definition`, `Instrument`, `Sample`, `Record` and `Pass when` lines, and one empty results table per metric; the SM-5 table gets the release baseline from the E6-F1-T4 commit as its first row.
  2. SM-1 dry run: do one timed return on the author's real board with the installed key; record seconds and key presses.
  3. SM-2 and SM-3 dry run: in the transcripts of the E6-F1-T7 sessions, count `/bs-park` and `/bs-back` invocations, picker questions and catch-up replies with the patterns written in step 1; pass when the counts equal what the E6-F1-T7 commit message says happened (for example, the AS-3 picks), else correct the patterns and repeat.
  4. SM-4 dry run: take one sample after a real return and record it.
  5. Check that `tests/docs.rs` passes with the new file, then link `docs/metrics.md` from the README.
- DoD:
  - [ ] `docs/metrics.md` is committed with one section per SM, each naming instrument, sample, record and pass rule, and the README links it.
  - [ ] The commit message records the date and the dry-run values of steps 2 to 4; this replaces 12.2 items 1 and 3.
  - [ ] `git diff` shows no change under `src/`, `pack/` or `templates/` from this task.

## Assumptions

- A-E6-01 Is the E6 manual check a full rerun of AS-1 to AS-11 on the build installed from a fresh checkout, with the pack and key installed by `install claude` and `install herdr`, although E3-F1 (AS-1 steps 1, 3, 4, 8, AS-2, AS-3) and E5-F3 (AS-1 step 7, AS-4, AS-5, AS-10, AS-11) already run parts by hand during development, and may it reference the E3-F1-T8 to T10 procedures for the Claude steps instead of restating them? Affects E6-F1-T7, T8, T9.
- A-E6-02 Does the release wait for E0-F3, since the SP-8 fallback may change step 3 of `bs-back/SKILL.md`, although TECHSPEC 15 lists only "all Core features" and E0-F3 is a spike? Affects E6-F1 and E6-F1-T7.
- A-E6-03 Is the perf board, written by E4-F1-T10's `write_big_board(dir, 300, 20)` after E6-F1-T2 changes its layout in place (signature kept), this: `board.md` with `letter: W`, `columns: Todo, Doing, Done`, `next: 301`; W-1 to W-290 in Done, W-291 to W-295 in Todo, W-296 to W-300 in Doing; each card a Feature card with a title of about 40 characters, one line per field and 20 notes of three one-line parts of about 60 characters, a place line with `cwd=/home/perf/Work/proj-<n mod 5>`, `pane=w4V:p9`, `tab=w4V:t1`, `workspace=w4V`, `session=perf-<n>`, stamps one hour apart in card and note order ending before `BRAIN_SWAP_NOW`? Affects E6-F1-T2, T3, T4.
- A-E6-04 Is NFR-01's p95 the value at rank `ceil(0.95 * n)` of the sorted samples, over 20 runs after 3 warm-ups in the `#[ignore]` tests and 50 runs after 3 warm-ups in `hyperfine` (whose summary has no p95, so it is computed from `--export-json`)? Affects E6-F1-T2, T3, T4.
- A-E6-05 May the `#[ignore]` write tests let the board grow during their 23 runs (up to 23 extra cards or notes), while the `hyperfine` write runs restore the pristine board before each sample with `--prepare`? Affects E6-F1-T2, T4.
- A-E6-06 Is NFR-02's first frame measured as the in-process start path to the first `TestBackend` draw (E4-F1-T10's `nfr02_first_frame_within_150_ms`, rerun in E6-F1-T3), plus the process start-up floor measured with `hyperfine` on `brain-swap templates`, since neither `hyperfine` nor any section 13 crate can time a real terminal's first frame? Affects E6-F1-T3, T4.
- A-E6-07 Is a key reaction the time from `update` receiving the key to the end of the next draw, including the store write and reload for `L`, with `App` built by E4-F1's `tui::prepare` and drawn by `tui::view_frame::draw`, and do E4-F1-T10's Board navigation (p95) and first frame (median of 5) tests stand as the NFR-02 measure for those two, with no second version in `tests/perf.rs`? Affects E6-F1-T3, T4.
- A-E6-08 Is "a jump adds at most 50 ms to its herdr calls" checked on `brain-swap jump` (not the TUI) as wall time minus the sum of the `herdr` call durations in the `BRAIN_SWAP_LOG` file, against the fake herdr script? Affects E6-F1-T3.
- A-E6-09 Is "idle, one directory scan per second" checked by counting `openat` calls on the board folder with `strace` over 10 idle seconds of the TUI, passing at 9 to 11? Affects E6-F1-T4.
- A-E6-10 Is an automated parity test in E6-F1 the right check for NFR-05, alongside E5-F2's `locate` and `jump` tests outside herdr, comparing output after masking the pane suffix and the best guess labels, and comparing `ls --open --guess` only as a set of IDs? Affects E6-F1-T5.
- A-E6-11 Do the manual release runs use scratch `XDG_*` folders and a temporary herdr popup binding whose command prefixes `env XDG_CONFIG_HOME=... XDG_DATA_HOME=... XDG_STATE_HOME=...`, since a popup inherits the herdr server's environment and not the pane's, and is AS-11 reproduced with `kill -STOP` on the herdr server while the TUI runs in a terminal outside herdr that carries a herdr pane's `HERDR_*` variables (a stopped server also freezes any popup inside herdr)? Affects E6-F1-T7, T8, T9.
- A-E6-12 Is AS-10's pane move done with whatever herdr 0.8.2 offers (its UI if no CLI command exists), recorded in the commit message? Affects E6-F1-T8.
- A-E6-13 Is "a fresh checkout" a `git clone` of the release commit into a new temp folder, installed with `cargo install --locked` (so `Cargo.lock` is committed), and does a clean `ldd` list (only C runtime libraries) count as "nothing else is needed at runtime"? Affects E6-F1-T6.
- A-E6-14 Does the README name only `cargo install --path .` until a public repository URL exists, adding the `--git <url>` form of TECHSPEC 13 then, and are its headings exactly `## Install`, `## Claude Code pack`, `## herdr key binding` (E5-F4-T3's) and `## Upgrade`? Affects E6-F1-T1.
- A-E6-15 Is AS-11's "within 3 seconds" checked with a stopwatch in the TUI and with `time` on `brain-swap jump`, allowing 0.5 seconds for process start and output (3.5 seconds real time)? Affects E6-F1-T8.
- A-E6-16 Does SM-1 count the herdr board key binding (`prefix+alt+b`, typed as `ctrl+a` then `alt+b`) as one key press, so the target of 4 leaves three presses for selection and Enter? Affects E6-F1-T10.
- A-E6-17 Are SM-2 and SM-3 counted by the author from Claude Code's own transcripts (`/bs-park` and `/bs-back` invocations, `Park to which card?` questions, replies starting `no note since` or `no note yet`), which brain-swap itself never reads (P3, T-09), rather than by a new log event or counter in the product? Affects E6-F1-T10.
- A-E6-18 Is SM-4's sample the first return of each working day until 10 are taken, judged by the author right after it (yes when the first prompt after reading the note continued the task without scrolling the transcript)? Affects E6-F1-T10.
- A-E6-19 Do the metric definitions and empty results tables live in `docs/metrics.md`, filled by the author during the two weeks after v0.1, outside this backlog? Affects E6-F1-T10.
- A-E6-20 Is the board for `hyperfine` written by the `#[ignore]` test `sm5_write_hyperfine_board` to `env!("CARGO_TARGET_TMPDIR")/perf-board/` (compile-time path, so the test reads no environment), with an `env.sh` the author sources? Affects E6-F1-T2, T4.
- A-E6-21 Is NFR-09's "macOS should build" checked at release by `cargo check --target aarch64-apple-darwin` on the release clone, plus `cargo test --all-features` on a macOS machine only when one is available, its result recorded in the commit message, since TECHSPEC 12.2 item 2 requires the tests on Linux only? Affects E6-F1-T6.
