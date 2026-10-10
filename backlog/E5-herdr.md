# E5 herdr adapter

Milestone: M1 (E5-F1), M4 (E5-F2 to E5-F4). This epic delivers the optional herdr adapter under `src/adapters/`: the `Runner` through which every herdr command runs (no shell, a 2 second kill timeout, one log line per call), the `FakeRunner` and the fake herdr script that every herdr test uses, the place captured at park time and the current pane's agent session (9.1, 9.2), `locate` and `focus` with their shared 3 second budget (9.3) behind the `locate` and `jump` subcommands, the jump from the TUI's board and detail view (7.7), and `install herdr` (9.4). E5-F1 sits in M1 because `park` and session resolution (E2-F3) call `capture` and `pane_session`; E5-F2 to E5-F4 sit in M4, after the TUI, because principle 1 ships the cue before the board and the jump is the board's way back to a pane (TECHSPEC 15). Outside herdr no herdr command runs and only the jump differs (FR-58, NFR-05). Spec: TECHSPEC 2.2, 3 (`LivePane`, `PaneStatus`), 6.2 (exit 6), 6.3 (`locate`, `jump`, `install herdr`), 6.4 (step 3), 6.7 (`locate`, `jump`, `install herdr` objects), 7.1, 7.7, 9, 11 (herdr log), 12.1 (herdr bullet), 14 (SP-2, SP-4, SP-5, SP-7) and the E5 entries of 15.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E5-F1 | Runner, capture, pane session | E1-F1, E0-F1 | 9 | L |
| E5-F2 | Locate and jump | E5-F1, E2-F1, E1-F5, E0-F1, E0-F2 | 9 | L |
| E5-F3 | TUI jump | E5-F2, E4-F2, E4-F3, E4-F4, E2-F3 | 7 | L |
| E5-F4 | install herdr | E2-F1, E0-F2 | 3 | S |

## E5-F1 Runner, capture, pane session

- Depends on: E1-F1, E0-F1
- Covers: FR-15, FR-41, FR-58
- Spec: TECHSPEC 2.2, 6.4 (step 3), 9.1, 9.2, 9.3 (call timeout, `LivePane.agent` rule, tolerant JSON), 9.5, 11 (herdr calls in the log), 12.1 (herdr bullet, hermetic tests), 14 (SP-4), T-15
- Scope: The one process boundary of the adapter: a `Runner` trait whose real implementation starts the herdr binary (`HERDR_BIN_PATH`, else `herdr` on `PATH`) without a shell, kills a call that outlives its timeout, and appends argv, exit and milliseconds to the `BRAIN_SWAP_LOG` file. A `FakeRunner` scripts answers, hangs and call durations for in-process tests, and the fake herdr script answers canned JSON per `FAKE_HERDR_SCENARIO` and logs argv for process-level tests. On top of that, the tolerant readers of herdr's pane JSON, `capture` (the herdr part of a note's place from `HERDR_*`, no herdr command, FR-15) and `pane_session` (the current pane's `agent_session.value`, 6.4 step 3), both inert outside herdr (FR-58). The SP-4 moved-pane check of 9.1 lands only if the spike enabled it.
- Provides: `adapters::runner::Runner` (object safe: `fn run(&self, args: &[&str], timeout: Duration) -> RunResult`), `adapters::runner::RunResult { outcome: RunOutcome, took: Duration }` with `RunResult::succeeded(&self) -> bool`, `adapters::runner::RunOutcome` (`Exited { code: Option<i32>, stdout: String, stderr: String }`, `TimedOut`, `SpawnFailed(String)`), `adapters::runner::ProcessRunner` (no lifetime parameter; `ProcessRunner::new(env: &Env) -> ProcessRunner`, owning a clone of the `Env`; `program(&self) -> &Path`; the runner E2-F3, E5-F2 and E5-F3 pass in production, E5-F3 boxed as `Box<dyn Runner>` in the TUI's `Parts`), `adapters::runner::FakeRunner` (`FakeRunner::new()`, `on(self, args: &str, answer: Answer) -> Self`, `calls(&self) -> Vec<String>`, `timeouts(&self) -> Vec<Duration>`) and `adapters::runner::Answer` (`Exit { code: i32, stdout: String, stderr: String, took: Duration }`, `Hang`, `SpawnFailed(String)`; `Answer::ok(stdout: &str)`, `Answer::fail(code: i32, stderr: &str)`, `Answer::taking(self, Duration) -> Answer`), `adapters::herdr::CALL_TIMEOUT` (2 s), `adapters::herdr::PaneJson<'a>` (`PaneJson::from_response(&'a Value) -> Option<PaneJson<'a>>`, `PaneJson::new(&'a Value)`, `id()`, `tab()`, `workspace()`, `session()` each `-> Option<&str>`, `has_agent() -> bool`), `adapters::herdr::error_code(stderr: &str) -> Option<String>`, `adapters::herdr::capture(env: &Env, runner: &dyn Runner) -> Option<HerdrPlace>` (A-E5-02), `adapters::herdr::pane_session(env: &Env, runner: &dyn Runner) -> Option<String>`; `tests/fake_herdr/herdr` (replacing the E1-F1-T9 stub) with scenario files `tests/fake_herdr/scenarios/<name>.sh` and the scenarios `pane-session`, `pane-no-agent` and `hang` (E2-F3 relies on them; it no longer adds scenarios), the argv log `$HOME/fake-herdr.log`, `tests/common/fake_herdr.rs` (`calls(home: &Path) -> Vec<String>`, `scenarios() -> Vec<String>`); fixtures `tests/fixtures/herdr/pane_agent.json`, `tests/fixtures/herdr/pane_shell.json`, `tests/fixtures/herdr/pane_not_found.stderr`
- Requires: E1-F1 `core::env::Env` (`Clone`, `herdr_active()`, `herdr_env`, `herdr_bin_path`, `herdr_pane_id`, `herdr_tab_id`, `herdr_workspace_id`, `log`, `now`), `core::log::event`, `tests/common/env.rs` `core_env`, `tests/common/spawn.rs` (`Scenario` with its `herdr` and `fake_scenario` setters, `command`, `FAKE_HERDR`) and the `tests/fake_herdr/herdr` stub; E1-F1-T3 `core::model::HerdrPlace`; E0-F1 the TECHSPEC 16 `SP-4` answer with its JSON samples of `herdr pane get` (agent pane, shell pane, `pane_not_found` stderr), `herdr workspace list` and `herdr pane list --workspace <id>` (E0-F1-T3)
- Feature DoD:
  - [ ] `ProcessRunner` starts `HERDR_BIN_PATH`, else `herdr`, with plain arguments and returns exit code, stdout and stderr (E5-F1-T1).
  - [ ] A hung child is killed at 2 s and reaped (E5-F1-T2).
  - [ ] Every Runner call is appended to the log file with argv, exit and ms (E5-F1-T3).
  - [ ] `FakeRunner` scripts answers, hangs and durations and records argv and timeouts without sleeping (E5-F1-T4).
  - [ ] The fake herdr script answers canned JSON per `FAKE_HERDR_SCENARIO` and logs argv (E5-F1-T5).
  - [ ] herdr's pane JSON is read with tolerant `serde_json::Value` lookups that match the SP-4 samples (E5-F1-T6).
  - [ ] No capture outside herdr; inside herdr the place comes from `HERDR_*` with no herdr call (E5-F1-T7).
  - [ ] `pane_session` returns the current pane's `agent_session.value` or none, never failing (E5-F1-T8).
  - [ ] The 9.1 moved-pane check is in place, or the task is closed as not needed, per SP-4 (E5-F1-T9).

### E5-F1-T1 Runner trait and process runner

- Status: doing
- Depends on: E1-F1-T6
- Covers: TECHSPEC 2.2 (herdr only through its CLI, never the socket), 9.1 (the binary), T-15
- Size: S
- Scope: Create `src/adapters/runner.rs` with `pub trait Runner { fn run(&self, args: &[&str], timeout: Duration) -> RunResult; }` (object safe, so every caller takes `&dyn Runner`), `pub struct RunResult { pub outcome: RunOutcome, pub took: Duration }` with `succeeded()` (exited with code 0), and `pub enum RunOutcome { Exited { code: Option<i32>, stdout: String, stderr: String }, TimedOut, SpawnFailed(String) }`. `ProcessRunner { env: Env, program: PathBuf }` with no lifetime parameter, so it can sit in the TUI's `Parts` beside the `Runtime` that owns the original `Env` (E5-F3-T3): `ProcessRunner::new(env: &Env) -> ProcessRunner` stores `env.clone()` and takes `env.herdr_bin_path` when set and non-empty, else the bare name `herdr`, which the OS finds on `PATH` (9.1); `run` starts the program with each element of `args` as its own argument (no shell), stdin null, stdout and stderr piped and drained on two threads so a full pipe cannot stall the child, waits for it and returns `Exited` with both streams decoded lossily as UTF-8 and `took` measured with `std::time::Instant`; a spawn error is `SpawnFailed(<os error text>)`. The `timeout` argument is enforced from E5-F1-T2 on. Tests live in `tests/runner.rs`: each writes a small `sh` script into its temp dir, marks it executable and names it through `herdr_bin_path` of a `core_env` value; those scripts read no environment variable, which keeps them hermetic in the 12.1 sense although the library, not the spawn helper, starts them (A-E5-06).
- Not in scope: the timeout and the kill (E5-F1-T2); the log line (E5-F1-T3); the test double (E5-F1-T4); which herdr subcommands run (E5-F1-T7, E5-F1-T8, E5-F2).
- Tests first:
  1. `ts9_1_runner_returns_exit_code_and_output` (runner, real process): given a script `printf out; printf err >&2; exit 3` as `herdr_bin_path`, when `ProcessRunner::new(&env).run(&["pane", "get", "w4V:p9"], Duration::from_secs(2))` runs, then the outcome is `Exited { code: Some(3), stdout: "out", stderr: "err" }` and `succeeded()` is false.
  2. `ts9_1_exit_0_succeeds` (runner, real process): given a script `printf '{"result":{}}'`, when it runs, then `succeeded()` is true and stdout is `{"result":{}}`.
  3. `ts9_1_runner_passes_arguments_without_shell` (runner, real process): given a script `for a; do printf '%s\n' "$a"; done`, when it runs with `["pane", "get", "w 4V;$x"]`, then stdout is the three lines `pane`, `get` and `w 4V;$x`.
  4. `ts9_1_program_is_herdr_bin_path_else_herdr` (unit): given `herdr_bin_path` `Some("/opt/h/herdr")`, then `program()` is `/opt/h/herdr`; given `None`, and given `Some("")`, then `program()` is `herdr` for both.
  5. `ts9_1_missing_binary_is_spawn_failed` (runner): given `herdr_bin_path` `<tmp>/nope`, which does not exist, when a call runs, then the outcome is `SpawnFailed(_)` and `succeeded()` is false.
  6. `ts9_1_child_stdin_is_null` (runner, real process): given a script `cat >/dev/null; printf done`, when it runs with a 2 s timeout, then it returns `Exited { code: Some(0), stdout: "done", .. }` with `took` under 1 s (stdin is null, so `cat` reads end of input at once).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `Command::new` appears under `src/adapters/` only in `runner.rs`, no `sh -c` is built there, and `tests/layering.rs` (E1-F1) passes.

### E5-F1-T2 Call timeout kills the child

- Status: todo
- Depends on: E5-F1-T1, E1-F1-T9
- Covers: TECHSPEC 9.3 (a timeout that kills the child), T-15 (2 seconds)
- Size: S
- Scope: `ProcessRunner::run` polls `try_wait` every 10 ms until `timeout`; on expiry it kills the child, waits for it so no zombie is left, and returns `TimedOut` with `took` the elapsed time. It does not join the reader threads after a timeout, so a grandchild that still holds a pipe cannot keep `run` waiting. Add `pub const CALL_TIMEOUT: Duration` of 2 seconds to `src/adapters/herdr.rs` (T-15): the timeout every herdr call asks for.
- Not in scope: the 3 second budget and `min(2 s, rest)` (E5-F2-T4); mapping `TimedOut` to `NotResponding` (E5-F2-T2, E5-F2-T5).
- Tests first:
  1. `ts9_3_hung_child_is_killed_at_2s` (runner, real process; waits the 2 s the spec demands): given a script that writes `$$` to `<tmp>/pid` and then runs `exec sleep 30`, when it runs with `CALL_TIMEOUT`, then the outcome is `TimedOut`, `took` lies between 2.0 s and 2.5 s, and `kill -0 <pid>` started through the 12.1 spawn helper (`command(&scenario, Path::new("/bin/sh"))` with `-c`) exits non-zero.
  2. `ts9_3_shorter_timeout_is_honoured` (runner, real process): given the same script, when it runs with a 200 ms timeout, then the outcome is `TimedOut` and `took` lies between 200 ms and 700 ms.
  3. `ts9_3_quick_child_is_not_cut` (runner, real process): given a script that exits 0 at once, when it runs with `CALL_TIMEOUT`, then the outcome is `Exited { code: Some(0), .. }` and `took` is under 1 s.
  4. `ts9_3_grandchild_holding_pipe_does_not_block` (runner, real process): given a script that starts `sleep 30 &`, writes that background PID to `<tmp>/bg` and then runs `exec sleep 30`, when it runs with a 200 ms timeout, then `run` returns `TimedOut` within 700 ms; the test then kills the background `sleep` by its PID through the spawn helper.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `CALL_TIMEOUT` is the only 2 second duration under `src/adapters/` (grep).

### E5-F1-T3 herdr calls in the log

- Status: todo
- Depends on: E5-F1-T2, E1-F1-T8
- Covers: TECHSPEC 11 (herdr calls: argv, exit, ms), 6.1 (`BRAIN_SWAP_LOG`)
- Size: S
- Scope: After every call, once its `RunResult` is fixed, `ProcessRunner` calls `core::log::event(&self.env, "herdr", text)` with `text` = `<args joined by single spaces> exit=<code> ms=<took in whole milliseconds>`, where `<code>` is `timeout` for `TimedOut`, `spawn_failed` for `SpawnFailed` and `signal` for an exit without a code (A-E5-07). The program path is not logged. `core::log::event` (E1-F1-T8) already writes nothing without `BRAIN_SWAP_LOG` and ignores write errors, so logging never changes a result.
- Not in scope: the log sink and its board-folder guard (E1-F1-T8); the CLI test that a `park` inside herdr logs its `pane get` (E2-F3-T4).
- Tests first:
  1. `ts11_runner_logs_argv_exit_and_ms` (runner, real process): given `env.log` `<tmp>/bs.log`, `env.now` `2026-10-08T11:52:00+03:00` and a script that exits 3, when `run(&["pane", "get", "w4V:p9"], CALL_TIMEOUT)` runs, then the log holds exactly one line; it starts with `2026-10-08T11:52:00+03:00 herdr pane get w4V:p9 exit=3 ms=` and the rest parses as `u64`.
  2. `ts11_timed_out_call_is_logged` (runner, real process): given a script running `exec sleep 30` and a 200 ms timeout, when it runs, then the only log line ends `exit=timeout ms=<n>` with `n` at least 200.
  3. `ts11_spawn_failure_is_logged` (runner): given `herdr_bin_path` `<tmp>/nope`, when `run(&["workspace", "list"], CALL_TIMEOUT)` runs, then the only log line contains `herdr workspace list exit=spawn_failed ms=`.
  4. `ts11_each_call_adds_one_line` (runner, real process): given two calls `pane get w4V:p9` and `workspace list`, then the log holds two lines in call order.
  5. `ts11_no_log_without_brain_swap_log` (runner, real process): given `env.log` `None`, when a call runs, then the temp dir holds no file but the script.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] With `env.log` naming a directory, test 1's call still returns `Exited { code: Some(3), .. }` (one extra assertion).

### E5-F1-T4 FakeRunner

- Status: todo
- Depends on: E5-F1-T1
- Covers: TECHSPEC 12.1 (herdr: `FakeRunner`)
- Size: S
- Scope: `pub struct FakeRunner` in `src/adapters/runner.rs`, compiled in every build so that tests under `tests/` and the TUI tests can use it (A-E5-05). `FakeRunner::new()`; `on(self, args, answer)` registers an answer for the argv joined by single spaces, reused for repeated calls. `run` records the joined argv and the timeout it was given, then returns: for `Answer::Exit` whose `took` is at most the timeout, `Exited` with that `took` (zero unless `taking` set it); for `Answer::Hang`, or an `Exit` taking longer than the timeout, `TimedOut` with `took` equal to the timeout; for `Answer::SpawnFailed(msg)`, `SpawnFailed(msg)` with zero `took`. It never sleeps. An argv without an answer panics with `FakeRunner: no answer for '<argv>'`, so an unplanned herdr call fails its test. `Answer::ok(stdout)` is exit 0 with empty stderr; `Answer::fail(code, stderr)` has empty stdout. `calls()` and `timeouts()` return what was recorded, in order.
- Not in scope: herdr answers and their fixtures (E5-F1-T6, E5-F2-T3); the budget (E5-F2-T4).
- Tests first:
  1. `ts12_1_fake_runner_answers_and_records_argv` (FakeRunner): given `on("pane get w4V:p9", Answer::ok("{}"))`, when `run(&["pane", "get", "w4V:p9"], 2 s)` runs twice, then both outcomes are `Exited { code: Some(0), stdout: "{}", .. }` and `calls()` is `["pane get w4V:p9", "pane get w4V:p9"]`.
  2. `ts12_1_fake_runner_records_timeouts` (FakeRunner): given the answer of test 1, when it is called with 2 s and then with 500 ms, then `timeouts()` is `[2 s, 500 ms]`.
  3. `ts12_1_fake_runner_hang_times_out_without_waiting` (FakeRunner): given `Answer::Hang` and a 2 s timeout, when it runs, then the outcome is `TimedOut` with `took` 2 s and the test's wall time is under 100 ms.
  4. `ts12_1_fake_runner_slow_answer_beyond_timeout_times_out` (FakeRunner): given `Answer::ok("{}").taking(2.5 s)`, then a call with 2 s is `TimedOut` with `took` 2 s; given `.taking(1.5 s)`, then it is `Exited` with `took` 1.5 s.
  5. `ts12_1_fake_runner_fail_and_spawn_failed` (FakeRunner): given `Answer::fail(1, "boom")`, then the outcome is `Exited { code: Some(1), stdout: "", stderr: "boom" }`; given `Answer::SpawnFailed("no such file".into())`, then it is `SpawnFailed("no such file")`.
  6. `ts12_1_fake_runner_unplanned_call_panics` (FakeRunner): given no answers, when `run(&["tab", "focus", "x"], 2 s)` runs, then it panics with a message containing `no answer for 'tab focus x'`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `FakeRunner` contains no `thread::sleep` and no `Command` (grep).

### E5-F1-T5 Fake herdr script

- Status: todo
- Depends on: E1-F1-T9, E0-F1-T3
- Covers: TECHSPEC 12.1 (herdr: fake script, `FAKE_HERDR_SCENARIO`, argv log)
- Size: M
- Scope: Replace the E1-F1-T9 stub `tests/fake_herdr/herdr` with a POSIX `sh` script (git mode 100755). It first appends `"$*"` as one line to `$HOME/fake-herdr.log` (A-E5-03). With `FAKE_HERDR_SCENARIO` unset or empty it prints `fake herdr: no scenario` on stderr and exits 1; with a name for which `scenarios/<name>.sh` (beside the script, through `dirname "$0"`) does not exist it prints `fake herdr: unknown scenario <name>` and exits 1. Otherwise it defines `out <json>` (print to stdout, exit 0), `fail <code> <json>` (print to stderr, exit with that code), `hang` (`exec sleep 30`, so killing the call closes every pipe) and `slow <seconds>` (sleep, then continue), and sources the scenario: a `case "$*" in ... esac` over the joined argv. A call the scenario does not match prints `fake herdr: unexpected call: <argv>` on stderr and exits 1 (A-E5-04). Scenario JSON has the shape of the TECHSPEC 16 `SP-4` samples (key names as recorded there, A-E5-10) with the 4.9 IDs. Scenarios: `pane-session` (`pane get w4V:p9` answers a Claude pane with tab `w4V:t1`, workspace `w4V`, `agent_status` `idle` and `agent_session.value` `herdr-s1`, as E2-F3 expects); `pane-no-agent` (`pane get w4V:p9` answers a plain-shell pane, `agent_status` `unknown`, no `agent` or `agent_session` key); `hang` (every call hangs). Add `tests/common/fake_herdr.rs` with `calls(home: &Path) -> Vec<String>` (the log's lines, empty when there is no log) and `scenarios() -> Vec<String>` (the names under `scenarios/`).
- Not in scope: the `locate` and `jump` scenarios (E5-F2-T6 to E5-F2-T9), the AS-6 scenario (E5-F3-T5) and the pack scenario `pack_two_panes` (E3-F1-T7); reading the JSON in Rust (E5-F1-T6).
- Tests first:
  1. `ts12_1_fake_herdr_logs_each_call` (fake herdr script, through `command(&scenario, FAKE_HERDR)`): given scenario `pane-session`, when the script runs `pane get w4V:p9` twice, then `calls(home)` is `["pane get w4V:p9", "pane get w4V:p9"]`.
  2. `ts12_1_fake_herdr_answers_scenario_json` (fake herdr script): given `pane-session`, when it runs `pane get w4V:p9`, then it exits 0 and stdout parses as JSON whose `result.pane.agent_session.value` is `herdr-s1`.
  3. `ts12_1_fake_herdr_pane_without_agent` (fake herdr script): given `pane-no-agent`, when it runs `pane get w4V:p9`, then stdout parses, `result.pane` has no `agent_session` key and its `agent_status` is `unknown`.
  4. `ts12_1_fake_herdr_unexpected_call_fails` (fake herdr script): given `pane-session`, when it runs `pane get w4V:p1`, then it exits 1, stderr is `fake herdr: unexpected call: pane get w4V:p1` and the call is in `calls(home)`.
  5. `ts12_1_fake_herdr_without_scenario_fails` (fake herdr script): given no `FAKE_HERDR_SCENARIO`, when it runs `workspace list`, then it exits 1 with stderr `fake herdr: no scenario` and the call is logged.
  6. `ts12_1_fake_herdr_unknown_scenario_fails` (fake herdr script): given `FAKE_HERDR_SCENARIO=nope`, when it runs `workspace list`, then it exits 1 with stderr `fake herdr: unknown scenario nope`.
  7. `ts12_1_fake_herdr_hang_keeps_running` (fake herdr script): given `hang`, when `pane get w4V:p9` is spawned, then after 300 ms the process has not exited and has printed nothing; the test then kills it.
  8. `ts12_1_every_scenario_parses_as_sh` (fake herdr script): given every name from `scenarios()`, when `sh -n tests/fake_herdr/scenarios/<name>.sh` runs through the spawn helper, then each exits 0.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The script writes no file but `$HOME/fake-herdr.log` (test 1 compares the temp root's file list before and after), and `git ls-files -s tests/fake_herdr/herdr` shows mode 100755.

### E5-F1-T6 Pane JSON reader

- Status: todo
- Depends on: E1-F1-T1, E0-F1-T3
- Covers: FR-41; TECHSPEC 9.2 (`result.pane.agent_session.value`), 9.3 (`LivePane.agent` rule, `pane_not_found`, tolerant `serde_json::Value` lookups)
- Size: S
- Scope: In `src/adapters/herdr.rs`, readers over `serde_json::Value` with no serde-derived herdr types (9.3): `PaneJson<'a>` with `from_response(&Value)` (the object at `result.pane`, else `None`), `new(&Value)` (a pane object taken from a list), `id()`, `tab()` and `workspace()` (non-empty strings at the key names SP-4 recorded, A-E5-10), `session()` (`agent_session.value`, non-empty) and `has_agent()` (true when the object has an `agent` or an `agent_session` key, or `agent_status` is a string other than `unknown`; a missing `agent_status` counts as `unknown`, A-E5-10); `error_code(stderr)` parses stderr as one JSON value and returns its `error.code`, else `None`. Fixtures copied from the SP-4 samples with the IDs rewritten to the 4.9 ones: `tests/fixtures/herdr/pane_agent.json` (pane `w4V:p9`, tab `w4V:t1`, workspace `w4V`, `agent_status` `idle`, `agent_session.value` `55583127-a22a-49bc-a803-e777c377a595`), `tests/fixtures/herdr/pane_shell.json` (pane `w4V:p3`, tab `w4V:t2`, workspace `w4V`, `agent_status` `unknown`) and `tests/fixtures/herdr/pane_not_found.stderr`.
- Not in scope: the list JSON (E5-F2-T3); building `LivePane` (E5-F2-T1).
- Tests first:
  1. `fr41_pane_json_reads_agent_session_value` (unit, fixtures through `include_str!`): given `pane_agent.json`, when `PaneJson::from_response` reads it, then `session()` is `Some("55583127-a22a-49bc-a803-e777c377a595")`.
  2. `ts9_2_shell_pane_has_no_session` (unit): given `pane_shell.json`, then `session()` is `None`.
  3. `ts9_2_empty_session_value_is_none` (unit): given `pane_agent.json` with `agent_session.value` set to `""`, then `session()` is `None`.
  4. `ts9_3_pane_json_reads_ids` (unit): given `pane_agent.json`, then `id()`, `tab()` and `workspace()` are `w4V:p9`, `w4V:t1` and `w4V`.
  5. `ts9_3_from_response_needs_result_pane` (unit): given `{"result":{}}`, and given `[]`, then `from_response` is `None` for both.
  6. `ts9_3_agent_session_key_means_agent` (unit): given `pane_agent.json` with `agent_status` changed to `unknown`, then `has_agent()` is true.
  7. `ts9_3_agent_key_means_agent` (unit): given `pane_shell.json` plus the key `agent` with value `"claude"`, then `has_agent()` is true.
  8. `ts9_3_known_agent_status_means_agent` (unit): given `pane_shell.json` with `agent_status` `working`, then `has_agent()` is true.
  9. `ts9_3_unknown_or_missing_status_means_no_agent` (unit): given `pane_shell.json`, and the same without its `agent_status` key, then `has_agent()` is false for both.
  10. `ts9_3_error_code_reads_pane_not_found` (unit): given `pane_not_found.stderr`, then `error_code` is `Some("pane_not_found")`.
  11. `ts9_3_error_code_none_for_plain_text` (unit): given `herdr: cannot connect`, and the empty string, then `error_code` is `None` for both.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The fixture files equal the SP-4 samples of TECHSPEC 16 except for the rewritten IDs (diffed in review), and `src/adapters/herdr.rs` has no `#[derive(Deserialize)]`.

### E5-F1-T7 Capture at park time

- Status: todo
- Depends on: E5-F1-T4, E1-F1-T3
- Covers: FR-15, FR-58; TECHSPEC 9.1 (capture), 9.5 (no herdr command outside herdr), T-15
- Size: S
- Scope: `adapters::herdr::capture(env: &Env, runner: &dyn Runner) -> Option<HerdrPlace>` (A-E5-02). Outside herdr (`env.herdr_active()` false) it returns `None`. Inside, it returns `HerdrPlace { pane: HERDR_PANE_ID, tab: HERDR_TAB_ID, workspace: HERDR_WORKSPACE_ID }`, an empty tab or workspace stored as absent, and `None` when `HERDR_PANE_ID` is unset or empty (A-E5-09). It runs no herdr command; the runner stays unused unless E5-F1-T9 enables the SP-4 check.
- Not in scope: writing the place into a note, the cwd and the session part of the place (E2-F3-T3); the moved-pane check (E5-F1-T9).
- Tests first:
  1. `fr15_capture_builds_place_from_herdr_variables` (FakeRunner without answers, so any call panics; `Env` from `core_env` with its `HERDR_*` fields set): given `herdr_env` `1`, pane `w4V:p9`, tab `w4V:t1` and workspace `w4V`, when `capture` runs, then it returns `HerdrPlace { pane: "w4V:p9", tab: Some("w4V:t1"), workspace: Some("w4V") }` and `calls()` is empty.
  2. `fr58_no_capture_outside_herdr` (FakeRunner): given the same pane, tab and workspace with `herdr_env` `None`, when `capture` runs, then it returns `None`.
  3. `fr58_herdr_env_must_be_1` (FakeRunner): given `herdr_env` `0`, and given `true`, then `capture` returns `None` for both.
  4. `ts9_1_empty_tab_and_missing_workspace_are_absent` (FakeRunner): given `herdr_env` `1`, pane `w4V:p9`, tab `""` and no workspace, then the place has pane `w4V:p9`, `tab` `None` and `workspace` `None`.
  5. `ts9_1_missing_pane_id_captures_nothing` (FakeRunner): given `herdr_env` `1`, no pane and tab `w4V:t1`, then `capture` returns `None`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Under `src/adapters/`, `herdr_pane_id`, `herdr_tab_id` and `herdr_workspace_id` are read only in `capture` and `pane_session` (grep).

### E5-F1-T8 Agent session of the current pane

- Status: todo
- Depends on: E5-F1-T2, E5-F1-T4, E5-F1-T6
- Covers: FR-41, FR-58; TECHSPEC 6.4 (step 3), 9.2, T-15
- Size: S
- Scope: `adapters::herdr::pane_session(env: &Env, runner: &dyn Runner) -> Option<String>`. Outside herdr, or without `HERDR_PANE_ID`, it returns `None` without a call. Otherwise it makes one call, `pane get <HERDR_PANE_ID>` with `CALL_TIMEOUT`; an exit 0 gives `PaneJson::from_response(..)` and its `session()`; a non-zero exit (including `pane_not_found`), a timeout, a spawn failure or unreadable JSON give `None`. The value is returned as herdr reports it: the I8 check, the warnings and the decision when step 3 runs belong to `resolve_session` (E2-F3-T4, A-E5-08).
- Not in scope: the 6.4 chain and its warnings (E1-F7, E2-F3-T4); the moved-pane case (E5-F1-T9).
- Tests first:
  1. `fr41_pane_session_returns_agent_session_value` (FakeRunner; `herdr_env` `1` and pane `w4V:p9` unless a test says otherwise): given `pane get w4V:p9` answering `pane_agent.json`, when `pane_session` runs, then it returns `Some("55583127-a22a-49bc-a803-e777c377a595")`, `calls()` is `["pane get w4V:p9"]` and `timeouts()` is `[2 s]`.
  2. `fr41_pane_without_agent_returns_none` (FakeRunner): given the answer `pane_shell.json`, then it returns `None` after one call.
  3. `fr58_pane_session_outside_herdr_calls_nothing` (FakeRunner without answers): given `herdr_env` `None` and pane `w4V:p9`, then it returns `None` and `calls()` is empty.
  4. `ts9_2_missing_pane_id_calls_nothing` (FakeRunner without answers): given `herdr_env` `1` and no pane, then it returns `None` without a call.
  5. `ts9_2_pane_not_found_returns_none` (FakeRunner): given `Answer::fail(1, <pane_not_found.stderr>)`, then it returns `None`.
  6. `ts9_2_timeout_returns_none` (FakeRunner): given `Answer::Hang`, then it returns `None` after one call.
  7. `ts9_2_unreadable_answer_returns_none` (FakeRunner): given `Answer::ok("not json")`, then it returns `None`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every test asserts at most one herdr call per `pane_session`.

### E5-F1-T9 SP-4 moved-pane check

- Status: todo
- Depends on: E5-F1-T7, E5-F1-T8, E0-F1-T3
- Covers: FR-15, FR-41; TECHSPEC 9.1 (moved-pane check), 2.2 (`pane current` only if SP-4 confirms it), T-15
- Size: S
- Scope: Runs only if the TECHSPEC 16 `SP-4` line says `herdr pane current --current` resolves the caller's new pane after a move; otherwise the task is closed as `not needed: SP-4 fallback` with no code change, and notes from a moved pane keep the old ID for E5-F2-T3's moved search. When enabled, inside herdr: `capture` runs `pane get <HERDR_PANE_ID>`; when that reports `pane_not_found` (`error_code`) it runs `pane current --current` and builds the `HerdrPlace` from that pane's `id()`, `tab()` and `workspace()`; when the pane exists, or either call times out or fails otherwise, it keeps the place built from the variables. `pane_session`, on `pane_not_found`, returns the `session()` of `pane current --current`. Add the scenario `pane-moved-self` (`pane get w4V:p9` fails with `pane_not_found`; `pane current --current` answers pane `w7Q:p2`, tab `w7Q:t1`, workspace `w7Q`, session `herdr-s1`). E2-F3-T3 and E2-F3-T4 already allow the extra call (A-E5-11).
- Not in scope: the jump's moved search (E5-F2-T3), which finds moved panes by session either way.
- Tests first:
  1. `ts9_1_capture_uses_current_pane_after_move` (FakeRunner): given the variables of pane `w4V:p9`, `pane get w4V:p9` failing with `pane_not_found.stderr` and `pane current --current` answering pane `w7Q:p2`, tab `w7Q:t1` and workspace `w7Q`, when `capture` runs, then it returns that pane, tab and workspace and `calls()` is `["pane get w4V:p9", "pane current --current"]`.
  2. `ts9_1_capture_keeps_variables_when_pane_exists` (FakeRunner): given `pane get w4V:p9` answering `pane_agent.json`, then the place is built from the variables and `calls()` is `["pane get w4V:p9"]`.
  3. `ts9_1_capture_keeps_variables_on_timeout` (FakeRunner): given `pane get w4V:p9` answering `Answer::Hang`, then the place is built from the variables after one call.
  4. `ts9_1_pane_session_uses_current_pane_after_move` (FakeRunner): given the answers of test 1 with session `herdr-s1` on the current pane, then `pane_session` returns `Some("herdr-s1")`.
  5. `ts9_1_no_check_outside_herdr` (FakeRunner without answers): given `herdr_env` `None`, then `capture` returns `None` without a call.
- DoD:
  - [ ] Either the task is closed with the SP-4 line cited in the commit message, or the tests above pass and the first was committed red (12.2 item 1).
  - [ ] When enabled, TECHSPEC 2.2 still names `pane current` (as SP-4 left it), and no E2 test changes, since E2-F3-T3 and E2-F3-T4 already allow the extra call (A-E5-11).

## E5-F2 Locate and jump

- Depends on: E5-F1, E2-F1, E1-F5, E0-F1, E0-F2
- Covers: FR-40, FR-59, FR-60, FR-61, FR-62, FR-63, NFR-05
- Spec: TECHSPEC 3 (`LivePane`, `PaneStatus`), 6.2 (exit 6, no error object), 6.3 (`locate`, `jump`), 6.7 (`locate`, `jump` objects), 9.3, 9.5, 12.1 (herdr bullet), 14 (SP-4, SP-5), T-15, T-16
- Scope: The adapter's jump logic and its two subcommands. `locate` turns a note's place into a `PaneStatus`: outside herdr or without a pane it runs nothing; otherwise it asks `pane get`, treats existence first (`Live`, with live IDs winning over stored ones), and on `pane_not_found` searches every workspace's panes for the note's agent session (`Moved`), else `Closed`; a timeout is `NotResponding`, any other failure `HerdrError`. `focus` uses `agent focus` when an agent runs, else `workspace focus` plus `tab focus`. Every call asks for 2 seconds or the rest of a 3 second budget that `locate` and `focus` share. `brain-swap locate <ID> [--note <n>]` prints the status and `Where:` and exits 0; `brain-swap jump` locates, focuses and exits 0, or prints why not and exits 6.
- Provides: `adapters::herdr::LivePane { pane: String, tab: String, workspace: String, agent: bool }`, `adapters::herdr::PaneStatus` (`Live(LivePane)`, `Moved(LivePane)`, `Closed`, `OutsideHerdr`, `NoPane`, `NotResponding`, `HerdrError(String)`) with `PaneStatus::json_status(&self) -> &'static str`, `adapters::herdr::locate(place: &Place, env: &Env, runner: &dyn Runner) -> PaneStatus`, `adapters::herdr::Focused` (`Agent`, `Tab`), `adapters::herdr::FocusError` (`Failed`, `NotResponding`), `adapters::herdr::focus(p: &LivePane, runner: &dyn Runner) -> Result<Focused, FocusError>`, `adapters::herdr::Budget<'a>` (`Budget::new(runner: &'a dyn Runner, total: Duration)`, `left(&self) -> Duration`, implements `Runner`), `adapters::herdr::BUDGET` (3 s), `adapters::herdr::workspace_ids(&Value) -> Vec<String>`, `adapters::herdr::list_panes(&Value) -> Vec<PaneJson>`; the `Command::Locate` and `Command::Jump` handlers `cli::cmd::locate` and `cli::cmd::jump`, `cli::cmd::target_note(card: &Card, note: Option<u32>) -> Result<Option<usize>, Error>`, `cli::out::locate_lines(status: &PaneStatus, place: &Place) -> String`, `cli::out::LocateJson`, `cli::out::JumpJson`; fake herdr scenarios `live-agent`, `live-shell`, `moved`, `closed`, `herdr-error`, `slow-then-hang`, `focus-fails`, `focus-hangs`, `slow-live-focus-hangs`; fixtures `tests/fixtures/herdr/workspace_list.json`, `tests/fixtures/herdr/pane_list_w4V.json`, `tests/fixtures/herdr/pane_list_w7Q.json`, `tests/fixtures/jump/W-14.md`
- Requires: E5-F1 (`Runner`, `RunResult`, `ProcessRunner`, `FakeRunner`, `Answer`, `CALL_TIMEOUT`, `PaneJson`, `error_code`, the fake herdr script and `tests/common/fake_herdr.rs` `calls`, the `pane_*` fixtures); E2-F1 (`Command::Locate` and `Command::Jump` with `todo!()` arms, `cli::run` loading `Ctx` first, `Ctx`, `Out`, `Sandbox` with `copy_board`, `scenario_mut` and `redact`); E1-F5 (`core::board::find_card`, `Card::jump_note`, the `tests/fixtures/boards/work_4_9/` board) and through it E1-F2 (`Note`); E1-F1 (`Place`, `HerdrPlace`, `EXIT_JUMP_NOT_PERFORMED`, `Error::NotFound`, `core_env`); E0-F1 the SP-4 list JSON samples; E0-F2 the SP-5 focus answers (TECHSPEC 9.3 as updated)
- Feature DoD:
  - [ ] FakeRunner tests for `OutsideHerdr`, `NoPane` and `Live`, with live IDs over stored ones (E5-F2-T1).
  - [ ] FakeRunner tests for `Closed`, `NotResponding` and `HerdrError` (E5-F2-T2).
  - [ ] FakeRunner tests for `Moved` and for a hang in the moved search (`NotResponding`) (E5-F2-T3).
  - [ ] The 3 second budget: per-call timeouts of `min(2 s, rest)` asserted, an exhausted budget gives `NotResponding` (E5-F2-T4).
  - [ ] Every focus path (agent, tab, agent falling back to tab), each failing call and each timed-out call (E5-F2-T5).
  - [ ] `locate` prints every status with `Where:` and exits 0, argv sequences asserted through the fake script, and returns within 3 seconds when herdr hangs (E5-F2-T6).
  - [ ] A card with no note and an out-of-range `--note` (E5-F2-T7).
  - [ ] `jump` focuses the pane or its tab and reports a moved pane, exit 0 (E5-F2-T8).
  - [ ] A failing focus (exit 6), a timed-out focus (`not_responding`, exit 6) and every other status exit 6 without an error object (E5-F2-T9).

### E5-F2-T1 PaneStatus and locate: outside herdr, no pane, live

- Status: todo
- Depends on: E5-F1-T4, E5-F1-T6, E1-F1-T3
- Covers: FR-40, FR-62, NFR-05; TECHSPEC 3 (`LivePane`, `PaneStatus`), 9.3 (existence first, live IDs, `LivePane.agent`), 9.5, T-16
- Size: S
- Scope: Add `LivePane` and `PaneStatus` to `src/adapters/herdr.rs` exactly as section 3, with `PaneStatus::json_status()` giving `live`, `moved`, `closed`, `outside`, `none`, `not_responding` and `herdr_error` (6.7). `locate(place, env, runner)`: not `env.herdr_active()` gives `OutsideHerdr` and `place.herdr` absent gives `NoPane`, both without a call; else `pane get <pane>` with `CALL_TIMEOUT`, and an exit 0 gives `Live(LivePane)` built from the answer's pane: ID from the answer, tab and workspace from the answer, else the stored ones, else empty (A-E5-13), `agent` from `has_agent()` (T-16: live IDs win). An exit 0 whose pane cannot be read is `HerdrError("pane get: unreadable answer")` (A-E5-12). In these tests "the W-12 place" is the place of W-12's second note in 4.9 (cwd `/home/smilen/Work/ati.billing`, pane `w4V:p9`, tab `w4V:t1`, workspace `w4V`, session `55583127-a22a-49bc-a803-e777c377a595`).
- Not in scope: `Closed`, `NotResponding`, `HerdrError` on failures (E5-F2-T2); the moved search (E5-F2-T3); the budget (E5-F2-T4).
- Tests first:
  1. `fr62_outside_herdr_is_outside_without_call` (FakeRunner without answers): given `herdr_env` `None` and the W-12 place, when `locate` runs, then it returns `OutsideHerdr` and `calls()` is empty.
  2. `nfr05_pane_variables_alone_do_not_enable_herdr` (FakeRunner without answers): given `herdr_env` `None` but `herdr_pane_id` `w4V:p11`, then `locate` on the W-12 place returns `OutsideHerdr` without a call.
  3. `fr62_note_without_pane_is_no_pane_without_call` (FakeRunner without answers): given `herdr_env` `1` and a place holding only cwd `/home/smilen/Work/private/brain-swap`, then `locate` returns `NoPane` without a call.
  4. `fr40_live_pane_is_live` (FakeRunner): given `herdr_env` `1`, the W-12 place and `pane get w4V:p9` answering `pane_agent.json`, then `locate` returns `Live(LivePane { pane: "w4V:p9", tab: "w4V:t1", workspace: "w4V", agent: true })`, `calls()` is `["pane get w4V:p9"]` and `timeouts()` is `[2 s]`.
  5. `ts9_3_live_ids_win_over_stored` (FakeRunner): given the W-12 place (tab `w4V:t1`) and an answer like `pane_agent.json` with tab `w4V:t3`, then the `Live` pane has tab `w4V:t3`.
  6. `ts9_3_live_shell_pane_has_no_agent` (FakeRunner): given a place with pane `w4V:p3` and `pane get w4V:p3` answering `pane_shell.json`, then `locate` returns `Live` with tab `w4V:t2` and `agent: false`.
  7. `ts9_3_live_answer_without_tab_keeps_stored_tab` (FakeRunner): given the W-12 place and `pane_agent.json` without its tab key, then the `Live` pane has tab `w4V:t1`.
  8. `ts9_3_unreadable_live_answer_is_herdr_error` (FakeRunner): given `pane get w4V:p9` answering `Answer::ok("not json")`, then `locate` returns `HerdrError("pane get: unreadable answer")`.
  9. `ts6_7_status_json_names` (unit): given one value of each `PaneStatus` variant, then `json_status()` gives `live`, `moved`, `closed`, `outside`, `none`, `not_responding` and `herdr_error` in variant order.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `PaneStatus` has exactly the seven variants of section 3 (no `no_note`, which `cmd.rs` decides, 6.3).

### E5-F2-T2 locate: closed, not responding, herdr error

- Status: todo
- Depends on: E5-F2-T1
- Covers: FR-61, FR-63; TECHSPEC 9.3 (timeout, error code, no session)
- Size: S
- Scope: Extend `locate` for a `pane get` that does not exit 0: `TimedOut` gives `NotResponding`; `SpawnFailed(msg)` gives `HerdrError(msg)`; a non-zero exit whose stderr `error_code` is not `pane_not_found` gives `HerdrError` with the first non-empty stderr line verbatim, or `exit <code>` when stderr is empty (A-E5-12); `pane_not_found` with no `place.session` gives `Closed` without further calls (the moved search, which needs a session, is E5-F2-T3).
- Not in scope: the moved search (E5-F2-T3); the 3 second budget (E5-F2-T4).
- Tests first:
  1. `fr61_pane_not_found_without_session_is_closed` (FakeRunner): given the W-12 place without its session and `pane get w4V:p9` failing with `Answer::fail(1, <pane_not_found.stderr>)`, when `locate` runs, then it returns `Closed` and `calls()` is `["pane get w4V:p9"]`.
  2. `fr63_pane_get_timeout_is_not_responding` (FakeRunner): given `pane get w4V:p9` answering `Answer::Hang`, then `locate` returns `NotResponding` after one call.
  3. `ts9_3_other_error_is_herdr_error_with_first_line` (FakeRunner): given `Answer::fail(1, "{\"error\":{\"code\":\"internal\",\"message\":\"socket closed\"}}\ntrace")`, then `locate` returns `HerdrError("{\"error\":{\"code\":\"internal\",\"message\":\"socket closed\"}}")`.
  4. `ts9_3_plain_text_error_is_herdr_error` (FakeRunner): given `Answer::fail(2, "cannot connect to server\n")`, then `locate` returns `HerdrError("cannot connect to server")`.
  5. `ts9_3_empty_stderr_error_names_exit_code` (FakeRunner): given `Answer::fail(2, "")`, then `locate` returns `HerdrError("exit 2")`.
  6. `ts9_3_spawn_failure_is_herdr_error` (FakeRunner): given `Answer::SpawnFailed("No such file or directory (os error 2)".into())`, then `locate` returns `HerdrError("No such file or directory (os error 2)")`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Each test asserts `calls()`, so no failure path makes an extra herdr call.

### E5-F2-T3 locate: the moved search

- Status: todo
- Depends on: E5-F2-T2, E0-F1-T3
- Covers: FR-60, FR-61, FR-63; TECHSPEC 9.3 (moved search), A-20
- Size: M
- Scope: On `pane_not_found` with a `place.session`, `locate` runs `workspace list`, then for each workspace in listed order `pane list --workspace <id>`, and returns `Moved(LivePane)` for the first pane whose `session()` equals `place.session` (stop at the first match), its workspace taken from the pane, else the listed workspace ID (A-E5-13); no match gives `Closed`. A `TimedOut` list call gives `NotResponding`; a list call that fails otherwise (non-zero exit, spawn failure, unreadable JSON) ends the search as `Closed`. `workspace_ids(&Value)` and `list_panes(&Value)` read the list JSON tolerantly at the SP-4 key names (A-E5-10). Fixtures from the SP-4 samples with rewritten IDs: `tests/fixtures/herdr/workspace_list.json` (workspaces `w4V`, then `w7Q`), `tests/fixtures/herdr/pane_list_w4V.json` (pane `w4V:p3`, a plain shell; pane `w4V:p11`, Claude with session `9b1e04d2-5c1f-4b0e-8a57-0d3c2f6e1a90`) and `tests/fixtures/herdr/pane_list_w7Q.json` (pane `w7Q:p2`, tab `w7Q:t1`, Claude with session `55583127-a22a-49bc-a803-e777c377a595`). "The moved answers" are: `pane get w4V:p9` failing with `pane_not_found.stderr`, `workspace list` answering `workspace_list.json`, `pane list --workspace w4V` answering `pane_list_w4V.json`, `pane list --workspace w7Q` answering `pane_list_w7Q.json`.
- Not in scope: the budget that ends a slow search (E5-F2-T4); focusing the moved pane (E5-F2-T5).
- Tests first:
  1. `fr60_moved_pane_found_by_session` (FakeRunner): given the W-12 place and the moved answers, when `locate` runs, then it returns `Moved(LivePane { pane: "w7Q:p2", tab: "w7Q:t1", workspace: "w7Q", agent: true })` and `calls()` is exactly `["pane get w4V:p9", "workspace list", "pane list --workspace w4V", "pane list --workspace w7Q"]`.
  2. `ts9_3_moved_search_stops_at_first_match` (FakeRunner): given the moved answers with `pane_list_w4V.json` extended by pane `w4V:p12` carrying session `55583127-a22a-49bc-a803-e777c377a595`, then `locate` returns `Moved` with pane `w4V:p12` and `calls()` has no `pane list --workspace w7Q`.
  3. `fr61_no_pane_with_the_session_is_closed` (FakeRunner): given the moved answers with `pane_list_w7Q.json` holding only a shell pane, then `locate` returns `Closed` after the four calls of test 1 (the Claude pane `w4V:p11` runs another session and is not matched).
  4. `ts9_3_hang_in_workspace_list_is_not_responding` (FakeRunner): given `workspace list` answering `Answer::Hang`, then `locate` returns `NotResponding` and `calls()` is `["pane get w4V:p9", "workspace list"]`.
  5. `ts9_3_hang_in_pane_list_is_not_responding` (FakeRunner): given `pane list --workspace w7Q` answering `Answer::Hang`, then `locate` returns `NotResponding` after four calls.
  6. `ts9_3_failed_workspace_list_ends_as_closed` (FakeRunner): given `workspace list` answering `Answer::fail(1, "x")`, then `locate` returns `Closed` after two calls.
  7. `ts9_3_failed_pane_list_ends_search_as_closed` (FakeRunner): given `pane list --workspace w4V` answering `Answer::fail(1, "x")`, then `locate` returns `Closed` and `calls()` has no `pane list --workspace w7Q`.
  8. `ts9_3_list_readers_follow_sp4_samples` (unit): given `workspace_list.json`, then `workspace_ids` is `["w4V", "w7Q"]`; given `pane_list_w7Q.json`, then `list_panes` yields one pane with `id()` `w7Q:p2` and `session()` `55583127-a22a-49bc-a803-e777c377a595`.
  9. `ts9_3_moved_pane_without_workspace_key_takes_listed_workspace` (FakeRunner): given the moved answers with the workspace key removed from pane `w7Q:p2`, then the `Moved` pane has workspace `w7Q`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every test asserts the full `calls()` sequence.

### E5-F2-T4 Shared 3 second budget

- Status: todo
- Depends on: E5-F2-T3
- Covers: FR-63; AS-11; TECHSPEC 9.3 (3 second budget, per-call timeout `min(2 s, rest)`)
- Size: S
- Scope: `pub const BUDGET: Duration` of 3 seconds and `Budget<'a>`, a `Runner` over another runner: each call gets `min(requested timeout, left())`; once `left()` is zero it returns `TimedOut` with zero `took` without calling the inner runner; after each call it subtracts the call's `took` (saturating). `locate` always wraps the runner it receives in its own `Budget::new(runner, BUDGET)`; callers that locate and then focus wrap both in one more budget (E5-F2-T8, E5-F3-T3), and nested budgets give each call the smallest rest (A-E5-14).
- Not in scope: focus itself (E5-F2-T5); the real-time check through the binary (E5-F2-T6, E5-F2-T9).
- Tests first:
  1. `ts9_3_call_timeout_is_min_of_2s_and_rest` (FakeRunner): given `Budget::new(&fake, BUDGET)` and an answer taking 1.2 s, when two calls each ask for `CALL_TIMEOUT`, then the fake's `timeouts()` is `[2 s, 1.8 s]` and `left()` is 0.6 s.
  2. `ts9_3_spent_budget_times_out_without_calling` (FakeRunner): given `Budget::new(&fake, BUDGET)` and an answer taking 1.5 s, when three calls run, then the third returns `TimedOut` with zero `took` and the fake's `calls()` has two entries.
  3. `as11_locate_ends_the_moved_search_at_3_seconds` (FakeRunner, simulated time): given the W-12 place, `pane get w4V:p9` failing with `pane_not_found` taking 1.2 s, `workspace list` taking 1.0 s, `pane list --workspace w4V` taking 0.5 s and `pane list --workspace w7Q` (the match) taking 0.5 s, when `locate` runs on the bare fake, then it returns `NotResponding`, `timeouts()` is `[2 s, 1.8 s, 0.8 s, 0.3 s]` and the summed `took` is 3.0 s.
  4. `ts9_3_outer_budget_shortens_locate` (FakeRunner): given `Budget::new(&fake, BUDGET)` through which a `workspace list` asking for 3 s and taking 2.5 s has run, when `locate` runs on the W-12 place through that budget with `pane get w4V:p9` answering `pane_agent.json` at once, then it returns `Live` and that call's timeout is 0.5 s.
  5. `ts9_3_budget_constants` (unit): then `BUDGET` is 3 s and `CALL_TIMEOUT` is 2 s.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/adapters/herdr.rs` holds no duration literal but `CALL_TIMEOUT` and `BUDGET` (grep).

### E5-F2-T5 Focus

- Status: todo
- Depends on: E5-F2-T1, E5-F2-T4, E0-F2-T2
- Covers: FR-59, FR-60; TECHSPEC 9.3 (focus), 14 (SP-5)
- Size: M
- Scope: `focus(p, runner) -> Result<Focused, FocusError>` as 9.3, read as updated by SP-5 (E0-F2-T2): when `p.agent`, `agent focus <p.pane>`; exit 0 gives `Ok(Agent)`. Otherwise (no agent, or agent focus failed) `workspace focus <p.workspace>`, and only after its exit 0 `tab focus <p.tab>`; both exit 0 give `Ok(Tab)`. Any call that times out ends at once with `Err(NotResponding)`; any other failure (non-zero exit, spawn failure) leads to the next step and finally `Err(Failed)`. Each call asks for `CALL_TIMEOUT`; the caller's `Budget` shortens it. herdr has no focus by pane ID, so no `pane focus` call is made (FR-59).
- Not in scope: the jump texts and exit codes (E5-F2-T8, E5-F2-T9); the TUI (E5-F3-T3).
- Tests first:
  1. `fr59_agent_pane_is_focused_with_agent_focus` (FakeRunner): given `LivePane { pane: "w4V:p9", tab: "w4V:t1", workspace: "w4V", agent: true }` and `agent focus w4V:p9` answering `Answer::ok("")`, when `focus` runs, then it returns `Ok(Agent)` and `calls()` is `["agent focus w4V:p9"]`.
  2. `fr59_pane_without_agent_focuses_workspace_then_tab` (FakeRunner): given the same pane with `agent: false` and both focus calls answering `Answer::ok("")`, then `Ok(Tab)` and `calls()` is `["workspace focus w4V", "tab focus w4V:t1"]`.
  3. `fr59_failed_agent_focus_falls_back_to_tab` (FakeRunner): given `agent: true`, `agent focus w4V:p9` answering `Answer::fail(1, "x")` and both other calls `Answer::ok("")`, then `Ok(Tab)` and `calls()` is `["agent focus w4V:p9", "workspace focus w4V", "tab focus w4V:t1"]`.
  4. `ts9_3_failed_workspace_focus_is_focus_failed` (FakeRunner): given `agent: false` and `workspace focus w4V` answering `Answer::fail(1, "x")`, then `Err(Failed)` and `calls()` is `["workspace focus w4V"]`.
  5. `ts9_3_failed_tab_focus_is_focus_failed` (FakeRunner): given `agent: false`, `workspace focus w4V` exit 0 and `tab focus w4V:t1` answering `Answer::fail(1, "x")`, then `Err(Failed)`.
  6. `ts9_3_spawn_failure_in_focus_is_focus_failed` (FakeRunner): given `agent: true` and every focus call answering `Answer::SpawnFailed("x".into())`, then `Err(Failed)` after the three calls.
  7. `ts9_3_agent_focus_timeout_is_not_responding` (FakeRunner): given `agent: true` and `agent focus w4V:p9` answering `Answer::Hang`, then `Err(NotResponding)` and `calls()` is `["agent focus w4V:p9"]`.
  8. `ts9_3_workspace_focus_timeout_is_not_responding` (FakeRunner): given `agent: false` and `workspace focus w4V` answering `Answer::Hang`, then `Err(NotResponding)` after one call.
  9. `ts9_3_tab_focus_timeout_is_not_responding` (FakeRunner): given `agent: false`, `workspace focus w4V` exit 0 and `tab focus w4V:t1` answering `Answer::Hang`, then `Err(NotResponding)`.
  10. `fr60_moved_pane_is_focused_by_its_new_id` (FakeRunner): given `LivePane { pane: "w7Q:p2", tab: "w7Q:t1", workspace: "w7Q", agent: true }` and `agent focus w7Q:p2` exit 0, then `Ok(Agent)` and `calls()` is `["agent focus w7Q:p2"]`.
  11. `ts9_3_locate_and_focus_share_the_budget` (FakeRunner): given `Budget::new(&fake, BUDGET)`, `pane get w4V:p9` answering `pane_agent.json` taking 1.5 s and `agent focus w4V:p9` answering `Answer::Hang`, when `locate` and then `focus` run through that budget, then `focus` returns `Err(NotResponding)` and the `agent focus` call's timeout is 1.5 s.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] No test's `calls()` contains `pane focus`.

### E5-F2-T6 The `locate` subcommand

- Status: todo
- Depends on: E5-F2-T4, E5-F1-T1, E5-F1-T5, E2-F1, E1-F5
- Covers: FR-40, FR-60, FR-61, FR-62, FR-63, NFR-05; AS-11 (CLI); TECHSPEC 6.3 (`locate`), 6.7 (`locate`), 9.5
- Size: M
- Scope: `cli::cmd::locate` replaces the E2-F1 `todo!()` arm of `Command::Locate`: find the card with `core::board::find_card` over the loaded config's boards and pass its `Error::NotFound("<ID> not found")` through unchanged (exit 3, code `not_found`, the same text as `show`, E2-F2-T4, A-E5-15), take its `jump_note()` (the card without one is E5-F2-T7), run `locate` with `ProcessRunner::new(env)` and print `cli::out::locate_lines`: `live <pane>`, `moved <pane>` (the live pane), `closed <pane>` (the recorded pane), `not in herdr`, `no pane recorded`, `herdr not responding` or `herdr: <message>`, then `Where: <cwd>`; exit 0 for every status. With `--json`, `LocateJson` is `{"v":1,"status":..,"pane":..,"cwd":..}` with `pane` the live pane for `live` and `moved`, else the recorded pane or null (A-E5-16). Add scenarios: `live-agent` (`pane get w4V:p9` answers a Claude pane with tab `w4V:t1`, workspace `w4V`, session `55583127-a22a-49bc-a803-e777c377a595`; `agent focus w4V:p9` exits 0), `live-shell` (`pane get w4V:p9` answers a shell pane with tab `w4V:t1`, workspace `w4V`; `workspace focus w4V` and `tab focus w4V:t1` exit 0), `moved` (the E5-F2-T3 moved answers; `agent focus w7Q:p2` exits 0), `closed` (`pane get w4V:p9` fails with `pane_not_found`, `workspace list` lists only `w4V`, whose pane list holds no matching session), `herdr-error` (`pane get w4V:p9` exits 1 with stderr `{"error":{"code":"internal","message":"socket closed"}}`) and `slow-then-hang` (`pane get w4V:p9` runs `slow 1.5`, then fails with `pane_not_found`; `workspace list` hangs). "The jump sandbox" is E2-F1's `Sandbox::new()`, `write_config()` and `copy_board("work_4_9", "work")`; "inside herdr" adds `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p11`, `HERDR_TAB_ID=w4V:t1` and `HERDR_WORKSPACE_ID=w4V` through `Sandbox::scenario_mut().herdr(..)` before `cmd(args)`, and a named scenario is set through `scenario_mut().fake_scenario(..)` the same way: every E5-F2 CLI test sets `HERDR_*`, `FAKE_HERDR_SCENARIO`, `BRAIN_SWAP_SESSION`, `BRAIN_SWAP_LOG` or a failpoint only through `sandbox.scenario_mut()` before `cmd(args)`.
- Not in scope: `--note` and the card without a jump note (E5-F2-T7); `jump` (E5-F2-T8, E5-F2-T9); the first-run notice (E2-F2-T1).
- Tests first:
  1. `fr40_locate_live_pane` (CLI, fake herdr): given the jump sandbox inside herdr with scenario `live-agent`, when `brain-swap locate W-12` runs, then it exits 0, stdout is exactly `live w4V:p9` and `Where: /home/smilen/Work/ati.billing`, and `calls(home)` is `["pane get w4V:p9"]`.
  2. `fr60_locate_moved_pane` (CLI, fake herdr): given scenario `moved`, when `brain-swap locate W-12` runs, then stdout's first line is `moved w7Q:p2`, it exits 0, and `calls(home)` is `["pane get w4V:p9", "workspace list", "pane list --workspace w4V", "pane list --workspace w7Q"]`.
  3. `fr61_locate_closed_pane` (CLI, fake herdr): given scenario `closed`, then stdout is `closed w4V:p9` and `Where: /home/smilen/Work/ati.billing`, exit 0.
  4. `fr62_locate_outside_herdr` (CLI, fake herdr): given the jump sandbox without `HERDR_ENV` (the helper still sets `HERDR_BIN_PATH`) and scenario `live-agent`, then stdout is `not in herdr` and `Where: /home/smilen/Work/ati.billing`, exit 0, and `calls(home)` is empty (NFR-05).
  5. `fr62_locate_note_without_pane` (CLI, fake herdr): given the jump sandbox inside herdr and scenario `live-agent`, when `brain-swap locate W-9` runs, then stdout is `no pane recorded` and `Where: /home/smilen/Work/private/brain-swap`, exit 0, and `calls(home)` is empty.
  6. `fr63_locate_hang_is_not_responding` (CLI, fake herdr): given scenario `hang`, then stdout is `herdr not responding` and `Where: /home/smilen/Work/ati.billing`, it exits 0, and the run takes under 2.5 s.
  7. `ts9_3_locate_herdr_error` (CLI, fake herdr): given scenario `herdr-error`, then stdout's first line is `herdr: {"error":{"code":"internal","message":"socket closed"}}` and it exits 0.
  8. `as11_locate_answers_within_3_seconds` (CLI, fake herdr; waits the 3 s the budget demands): given scenario `slow-then-hang`, when `brain-swap locate W-12` runs, then stdout is `herdr not responding` and `Where: /home/smilen/Work/ati.billing`, it exits 0, and the wall time lies between 2.9 s and 3.5 s.
  9. `fr40_locate_never_focuses` (CLI, fake herdr): given scenarios `live-agent`, `live-shell` and `moved` in turn, when `brain-swap locate W-12` runs in each, then no line of `calls(home)` contains `focus`.
  10. `ts6_7_locate_json_per_status` (CLI, fake herdr): given scenarios `live-agent`, `moved`, `closed`, `hang` and `herdr-error` inside herdr, and `live-agent` outside herdr, for W-12, and W-9 inside herdr, when `brain-swap --json locate <ID>` runs, then each stdout is one object whose `status` is `live`, `moved`, `closed`, `not_responding`, `herdr_error`, `outside` and `none` respectively, with `pane` `w4V:p9`, `w7Q:p2`, `w4V:p9`, `w4V:p9`, `w4V:p9`, `w4V:p9` and null and `cwd` the note's directory (`insta::assert_snapshot!` of each raw one-line JSON stdout with the temp root replaced through `Sandbox::redact`).
  11. `ts6_2_locate_unknown_card_exits_3` (CLI): given the jump sandbox, when `brain-swap locate W-99` runs, then it exits 3 with stderr `brain-swap: W-99 not found`, and the `--json` variant prints the error envelope with code `not_found`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Text and JSON snapshots are reviewed and committed (12.2 item 3), and `brain-swap locate --help` describes the command and its `--note` flag (12.2 item 5).

### E5-F2-T7 `--note` and the card without a jump note

- Status: todo
- Depends on: E5-F2-T6
- Covers: FR-40; TECHSPEC 6.3 (`--note <n>`, `no note yet`, out of range), 6.7 (`no_note`)
- Size: S
- Scope: `cli::cmd::target_note(card, note)`: `None` gives `card.jump_note()`; `Some(n)` with `1 <= n <= notes.len()` gives index `n - 1` (1 is the oldest); any other `n` is `Error::NotFound("<ID> has <count> notes")` (exit 3, code `not_found`, A-E5-18). `locate` without a target prints `no note yet`, exits 0, and `--json` gives status `no_note` with `pane` and `cwd` null; this is decided in `cmd.rs` before `locate` runs (6.3). A note chosen with `--note` that has no place locates as `NoPane` (or `OutsideHerdr`) and prints `Where: none`, JSON `cwd` null (A-E5-17). `jump` reuses `target_note` (E5-F2-T8). Fixture `tests/fixtures/jump/W-14.md`: column Doing, template Chore, created `2026-10-08T08:00:00+03:00`, title `# three places`, and a timeline of three notes: `### 2026-10-08T09:00:00+03:00` with place `cwd=/work/a pane=w4V:p3 tab=w4V:t2 workspace=w4V`, `### 2026-10-08T10:00:00+03:00 auto` with place `cwd=/work/b pane=w4V:p9 tab=w4V:t1 workspace=w4V session=55583127-a22a-49bc-a803-e777c377a595`, and `### 2026-10-08T11:00:00+03:00` with no place line; the tests copy it into the jump sandbox's `work` folder.
- Not in scope: the jump note rule itself (E1-F5-T3); `jump`'s exit 6 for `no note yet` (E5-F2-T9).
- Tests first:
  1. `ts6_3_locate_card_without_notes_prints_no_note_yet` (CLI, fake herdr): given the jump sandbox inside herdr with scenario `live-agent`, when `brain-swap locate W-11` runs, then stdout is exactly `no note yet`, it exits 0 and `calls(home)` is empty.
  2. `ts6_7_locate_no_note_json` (CLI): given the same, when `brain-swap --json locate W-11` runs, then stdout is `{"v":1,"status":"no_note","pane":null,"cwd":null}`.
  3. `ts6_3_default_target_is_the_last_placed_note` (CLI): given W-14 in the jump sandbox outside herdr, when `brain-swap locate W-14` runs, then stdout is `not in herdr` and `Where: /work/b`.
  4. `ts6_3_note_flag_counts_from_the_oldest` (CLI): given the same, when `brain-swap locate W-14 --note 1` runs, then stdout is `not in herdr` and `Where: /work/a`.
  5. `ts6_3_note_without_place_prints_where_none` (CLI, fake herdr): given W-14 inside herdr with scenario `live-agent`, when `brain-swap locate W-14 --note 3` runs, then stdout is `no pane recorded` and `Where: none`, it exits 0 and `calls(home)` is empty.
  6. `ts6_3_note_out_of_range_exits_3` (CLI): given W-14, when `brain-swap locate W-14 --note 4` and `brain-swap locate W-14 --note 0` run, then each exits 3 with stderr `brain-swap: W-14 has 3 notes`, and the `--json` variant has code `not_found`.
  7. `ts6_3_note_on_card_without_notes_exits_3` (CLI): given the jump sandbox, when `brain-swap locate W-11 --note 1` runs, then it exits 3 with `brain-swap: W-11 has 0 notes`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `target_note` is the only place in `src/cli/` that turns `--note` into an index (grep).

### E5-F2-T8 The `jump` subcommand: focused paths

- Status: todo
- Depends on: E5-F2-T5, E5-F2-T7
- Covers: FR-40, FR-59, FR-60; AS-10 (CLI); TECHSPEC 6.3 (`jump`), 6.7 (`jump`, `focused`)
- Size: M
- Scope: `cli::cmd::jump` replaces the `todo!()` arm of `Command::Jump`: the same card lookup and `target_note` as `locate`, one `Budget::new(&ProcessRunner::new(env), BUDGET)` shared by `locate` and `focus` (A-E5-14), and for `Live` or `Moved` a `focus`. On `Ok`: `Live` with `Agent` prints `jumped to <ID> (pane <pane>)`, `Live` with `Tab` prints `jumped to <ID> (tab <tab>; pane not focusable)`, `Moved` (either way) prints `pane moved: now <pane>`; exit 0. `JumpJson` adds `focused` to the `LocateJson` fields: `"pane"` for `Agent`, `"tab"` for `Tab`, `false` otherwise, serialised by a hand-written `Serialize` so the object stays a typed struct (E2-F1-T2). Extend the `live-shell` scenario by `pane get w4V:p3` answering a plain-shell pane with tab `w4V:t2` and workspace `w4V`, and `tab focus w4V:t2` exiting 0, for the `--note` test.
- Not in scope: every path that exits 6 (E5-F2-T9); the TUI jump (E5-F3).
- Tests first:
  1. `fr59_jump_focuses_the_agent_pane` (CLI, fake herdr): given the jump sandbox inside herdr with scenario `live-agent`, when `brain-swap jump W-12` runs, then it exits 0, stdout is exactly `jumped to W-12 (pane w4V:p9)` and `calls(home)` is `["pane get w4V:p9", "agent focus w4V:p9"]`.
  2. `fr59_jump_without_agent_focuses_the_tab` (CLI, fake herdr): given scenario `live-shell`, then stdout is `jumped to W-12 (tab w4V:t1; pane not focusable)`, exit 0, and `calls(home)` is `["pane get w4V:p9", "workspace focus w4V", "tab focus w4V:t1"]`.
  3. `as10_jump_to_a_moved_pane_reports_its_new_id` (CLI, fake herdr): given scenario `moved`, then stdout is `pane moved: now w7Q:p2`, exit 0, and the last line of `calls(home)` is `agent focus w7Q:p2`.
  4. `ts6_7_jump_json_names_what_was_focused` (CLI, fake herdr): given scenarios `live-agent`, `live-shell` and `moved`, when `brain-swap --json jump W-12` runs, then the objects are `{"v":1,"status":"live","pane":"w4V:p9","cwd":"/home/smilen/Work/ati.billing","focused":"pane"}`, the same with `"focused":"tab"`, and `{"v":1,"status":"moved","pane":"w7Q:p2","cwd":"/home/smilen/Work/ati.billing","focused":"pane"}` (`insta::assert_snapshot!` of each raw one-line JSON stdout with the temp root replaced through `Sandbox::redact`).
  5. `ts6_3_jump_note_flag_targets_that_note` (CLI, fake herdr): given W-14 in the jump sandbox inside herdr and scenario `live-shell`, when `brain-swap jump W-14 --note 1` runs, then stdout is `jumped to W-14 (tab w4V:t2; pane not focusable)` and the first line of `calls(home)` is `pane get w4V:p3`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Text and JSON snapshots reviewed and committed (12.2 item 3); `brain-swap jump --help` describes the command and its `--note` flag (12.2 item 5).

### E5-F2-T9 The `jump` subcommand: exit 6

- Status: todo
- Depends on: E5-F2-T8
- Covers: FR-61, FR-62, FR-63, NFR-05; TECHSPEC 6.2 (exit 6, no error object), 6.3 (`jump` failures)
- Size: M
- Scope: The paths where `jump` performs no jump, all with exit `EXIT_JUMP_NOT_PERFORMED` (6), nothing on stderr, and with `--json` the `JumpJson` object (never the error envelope) with `focused` false: a focus `Err(Failed)` prints `focus failed: <pane>` and `Where: <cwd>`, JSON `status` unchanged (`live` or `moved`); a focus `Err(NotResponding)` prints `herdr not responding` and `Where: <cwd>`, JSON `status` `not_responding`; `Closed`, `OutsideHerdr`, `NoPane`, `NotResponding` and `HerdrError` print the `locate` lines; a card without a jump note prints `no note yet`, JSON status `no_note`. Add scenarios `focus-fails` (`pane get w4V:p9` answers the `live-agent` pane; `agent focus w4V:p9` and `workspace focus w4V` exit 1 with the SP-5 failure stderr), `focus-hangs` (the same pane; `agent focus w4V:p9` hangs) and `slow-live-focus-hangs` (`pane get w4V:p9` runs `slow 1.5`, then answers the `live-agent` pane; `agent focus w4V:p9` hangs).
- Not in scope: the TUI Messages for the same outcomes (E5-F3-T2).
- Tests first:
  1. `ts6_3_jump_failed_focus_exits_6` (CLI, fake herdr): given the jump sandbox inside herdr with scenario `focus-fails`, when `brain-swap jump W-12` runs, then it exits 6, stdout is exactly `focus failed: w4V:p9` and `Where: /home/smilen/Work/ati.billing`, and stderr is empty.
  2. `ts6_3_jump_failed_focus_json_keeps_status` (CLI, fake herdr): given the same, when `brain-swap --json jump W-12` runs, then it exits 6 and stdout is `{"v":1,"status":"live","pane":"w4V:p9","cwd":"/home/smilen/Work/ati.billing","focused":false}` with no `error` key.
  3. `ts6_3_jump_focus_timeout_exits_6_not_responding` (CLI, fake herdr): given scenario `focus-hangs`, then it exits 6 with stdout `herdr not responding` and `Where: /home/smilen/Work/ati.billing`, and the `--json` variant has `"status":"not_responding"` and `"focused":false`.
  4. `fr61_jump_to_a_closed_pane_exits_6` (CLI, fake herdr): given scenario `closed`, then it exits 6 with stdout `closed w4V:p9` and `Where: /home/smilen/Work/ati.billing`.
  5. `fr62_jump_outside_herdr_exits_6_without_herdr_call` (CLI, fake herdr): given the jump sandbox without `HERDR_ENV` and scenario `live-agent`, then it exits 6 with stdout `not in herdr` and `Where: /home/smilen/Work/ati.billing`, and `calls(home)` is empty (NFR-05).
  6. `fr62_jump_to_a_note_without_pane_exits_6` (CLI, fake herdr): given the jump sandbox inside herdr, when `brain-swap jump W-9` runs, then it exits 6 with stdout `no pane recorded` and `Where: /home/smilen/Work/private/brain-swap`.
  7. `fr63_jump_while_herdr_hangs_exits_6` (CLI, fake herdr): given scenario `hang`, then it exits 6 with stdout `herdr not responding` and `Where: /home/smilen/Work/ati.billing` within 2.5 s.
  8. `ts9_3_jump_herdr_error_exits_6` (CLI, fake herdr): given scenario `herdr-error`, then it exits 6 and stdout's first line starts with `herdr: `.
  9. `ts6_3_jump_card_without_note_exits_6` (CLI): given the jump sandbox, when `brain-swap jump W-11` runs, then it exits 6 with stdout `no note yet`, and the `--json` variant is `{"v":1,"status":"no_note","pane":null,"cwd":null,"focused":false}`.
  10. `as11_jump_budget_is_shared_with_focus` (CLI, fake herdr; waits the 3 s the budget demands): given scenario `slow-live-focus-hangs`, when `brain-swap jump W-12` runs, then it exits 6 with stdout `herdr not responding` and `Where: /home/smilen/Work/ati.billing`, and the wall time lies between 2.9 s and 3.5 s.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every exit 6 test asserts an empty stderr and, in its JSON variant, the absence of the `error` key.

## E5-F3 TUI jump

- Depends on: E5-F2, E4-F2, E4-F3, E4-F4, E2-F3
- Covers: FR-25, FR-28, FR-59, FR-60, FR-61, FR-62, FR-63
- Spec: TECHSPEC 7.1 (`Effect::Jump`, the jump outcomes, Board and Detail `jump`), 7.5, 7.7, 9.3, 12.1 (TUI), 14 (SP-2); AS-1 step 7, AS-4, AS-5, AS-6, AS-10, AS-11
- Scope: Enter (the `jump` action) on a card in the board jumps to its newest placed note, and in the detail view to the note shown. `update` emits the jump effect with the status `jumping...`, or opens the Message `no note yet`. The runtime runs the effect: `locate`, then `focus` for a live or moved pane, all within one 3 second budget; a focused pane ends the TUI with exit 0, closing the herdr popup, and every other outcome opens a Message with the working directory (`pane closed` and the `/bs-link` hint, `not in herdr`, `no pane recorded`, `herdr not responding`, `herdr: <message>`, `focus failed: <pane>`) while the TUI stays usable. The SP-2 answer may move the terminal restore before the focus. AS-6 runs end to end without a terminal or real herdr; AS-1 step 7, AS-4, AS-5, AS-10 and AS-11 run by hand in real herdr.
- Provides: `tui::app::Effect::Jump(CardId, usize)` (the index into `Card.notes`, 0 the oldest, as `jump_note()`) and `tui::app::Outcome::Jumped`, `Outcome::NotJumped(PaneStatus, Option<PathBuf>)`, `Outcome::FocusFailed(String, Option<PathBuf>)` (added unless E4-F1 has them, A-E5-23); the Board and Detail `jump` arms and the three outcome arms of `tui::app::update`; `tui::app::jump_message(outcome: &Outcome, card: CardId) -> Option<String>`; `tui::runtime::run_jump(env: &Env, card: &Card, note: usize, runner: &dyn Runner) -> Outcome`; the field `runner: Box<dyn Runner>` of `tui::runtime::Parts`; fake herdr scenario `as6-shell`; fixture `tests/fixtures/boards/as6/`
- Requires: E5-F2 (`locate`, `focus`, `Budget`, `BUDGET`, `PaneStatus`, `LivePane`, `Focused`, `FocusError`) and E5-F1 (`ProcessRunner`, `FakeRunner`, `Answer`, the fake herdr script); E4-F1 (`App`, `Mode` with `Message { text, back }`, `Input::Done`, `Effect`, `Outcome`, `update`, the runtime with its injectable Parts (terminal guard, events, clock), the status line); E4-F2 (Board selection and focused column); E4-F3 (Detail mode and its note index); E4-F4 (template pick and title input, for AS-6); E2-F3 (`brain-swap park --card` from stdin inside herdr, for AS-6); E1-F5 (`Card::jump_note`, the `work_4_9` board); E0-F2 the SP-2 answer (TECHSPEC 7.7 as updated)
- Feature DoD:
  - [ ] Enter in Board targets the jump note and in Detail the note shown, or opens `no note yet` (E5-F3-T1).
  - [ ] A state test per outcome, including a failed focus (E5-F3-T2).
  - [ ] The jump effect locates and focuses within one budget; a focused pane ends the TUI with exit 0, anything else keeps it open (E5-F3-T3).
  - [ ] The terminal restore follows the SP-2 order, or the task is closed as not needed (E5-F3-T4).
  - [ ] AS-6 end to end (E5-F3-T5).
  - [ ] Manual AS-1 step 7 and AS-10 recorded (E5-F3-T6).
  - [ ] Manual AS-4, AS-5 and AS-11 recorded (E5-F3-T7).

### E5-F3-T1 Jump key in Board and Detail

- Status: todo
- Depends on: E5-F2-T1, E5-F2-T7, E4-F2, E4-F3
- Covers: FR-25, FR-28; TECHSPEC 7.1 (Board `jump` emits `Jump` on `jump_note()`, Detail `jump` targets the note shown), 7.7 (target, `jumping...`, `no note yet`)
- Size: S
- Scope: Add `Effect::Jump(CardId, usize)` and the `Outcome` variants `Jumped`, `NotJumped(PaneStatus, Option<PathBuf>)` and `FocusFailed(String, Option<PathBuf>)` to `tui::app` unless E4-F1 already has them (A-E5-23). In `update`: Board `jump` on the selected card emits `[Effect::Jump(id, i)]` for `i = card.jump_note()` and sets the status line to `jumping...`; a card without a jump note opens `Mode::Message { text: "no note yet", back: Board }` and emits nothing; with no card selected `jump` does nothing (A-E5-22). Detail `jump` emits `Jump(card, i)` with `i` the `Card.notes` index of the note shown, or opens `no note yet` for a card without notes. The key comes from the key map's `jump` action (default `enter`, 5.2). In these tests "the board" is an `App` on the loaded `work_4_9` board with default keys, built as E4-F1's `update` tests build it.
- Not in scope: the outcomes (E5-F3-T2); running the effect (E5-F3-T3); the Detail note navigation itself (E4-F3).
- Tests first:
  1. `fr25_enter_on_board_emits_jump_to_the_jump_note` (TUI `update`): given the board with W-12 selected, when `update(Key(enter))` runs, then it returns `[Effect::Jump(W-12, 1)]` and `app.status` is `Some("jumping...")`.
  2. `fr25_jump_skips_a_newer_unplaced_note` (TUI `update`): given the board with W-14 of E5-F2-T7 (notes placed, placed, unplaced) added and selected, when `update(Key(enter))` runs, then it returns `[Effect::Jump(W-14, 1)]`.
  3. `ts7_7_board_jump_without_jump_note_shows_no_note_yet` (TUI `update`): given the board with W-11 selected, when `update(Key(enter))` runs, then it returns no effect and the mode is `Message { text: "no note yet", back: Board }`.
  4. `fr28_detail_jump_targets_the_shown_note` (TUI `update`): given Detail on W-12 opened with `o` and moved one note older with `j`, when `update(Key(enter))` runs, then it returns `[Effect::Jump(W-12, 0)]`.
  5. `fr28_detail_opens_on_the_newest_note_as_target` (TUI `update`): given Detail on W-12 opened with `o`, when `update(Key(enter))` runs, then it returns `[Effect::Jump(W-12, 1)]`.
  6. `ts7_7_detail_jump_without_notes_shows_no_note_yet` (TUI `update`): given Detail on W-11, when `update(Key(enter))` runs, then the mode is `Message { text: "no note yet", back: Detail { .. } }` and no effect is returned.
  7. `fr25_jump_follows_the_configured_key` (TUI `update`): given the board with `jump = "g"` in the key map and W-12 selected, when `update(Key(g))` runs, then it returns `[Effect::Jump(W-12, 1)]`, and `update(Key(enter))` returns no `Jump`.
  8. `ts7_7_board_jump_without_a_selected_card_does_nothing` (TUI `update`): given an `App` on a board folder without card files (first column focused, nothing selected), when `update(Key(enter))` runs, then it returns no effect, the mode stays `Board` and the status line is unchanged.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] A `TestBackend` snapshot at 80x24 after test 1 shows `jumping...` in the status line.

### E5-F3-T2 Jump outcomes: exit or Message

- Status: todo
- Depends on: E5-F3-T1
- Covers: FR-59, FR-61, FR-62, FR-63; AS-4, AS-5, AS-11 (state side); TECHSPEC 7.1 (outcomes), 7.7 (Messages)
- Size: M
- Scope: In `update`: `Done(Jumped)` returns `[Effect::Quit]` (FR-59). `Done(NotJumped(status, cwd))` and `Done(FocusFailed(pane, cwd))` open `Mode::Message { text, back }` with `back` the mode the jump started from and clear the `jumping...` status. `jump_message(outcome, card)` builds the text, one item per line (A-E5-21): the status line `pane closed`, `not in herdr`, `no pane recorded`, `herdr not responding`, `herdr: <message>` or `focus failed: <pane>`; then the working directory when the note has one; then, for `Closed` only, `open a session there and run /bs-link <ID>` (FR-61), the ID taken from the Detail card or the Board selection (A-E5-20). A focus that timed out arrives as `NotJumped(PaneStatus::NotResponding, cwd)` (A-E5-19). Any key in the Message returns to `back` (E4-F1), and the TUI stays usable.
- Not in scope: producing the outcomes (E5-F3-T3); Message rendering and its key handling in general (E4-F1).
- Tests first:
  1. `fr59_jumped_outcome_quits` (TUI `update`): given the board after test 1 of E5-F3-T1, when `update(Done(Jumped))` runs, then it returns `[Effect::Quit]`.
  2. `fr61_closed_outcome_shows_pane_closed_cwd_and_link_hint` (TUI `update`): given the board with W-12 selected, when `update(Done(NotJumped(Closed, Some("/home/smilen/Work/ati.billing"))))` runs, then it returns no effect and the mode is `Message` with text `pane closed`, `/home/smilen/Work/ati.billing`, `open a session there and run /bs-link W-12` (three lines) and `back` `Board`.
  3. `fr62_outside_outcome_shows_not_in_herdr_and_cwd` (TUI `update`): given the same, when `Done(NotJumped(OutsideHerdr, Some("/home/smilen/Work/ati.billing")))` arrives, then the text is `not in herdr` and `/home/smilen/Work/ati.billing`.
  4. `fr62_no_pane_outcome_shows_no_pane_recorded_and_cwd` (TUI `update`): given W-9 selected, when `Done(NotJumped(NoPane, Some("/home/smilen/Work/private/brain-swap")))` arrives, then the text is `no pane recorded` and `/home/smilen/Work/private/brain-swap`.
  5. `fr63_not_responding_outcome_shows_message_and_cwd` (TUI `update`): given W-12 selected, when `Done(NotJumped(NotResponding, Some("/home/smilen/Work/ati.billing")))` arrives, then the text is `herdr not responding` and `/home/smilen/Work/ati.billing`.
  6. `ts7_7_herdr_error_outcome_shows_the_message` (TUI `update`): given W-12 selected, when `Done(NotJumped(HerdrError("socket closed"), Some(..)))` arrives, then the first line is `herdr: socket closed`.
  7. `ts7_7_failed_focus_outcome_shows_pane_and_cwd` (TUI `update`): given W-12 selected, when `Done(FocusFailed("w4V:p9", Some("/home/smilen/Work/ati.billing")))` arrives, then the text is `focus failed: w4V:p9` and `/home/smilen/Work/ati.billing`.
  8. `ts7_7_message_without_cwd_has_one_line` (TUI `update`): given W-12 selected, when `Done(NotJumped(NoPane, None))` arrives, then the text is exactly `no pane recorded`.
  9. `ts7_7_outcome_in_detail_returns_to_detail` (TUI `update`): given Detail on W-12 showing note 1 of 2, when `Done(NotJumped(Closed, Some(..)))` arrives and then any key, then the mode is that same `Detail` with the same note index.
  10. `fr63_tui_stays_usable_after_the_message` (TUI `update`): given the Message of test 5, when `update(Key(x))`, then `update(Key(l))` and then `update(Key(enter))` run, then the mode is `Board` after the first key, W-9 in Done is selected after the second, and the third returns `[Effect::Jump(W-9, 0)]`.
  11. `ts7_7_jumping_status_is_cleared` (TUI `update`): given the board after a `Jump` was emitted, when any `NotJumped` outcome arrives, then `app.status` is `None`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `TestBackend` snapshots at 80x24 of the Messages of tests 2 and 5 are reviewed and committed, showing every line without colour cues (NFR-10).

### E5-F3-T3 Running the jump effect

- Status: todo
- Depends on: E5-F3-T2, E5-F2-T5, E5-F2-T7, E4-F1
- Covers: FR-28, FR-59, FR-60, FR-61, FR-62, FR-63; AS-11; TECHSPEC 7.1 (runtime runs effects through `adapters::herdr`), 7.7 (focus first, then exit), 9.3 (shared budget)
- Size: M
- Scope: `tui::runtime::run_jump(env, card, note, runner) -> Outcome`: take `card.notes[note].place`, wrap `runner` in one `Budget::new(runner, BUDGET)`, run `locate`; for `Live` or `Moved` run `focus` through the same budget: `Ok(_)` gives `Jumped`, `Err(Failed)` gives `FocusFailed(<live pane>, cwd)`, `Err(NotResponding)` gives `NotJumped(PaneStatus::NotResponding, cwd)`; any other status gives `NotJumped(status, cwd)`. The runtime handles `Effect::Jump(id, note)` by drawing one frame (so `jumping...` shows), finding the card in `app.board`, calling `run_jump` with its runner and sending `Input::Done(outcome)`. The runtime takes the `Runner` as an injectable part: this task adds `runner: Box<dyn Runner>` to E4-F1's injectable Parts (terminal guard, events, clock), whether or not E4-F5 has added `launcher` yet (12.1, A-E5-24), and the production entry point puts `Box::new(ProcessRunner::new(&env))` there. On `Quit` after `Jumped` it restores the terminal and returns exit 0 through E4-F1's quit path, which closes the herdr popup (FR-59).
- Not in scope: restoring the terminal before the focus (E5-F3-T4); the `update` arms (E5-F3-T1, E5-F3-T2).
- Tests first:
  1. `fr59_live_agent_pane_is_jumped` (FakeRunner): given W-12 of `work_4_9`, note 1, `herdr_env` `1`, `pane get w4V:p9` answering `pane_agent.json` and `agent focus w4V:p9` exit 0, when `run_jump` runs, then it returns `Jumped` and `calls()` is `["pane get w4V:p9", "agent focus w4V:p9"]`.
  2. `fr59_live_shell_pane_is_jumped_by_tab` (FakeRunner): given `pane get w4V:p9` answering a shell pane with tab `w4V:t1` and both focus calls exit 0, then `Jumped` and `calls()` is `["pane get w4V:p9", "workspace focus w4V", "tab focus w4V:t1"]`.
  3. `fr60_moved_pane_is_jumped` (FakeRunner): given the E5-F2-T3 moved answers and `agent focus w7Q:p2` exit 0, then `Jumped` and the last call is `agent focus w7Q:p2`.
  4. `ts7_7_failed_focus_is_focus_failed_with_cwd` (FakeRunner): given the answers of test 1 with `agent focus w4V:p9` and `workspace focus w4V` exiting 1, then `FocusFailed("w4V:p9", Some("/home/smilen/Work/ati.billing"))`.
  5. `ts7_7_timed_out_focus_is_not_responding` (FakeRunner): given the answers of test 1 with `agent focus w4V:p9` answering `Answer::Hang`, then `NotJumped(NotResponding, Some("/home/smilen/Work/ati.billing"))`.
  6. `fr61_closed_pane_is_not_jumped_without_focus` (FakeRunner): given `pane get w4V:p9` failing with `pane_not_found` and lists without the session, then `NotJumped(Closed, Some(..))` and no `calls()` entry contains `focus`.
  7. `fr62_outside_herdr_is_not_jumped_without_a_call` (FakeRunner without answers): given `herdr_env` `None`, then `NotJumped(OutsideHerdr, Some("/home/smilen/Work/ati.billing"))` and `calls()` is empty.
  8. `fr62_note_without_pane_is_not_jumped` (FakeRunner without answers): given W-9 note 0 inside herdr, then `NotJumped(NoPane, Some("/home/smilen/Work/private/brain-swap"))`.
  9. `as11_hang_is_not_jumped_within_3_seconds` (FakeRunner, simulated time): given `pane get w4V:p9` failing with `pane_not_found` taking 1.5 s and `workspace list` answering `Answer::Hang`, then `NotJumped(NotResponding, Some(..))` and the summed `took` is at most 3 s.
  10. `ts9_3_locate_and_focus_share_one_budget` (FakeRunner): given `pane get w4V:p9` answering `pane_agent.json` taking 1.5 s and `agent focus w4V:p9` exit 0, then `run_jump` returns `Jumped` and the `agent focus` call's timeout is 1.5 s.
  11. `fr28_run_jump_uses_the_given_note` (FakeRunner): given W-14 of E5-F2-T7 copied into the board, note 0, and `pane get w4V:p3` answering `pane_shell.json`, then the first call is `pane get w4V:p3`.
  12. `fr59_jumped_outcome_ends_the_runtime_with_exit_0` (TUI runtime, FakeRunner and E4-F1's test terminal guard): given the runtime on `work_4_9` with W-12 selected and the answers of test 1, when the key `enter` is fed, then the runtime returns exit 0 and the guard recorded exactly one restore.
  13. `fr63_tui_stays_open_after_not_responding` (TUI runtime, FakeRunner): given the runtime on `work_4_9` with every herdr call answering `Answer::Hang`, when `enter` is fed, then the runtime keeps running, the next drawn frame (`TestBackend`) contains `herdr not responding` and `/home/smilen/Work/ati.billing`, and feeding `x` and then `q` ends it with exit 0.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/tui/` calls `locate` and `focus` only inside `run_jump` (grep), and the runtime constructs `ProcessRunner` only in its production entry point.

### E5-F3-T4 Terminal order per SP-2

- Status: todo
- Depends on: E5-F3-T3, E0-F2-T1
- Covers: FR-59; TECHSPEC 7.7 (SP-2 may move the restore before the focus), 14 (SP-2)
- Size: S
- Scope: Runs only if the TECHSPEC 16 `SP-2` line chose "restore, then focus" for the popup; if it chose "focus, then exit", the task is closed as `not needed: SP-2 focus then exit`; if it chose the section 14 fallback (exit first, then a detached `brain-swap jump <ID>`), the task is closed and a new task is filed once TECHSPEC 7.7 describes that path (A-E5-25). When enabled: after `locate` returns `Live` or `Moved`, the runtime restores the terminal through E4-F1's terminal guard, then focuses; on `Jumped` it exits 0 without a second restore; on `FocusFailed` or a timed-out focus it re-enters the alternate screen and raw mode and shows the same Message as E5-F3-T2. `run_jump` gains a hook between locate and focus for this (named in the commit).
- Not in scope: the detached jump fallback (A-E5-25); the Message texts (E5-F3-T2).
- Tests first:
  1. `ts7_7_restore_precedes_focus` (TUI runtime, FakeRunner, recording terminal guard): given the answers of E5-F3-T3 test 1, when `enter` is fed, then the recorded order of guard events and runner calls is `pane get w4V:p9`, `restore`, `agent focus w4V:p9`, and the runtime exits 0 with one restore in total.
  2. `ts7_7_failed_focus_after_restore_reenters_and_shows_the_message` (TUI runtime): given the answers of E5-F3-T3 test 4, when `enter` is fed, then the order is `restore`, the focus calls, `enter`, and the next frame shows `focus failed: w4V:p9`.
  3. `ts7_7_status_without_a_live_pane_never_restores` (TUI runtime): given the answers of E5-F3-T3 test 6, when `enter` is fed, then the guard records no restore and the frame shows `pane closed`.
- DoD:
  - [ ] Either the task is closed with the SP-2 line cited in the commit message, or the tests above pass and the first was committed red (12.2 item 1).
  - [ ] When enabled, TECHSPEC 7.7 and the code name the same order (the commit message quotes the SP-2 line).

### E5-F3-T5 AS-6 end to end

- Status: todo
- Depends on: E5-F3-T3, E4-F4, E2-F3-T4
- Covers: FR-25, FR-59; AS-6
- Size: M
- Scope: One scenario test, `tests/tui_as6.rs`, with no terminal and no real herdr. Fixture `tests/fixtures/boards/as6/board.md` (`---`, `next: 12`, `---`, so the next card is W-12 as the setup's W-1 to W-11 imply), copied as the default board `work` into a temp home whose config the test writes. The TUI part runs in process: an `App` on that board, keys fed to `update`, and every effect executed through the runtime's effect executor (E4-F1) with the real store and an `Env` from `core_env`. The park runs as a process through the 12.1 spawn helper inside a fake herdr pane with scenario `as6-shell` (added here: `pane get w4V:p3` answers a plain-shell pane with tab `w4V:t2` and workspace `w4V`, `agent_status` `unknown`), with `HERDR_ENV=1`, `HERDR_PANE_ID=w4V:p3`, `HERDR_TAB_ID=w4V:t2`, `HERDR_WORKSPACE_ID=w4V`, no `--session` and no `BRAIN_SWAP_SESSION`. The jump step uses `run_jump` with a `FakeRunner` scripted like `as6-shell` plus `workspace focus w4V` and `tab focus w4V:t2` exiting 0.
- Not in scope: the terminal prompts of `park` (E2-F3-T6; a real terminal run is manual in E6-F1); the creation and move details (E4-F4, E4-F2).
- Tests first:
  1. `as6_tui_card_park_from_shell_and_enter_focuses_the_tab` (TUI and CLI, fake herdr, FakeRunner): given the `as6` board with no card, when the keys `n`, `f`, the characters of `migrate invoices to v13` and `enter` are fed and their effects run, then W-12 exists in Todo and is selected; when `L` is fed and its effect runs, then W-12 is in Doing and still selected; when `brain-swap park --card W-12` runs in the fake pane with stdin `Doing: a`, `Next: b`, `Watch out: c`, then it exits 0 with stdout `parked to W-12: migrate invoices to v13` and the new heading has no `auto`, and its place line holds `pane=w4V:p3 tab=w4V:t2 workspace=w4V` and no `session=`; when the board is reloaded (`Input::Reloaded`) and `enter` is fed, then `update` returns `Jump(W-12, 0)`, `run_jump` returns `Jumped` with `calls()` `["pane get w4V:p3", "workspace focus w4V", "tab focus w4V:t2"]`, and `update(Done(Jumped))` returns `[Effect::Quit]`.
- DoD:
  - [ ] The test above passes; it was committed red before the missing pieces turned it green (12.2 item 1).
  - [ ] The test runs under `cargo test` without a terminal (it passes with stdin and stdout redirected from and to `/dev/null`).

### E5-F3-T6 Manual jumps that land (AS-1 step 7, AS-10)

- Status: todo
- Depends on: E5-F3-T3, E5-F3-T4
- Covers: FR-25, FR-28, FR-59, FR-60; AS-1 (step 7), AS-10; TECHSPEC 7.7, 12.1 (manual)
- Size: M
- Scope: Run in real herdr 0.8.2 and Claude Code the steps that land a jump: Enter from the board popup focuses the note's pane and closes the popup (AS-1 step 7), Enter in the detail view targets the note shown (FR-28) and focuses a plain-shell pane's tab (AS-6 last step), and a pane moved to another workspace is found by its session (AS-10). Notes are parked with Claude Code's `!` shell escape (`! brain-swap park --card W-12` with the parts on stdin), whose shell sees the pane's `HERDR_*`, so the session comes from herdr (6.4 step 3) and no pack is needed (A-E5-26).
- Not in scope: the fallbacks (E5-F3-T7); the pack steps of AS-1 (E3-F1-T8).
- Procedure:
  1. Install the build with `cargo install --path .`; back up `~/.config/brain-swap/` and `~/.local/state/brain-swap/`; point the `work` board at a scratch copy of the 4.9 board; add the 9.4 key binding to herdr's config (the output of `brain-swap install herdr` if E5-F4 is done, else by hand with the binary's absolute path) and run `herdr server reload-config`.
  2. In workspace A open a plain-shell pane S (record its pane, tab and workspace IDs) and run `printf 'Doing: shell note\nNext: x\nWatch out: y\n' | brain-swap park --card W-12`.
  3. In workspace A open pane P (record its ID), start `claude`, and record `agent_session.value` from `herdr pane get <P>`; in the session run `! printf 'Doing: probe\nNext: jump back\nWatch out: nothing\n' | brain-swap park --card W-12`; pass when the new note's place names P, its tab and workspace and the recorded session.
  4. AS-1 step 7: switch to workspace B, press the board key, select W-12 and press Enter; pass when the popup closes and herdr focuses P; note the time and key presses from the board key to typing in P (SM-1).
  5. FR-28: press the board key, `o` on W-12, `j` once (the note from S), Enter; pass when the popup closes and herdr shows S's tab.
  6. AS-10: run `herdr pane move <P> --new-workspace`, record the new ID P2 and check that `herdr pane get <P>` reports `pane_not_found`; press the board key and Enter on W-12; pass when herdr focuses P2 and the popup closes.
  7. From S run `brain-swap jump W-12`; pass when it prints `pane moved: now <P2>` and focus moves to P2.
  8. Restore the backups and herdr's config, and run `herdr server reload-config`.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the herdr and Claude Code versions, the pane IDs, the SP-2 order in effect, each step as passed or failed with what was seen, and the step 4 time and key count; this replaces 12.2 items 1 and 3.
  - [ ] Every failed step is filed as a bug task in this feature (next free task number).

### E5-F3-T7 Manual jumps that fall back (AS-4, AS-5, AS-11)

- Status: todo
- Depends on: E5-F3-T3, E5-F3-T4
- Covers: FR-61, FR-62, FR-63; AS-4, AS-5, AS-11; TECHSPEC 7.7, 12.1 (manual)
- Size: M
- Scope: Run in real herdr the jumps that cannot land: a closed pane (AS-4), the board outside herdr and a note parked outside herdr (AS-5), and a herdr server that does not answer (AS-11). A stopped herdr server also stops herdr's own display, so for AS-11 the TUI runs in a plain terminal outside herdr with `HERDR_ENV=1` and the pane's `HERDR_SOCKET_PATH` exported, and the server is paused with `kill -STOP` (A-E5-27).
- Not in scope: the jumps that land (E5-F3-T6); the clipboard fallback (E7-F7).
- Procedure:
  1. Set up as E5-F3-T6 step 1; park W-12 from a Claude pane P as in E5-F3-T6 step 3, and W-9 from a plain terminal outside herdr (its place is only a cwd).
  2. AS-4: quit Claude in P and close P, so that no live pane runs W-12's last session; press the board key and Enter on W-12; pass when the TUI stays open showing `pane closed`, the note's working directory and `open a session there and run /bs-link W-12`.
  3. Open a pane in that directory, start `claude` and park W-12 there with the `!` shell escape (or `/bs-link W-12` and `/bs-park` when the pack is installed); pass when the new note's place names the new pane.
  4. AS-5 outside herdr: in a plain terminal outside herdr run `brain-swap` and press Enter on W-12; pass when it shows `not in herdr` and the note's directory and stays open.
  5. AS-5 inside herdr: press the board key and Enter on W-9; pass when it shows `no pane recorded` and W-9's directory.
  6. AS-11: in a plain terminal outside herdr export `HERDR_ENV=1` and the `HERDR_SOCKET_PATH` copied from a herdr pane, run `brain-swap`, then `kill -STOP` the herdr server process; press Enter on W-12 and time it; pass when `herdr not responding` and the note's directory appear within 3 seconds and `j`, `k`, `o` and Esc still work; then `kill -CONT` the server.
  7. Restore the backups and herdr's config, and run `herdr server reload-config`.
- DoD:
  - [ ] One commit (empty when no file changed) whose message records the date, the herdr and Claude Code versions, the pane IDs, each step as passed or failed with what was seen, and the step 6 time; this replaces 12.2 items 1 and 3.
  - [ ] Every failed step is filed as a bug task in this feature (next free task number).

## E5-F4 install herdr

- Depends on: E2-F1, E0-F2
- Covers: FR-64
- Spec: TECHSPEC 6.3 (`install herdr`), 6.7 (`snippet`), 9.4, 14 (SP-2, SP-7), 12.2 item 5 (README)
- Scope: `brain-swap install herdr` prints the 9.4 key binding with the canonical absolute path of the running binary (`current_exe`, read once by `main` into `Env.exe`, since the herdr server may lack `~/.cargo/bin` on its `PATH`), followed by the instruction to run `herdr server reload-config`; `--json` returns the snippet. It never edits herdr's config, reads no brain-swap config and runs no herdr command. The key is the one SP-7 confirmed, and if SP-2 found the popup without `HERDR_*`, the command gets the `env` prefix of section 14. A README section explains pasting the snippet, reloading, changing the key and a second binding for `--board home`.
- Provides: `adapters::herdr::key_snippet(command: &str) -> String` (the six 9.4 lines with `command` as a TOML basic string), the `InstallTarget::Herdr` handler `cli::cmd::install_herdr`, `cli::out::SnippetJson`; README section `## herdr key binding`
- Requires: E2-F1 (`InstallTarget::Herdr` with a `todo!()` arm, `cli::run` skipping the config prelude for `install`, `Out`, `Sandbox` with `scenario_mut`, through which any E5-F4 CLI test sets `HERDR_*`, `FAKE_HERDR_SCENARIO`, `BRAIN_SWAP_SESSION`, `BRAIN_SWAP_LOG` or a failpoint before `cmd(args)`); E1-F1 (`Env.exe`, the canonical `current_exe` that `src/main.rs` reads; spawn helper `command` and `FAKE_HERDR`, `tests/docs.rs`); E0-F2 the SP-7 key and the SP-2 popup answer (TECHSPEC 9.4 as updated); nothing from E5-F1, since `install herdr` makes no herdr call
- Feature DoD:
  - [ ] Snippet snapshot (E5-F4-T1).
  - [ ] The SP-2 `env` prefix is applied, or the task is closed as not needed (E5-F4-T2).
  - [ ] README carries the herdr section, checked against `key_snippet` (E5-F4-T3).

### E5-F4-T1 Key binding snippet

- Status: todo
- Depends on: E2-F1, E0-F2-T3
- Covers: FR-64; TECHSPEC 6.3 (`install herdr`), 6.7 (`snippet`), 9.4
- Size: S
- Scope: `adapters::herdr::key_snippet(command)` renders exactly the six lines of 9.4 (comment, `[[keys.command]]`, `key` as SP-7 left it in 9.4, `type = "popup"`, `command`, `width = "90%"`, `height = "90%"`), with `command` escaped as a TOML basic string (`\` and `"`, A-E5-29). `cli::cmd::install_herdr` replaces the `todo!()` arm: `env.exe` gives the path (the canonical `current_exe` that `src/main.rs` reads into `Env`, so `src/cli/` reads no environment; E3-F2 takes its binary path the same way, A-E3-17); `env.exe` being `None` fails `Error::Io("cannot determine the brain-swap binary path")`, exit 1 (A-E5-33); text mode prints the snippet, a blank line and `then run: herdr server reload-config` (A-E5-28); `--json` prints `SnippetJson` `{"v":1,"snippet":"<the six lines>"}`. It reads and writes no config (A-E2-01, A-E5-32), writes no file and runs no herdr command.
- Not in scope: the `env` prefix (E5-F4-T2); the README (E5-F4-T3); `install claude` (E3-F2).
- Tests first:
  1. `fr64_install_herdr_prints_the_snippet` (CLI): given `Sandbox::new()`, when `brain-swap install herdr` runs, then it exits 0 and `insta::assert_snapshot!` of stdout, with the canonical binary path replaced by `<brain-swap>` through `str::replace` (and the temp root through `Sandbox::redact`), shows the 9.4 lines with `command = "<brain-swap>"`, a blank line and `then run: herdr server reload-config`.
  2. `fr64_snippet_names_the_canonical_absolute_path` (CLI): given a symlink `<root>/bin/bs` to `env!("CARGO_BIN_EXE_brain-swap")`, when `<root>/bin/bs install herdr` runs through `command(&scenario, <symlink>)`, then the `command = ` line names `std::fs::canonicalize(env!("CARGO_BIN_EXE_brain-swap"))` and not the symlink.
  3. `fr64_install_herdr_never_edits_herdr_config` (CLI): given `$XDG_CONFIG_HOME/herdr/config.toml` holding `[keys]`, when `brain-swap install herdr` runs, then that file's bytes are unchanged and the list of files under the sandbox root is the same before and after.
  4. `ts6_7_install_herdr_json_returns_the_snippet` (CLI): given `Sandbox::new()`, when `brain-swap --json install herdr` runs, then stdout is one object with `v` 1 and `snippet` equal to `key_snippet(<canonical path>)`, and holds no reload line.
  5. `ts9_4_snippet_parses_as_a_herdr_key_command` (unit): given `key_snippet("/x/brain-swap")`, when parsed with `toml`, then `keys.command[0]` has `key` `prefix+alt+b` (or the SP-7 key), `type` `popup`, `command` `/x/brain-swap`, `width` `90%` and `height` `90%`.
  6. `ts9_4_command_is_escaped_as_toml_string` (unit): given `key_snippet("/a \"b\"\\c")`, then the text holds `command = "/a \"b\"\\c"` and parsing it gives back `/a "b"\c`.
  7. `ts6_3_install_herdr_ignores_a_broken_config` (CLI): given a `config.toml` whose only line is `default_board = `, when `brain-swap install herdr` runs, then it exits 0 and the config bytes are unchanged.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The snippet snapshot is reviewed and committed (12.2 item 3), and `brain-swap install herdr --help` says it prints a binding and edits nothing.
  - [ ] `install_herdr` uses no `Runner` and opens no file for writing (grep of the handler), so it runs no herdr command; `std::env::current_exe` appears only in `src/main.rs` (grep).

### E5-F4-T2 SP-2 environment prefix

- Status: todo
- Depends on: E5-F4-T1, E0-F2-T1
- Covers: FR-64; TECHSPEC 9.4, 14 (SP-2 fallback for missing `HERDR_*`)
- Size: S
- Scope: Runs only if the TECHSPEC 16 `SP-2` line records that a popup command lacks `HERDR_*`; otherwise the task is closed as `not needed: popup sees HERDR_*`. When enabled, `install_herdr` builds the command as `env HERDR_ENV=1 HERDR_BIN_PATH=<herdr path> <brain-swap path>` (plus `HERDR_SOCKET_PATH=<path>` if SP-2 recorded that the herdr CLI needs it), with `<herdr path>` `HERDR_BIN_PATH` from `Env` when set, else `herdr` (A-E5-30), and updates the E5-F4-T1 snapshot.
- Not in scope: the jump order (E5-F3-T4).
- Tests first:
  1. `ts9_4_env_prefix_names_herdr_bin_path` (CLI): given the spawn helper's `HERDR_BIN_PATH` (always `FAKE_HERDR`, 12.1), when `brain-swap install herdr` runs, then the `command` line is `command = "env HERDR_ENV=1 HERDR_BIN_PATH=<fake herdr> <brain-swap>"` with both paths replaced through `str::replace`.
  2. `ts9_4_env_prefix_without_herdr_bin_path_uses_herdr` (unit, `install_herdr`'s command builder): given an `Env` without `herdr_bin_path`, then the command is `env HERDR_ENV=1 HERDR_BIN_PATH=herdr <path>`.
- DoD:
  - [ ] Either the task is closed with the SP-2 line cited in the commit message, or the tests above pass and the first was committed red (12.2 item 1).
  - [ ] When enabled, the E5-F4-T1 snapshot shows the prefix and TECHSPEC 9.4 shows the same command form.

### E5-F4-T3 README herdr section

- Status: todo
- Depends on: E5-F4-T1, E5-F4-T2
- Covers: FR-64; TECHSPEC 9.4 (a second binding with `--board home`, why `prefix+alt+b`), 12.2 item 5
- Size: S
- Scope: Add `## herdr key binding` to `README.md`: run `brain-swap install herdr`, paste the snippet into herdr's config, run `herdr server reload-config`; the default key and why (`prefix+b` is herdr's sidebar toggle, SP-7), how to change it if it collides; a second binding whose `command` ends in ` --board home` on another key opens the home board; `install herdr` never edits herdr's config; outside herdr everything works except the jump. The first `toml` block of the section is the snippet for the placeholder path `/home/you/.cargo/bin/brain-swap` (A-E5-31). Tests go in `tests/docs.rs`.
- Not in scope: `install claude` documentation (E3-F2); the release README pass (E6-F1).
- Tests first:
  1. `fr64_readme_snippet_matches_install_output` (docs): given `README.md`, when the first `toml` fenced block under `## herdr key binding` is extracted, then it equals `key_snippet("/home/you/.cargo/bin/brain-swap")` (as E5-F4-T2 left the command form).
  2. `fr64_readme_second_binding_opens_board_home` (docs): given the same section, when its second `toml` block is parsed, then `keys.command[0].command` ends with ` --board home` and its `key` differs from the first block's.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the README section turned it green (12.2 item 1).
  - [ ] The docs test of E1-F1-T10 passes (no U+2014 or U+2013 in `README.md`).

## Assumptions

- A-E5-02 Does `capture` take the `Runner` as a second argument (`capture(env, runner)`), so the SP-4 moved-pane check needs no signature change, with E2-F3-T3 passing the runner it already holds for `resolve_session`? Affects E5-F1-T7, E5-F1-T9.
- A-E5-03 Is the fake herdr script's argv log the file `$HOME/fake-herdr.log`, one line per call with the arguments joined by single spaces, read through `tests/common/fake_herdr.rs` `calls(home)`, since the 12.1 helper passes no variable that could name a log path? Affects E5-F1-T5 and every argv assertion in E5-F2 and E5-F3.
- A-E5-04 Are scenarios POSIX `sh` files `tests/fake_herdr/scenarios/<name>.sh` matched on the joined argv with `case`, using the helpers `out`, `fail`, `hang` and `slow`, an unmatched call failing with `fake herdr: unexpected call: <argv>`, and named in kebab case as E2-F3 names them (`pane-session`), while E3-F1's `pack_two_panes` keeps its name? Affects E5-F1-T5, E5-F2-T6 to E5-F2-T9, E5-F3-T5.
- A-E5-05 Is `FakeRunner` a public type of `adapters::runner` compiled in every build (no new cargo feature), so that tests under `tests/` and the TUI runtime tests can use it? Affects E5-F1-T4.
- A-E5-06 Are the in-process `ProcessRunner` tests hermetic in the 12.1 sense when the library starts a temporary script, named through `Env.herdr_bin_path`, that reads no environment variable, although such a child is not started by the spawn helper? Affects E5-F1-T1, E5-F1-T2, E5-F1-T3.
- A-E5-07 Is the herdr log line `<stamp> herdr <args> exit=<code> ms=<n>`, with `exit=timeout`, `exit=spawn_failed` or `exit=signal` for calls without an exit code? Affects E5-F1-T3.
- A-E5-08 Does `pane_session` return herdr's `agent_session.value` unvalidated, leaving the I8 check to E2-F3-T4's `resolve_session` (E1-F7 is not a dependency of E5-F1)? Affects E5-F1-T8.
- A-E5-09 Inside herdr with `HERDR_PANE_ID` unset or empty, do `capture` and `pane_session` return nothing without a call, and are an empty `HERDR_TAB_ID` or `HERDR_WORKSPACE_ID` stored as absent? Affects E5-F1-T7, E5-F1-T8.
- A-E5-10 Are herdr's JSON key names those the SP-4 samples show, starting from the author's herdr 0.8.2 notes (`result.pane` with `pane_id`, `tab_id`, `workspace_id`, `agent_status` and `agent_session.value`; the lists under `result`), and does a missing `agent_status` count as `unknown` in the `LivePane.agent` rule? Affects E5-F1-T5, E5-F1-T6, E5-F2-T1, E5-F2-T3.
- A-E5-11 Is E2-F3 written after E5-F1-T9 (it depends on all of E5-F1), reading the TECHSPEC 16 SP-4 line, so its argv expectations (E2-F3-T3 test 2, E2-F3-T4 tests 6, 7 and 9) already allow the extra `pane get` call and E5-F1-T9 touches no E2 test? Affects E5-F1-T9.
- A-E5-12 In `locate`, is a spawn failure `HerdrError(<os error>)`, an exit 0 whose pane cannot be read `HerdrError("pane get: unreadable answer")`, a failure with empty stderr `HerdrError("exit <code>")`, and otherwise the first stderr line kept verbatim even when it is JSON (so the CLI may print `herdr: {"error":...}`)? Affects E5-F2-T1, E5-F2-T2.
- A-E5-13 Does a live pane whose JSON lacks a tab or workspace take the stored ones (`Live`) or the listed workspace (`Moved`), else an empty string? Affects E5-F2-T1, E5-F2-T3.
- A-E5-14 Does `locate` always wrap its runner in its own 3 second `Budget`, while `jump` and the TUI wrap `locate` and `focus` in one more, so that nested budgets give each call `min(2 s, the smallest rest)`? Affects E5-F2-T4, E5-F2-T8, E5-F3-T3.
- A-E5-15 Do `locate` and `jump` report an unknown card with E1-F5's `<ID> not found` (`W-99 not found` for W-99; exit 3, `not_found`), passed through unchanged, the same text as `show` (E2-F2-T4), since E1-F5-T4 now uses that text? Affects E5-F2-T6, E5-F2-T8.
- A-E5-16 In the `locate` and `jump` JSON, is `pane` the live pane for `live` and `moved`, else the recorded pane (null when none), with no field for the `HerdrError` message, since 6.7 lists only `status`, `pane`, `cwd` and `focused`? Affects E5-F2-T6, E5-F2-T8, E5-F2-T9.
- A-E5-17 Does a `--note <n>` that has no place locate as `no pane recorded` (or `not in herdr`) with the line `Where: none` and JSON `cwd` null, matching E2's `Where: none` (A-E2-16)? Affects E5-F2-T7.
- A-E5-18 Is the literal 6.3 form `W-12 has 1 notes` acceptable for a card with one note, and `W-11 has 0 notes` for a card without notes? Affects E5-F2-T7.
- A-E5-19 Does a focus that timed out reach the TUI as `NotJumped(PaneStatus::NotResponding, cwd)`, since 7.1's `Outcome` has no variant for it? Affects E5-F3-T2, E5-F3-T3.
- A-E5-20 Does the Message take the card ID for `open a session there and run /bs-link <ID>` from the mode the jump started in (the Detail card, or the Board selection), since `NotJumped` carries no ID? Affects E5-F3-T2.
- A-E5-21 Is the jump Message one item per line (status, the bare working directory as AS-4 shows it, then the hint), the directory line omitted when the note has none, and is the `jumping...` status cleared when the outcome arrives? Affects E5-F3-T2.
- A-E5-22 On the board, does `jump` on a card without a jump note open the Message `no note yet` (not the status line), and does `jump` with no card selected do nothing? Affects E5-F3-T1.
- A-E5-23 Does E4-F1 define `Effect` and `Outcome` without the jump variants (its Dep does not reach `PaneStatus` in E5-F2), leaving E5-F3-T1 to add `Effect::Jump(CardId, usize)`, `Outcome::Jumped`, `NotJumped` and `FocusFailed`, with the `usize` the index into `Card.notes` (0 the oldest) as `jump_note()` returns it? Affects E5-F3-T1, E5-F3-T2, E5-F3-T3.
- A-E5-24 Does the TUI runtime take the `Runner` as a `runner: Box<dyn Runner>` field of E4-F1's injectable Parts (terminal guard, events, clock), added by E5-F3-T3 whether or not E4-F5 has added `launcher`, so its tests run with `FakeRunner`? Affects E5-F3-T3, E5-F3-T4.
- A-E5-25 If SP-2 takes the section 14 fallback (exit first, then a detached `brain-swap jump <ID>`), is E5-F3-T4 closed and a new task filed once TECHSPEC 7.7 describes it, noting that a detached spawn from `src/tui/` also needs a layering exception and `--note <n>` for a Detail jump? Affects E5-F3-T4.
- A-E5-26 May the manual runs park with Claude Code's `!` shell escape (`! brain-swap park --card W-12`, parts on stdin) instead of `/bs-park`, so they need no E3 dependency while the session still comes from herdr? Affects E5-F3-T6, E5-F3-T7.
- A-E5-27 Is AS-11 checked by pausing the real herdr server with `kill -STOP` while the TUI runs in a plain terminal with `HERDR_ENV=1` and the pane's `HERDR_SOCKET_PATH`, since a stopped server also freezes herdr's own display? Affects E5-F3-T7.
- A-E5-28 Does `install herdr` print, after the snippet, a blank line and `then run: herdr server reload-config`, with the JSON `snippet` holding only the six TOML lines? Affects E5-F4-T1.
- A-E5-29 Is the binary path written into `command` escaped as a TOML basic string (`\` and `"`), so a path with a quote still yields valid herdr config? Affects E5-F4-T1.
- A-E5-30 With the SP-2 `env` prefix, is `<herdr path>` taken from `HERDR_BIN_PATH` when set, else the bare `herdr`? Affects E5-F4-T2.
- A-E5-31 Does the README herdr section show the snippet for the placeholder path `/home/you/.cargo/bin/brain-swap`, checked against `key_snippet` by a docs test, plus a second binding for `--board home`? Affects E5-F4-T3.
- A-E5-32 Does `install herdr` neither read nor write `config.toml` (A-E2-01), although FR-66 says any invocation without a config writes the default one? Affects E5-F4-T1.
- A-E5-33 Does `install herdr` take the binary path from `Env.exe` (the canonical `current_exe`, read in `src/main.rs`), and fail with exit 1, `Error::Io("cannot determine the brain-swap binary path")`, when `main` could not resolve it? Affects E5-F4-T1, E5-F4-T2.
