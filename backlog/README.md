# brain-swap backlog

Status: draft for review, derived from `PRD.md` and `TECHSPEC.md` (both "Draft for review", 2026-10-08). Where this backlog and those documents disagree, the documents win and the backlog is corrected; where the documents disagree with `idea.md`, see `CLAUDE.md`.

The backlog splits the work breakdown of TECHSPEC section 15 into tasks that a sub-agent can take on its own, test first, in parallel with other agents. Epic and feature IDs are those of TECHSPEC 15, so its traceability matrix (15.2) still holds; tasks add a third level.

## Files

| File | Epic | Milestone | Features | Tasks |
|---|---|---|---|---|
| `E0-spikes.md` | E0 Spikes | M0 (E0-F3 in M2) | 3 | 10 |
| `E1-core.md` | E1 Core | M1 | 8 | 49 |
| `E2-cli.md` | E2 CLI | M1 | 3 | 24 |
| `E3-claude-pack.md` | E3 Claude pack | M2 | 2 | 17 |
| `E4-tui.md` | E4 TUI | M3 | 5 | 31 |
| `E5-herdr.md` | E5 herdr adapter | M1 (E5-F1), M4 | 4 | 28 |
| `E6-release.md` | E6 Release | M5 | 1 | 10 |
| `E7-later.md` | E7 Later | none | 7 | 0 |
| | total | | 33 | 169 |

## How to read a task

Every task has the same labelled lines:

- `Status`: `todo`, `doing`, `done` or `blocked <why>`. An agent that takes a task sets `doing` in the same commit that starts the work, and `done` in the commit that finishes it.
- `Depends on`: task IDs, or feature IDs meaning every task of that feature. A task may start when every dependency is `done`. Order is defined by these lines, not by the task number.
- `Covers`: the PRD requirements (FR, NFR) and TECHSPEC sections the task implements. Read those sections before starting; the task text does not repeat them.
- `Size`: S under two hours, M half a day, L a day, for one focused agent.
- `Scope` and `Not in scope`: what to build and what to leave to the named task.
- `Tests first`: the tests to write, in the order to write them, each one behaviour, each named `<requirement>_<behaviour>` (`fr16_park_appends_only_to_timeline`). Spikes and manual checks have a `Procedure` instead.
- `DoD`: task-specific checks on top of the inherited definition of done in TECHSPEC 12.2 (failing test first, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-features`, snapshots reviewed, no new crates, layering and docs tests, one commit per task with no AI attribution).

Each feature lists `Provides` (the public items it introduces, with module paths) and `Requires` (what it calls from other features). These are the contracts between agents working in parallel: a task that needs something from another epic finds the name there.

## Test discipline

TECHSPEC 12.2 item 1 and the three laws of TDD apply to every task: write the first test of the list, see it fail (not compiling counts), write only enough code to pass, refactor, take the next test. Tests are hermetic (TECHSPEC 12.1): processes spawn through the one helper, core code receives `Env` and never reads the environment, clocks come from `BRAIN_SWAP_NOW`, files live in temp dirs, herdr and Claude Code are faked. One concept per test, assertions over logging, snapshots through `insta` and reviewed with `cargo insta review`.

## Taking a task

1. Pick the lowest wave with a `todo` task whose dependencies are all `done`; prefer the lane with the fewest agents on it.
2. Branch from `main`: `task/<task-id>-<short-title>` (`task/E1-F2-T1-frontmatter`).
3. Read the task, its feature's `Provides` and `Requires`, and the spec sections in `Covers`.
4. Work through `Tests first` in order. Add nothing the task does not ask for; if the spec is silent, choose the smallest behaviour and note it in the commit message.
5. Tick the DoD, set `Status: done`, commit (one commit per task, message says what and why), open the merge request against `main`.

## Execution order

Waves are computed from the `Depends on` lines (a task in wave n has its longest dependency chain of length n). Tasks in one wave are independent of each other and can run in parallel. The lanes of TECHSPEC 15.1 group them by area.

| Wave | Tasks |
|---|---|
| 0 | E0-F1-T1 E0-F1-T2 E0-F1-T3 E0-F1-T4 E0-F2-T1 E0-F2-T2 E0-F2-T3 E1-F1-T1 |
| 1 | E1-F1-T2 E1-F1-T3 E1-F1-T4 E1-F1-T9 E1-F1-T10 E1-F3-T4 E5-F1-T6 |
| 2 | E1-F1-T5 E1-F1-T6 E1-F2-T1 E5-F1-T5 |
| 3 | E1-F1-T7 E1-F1-T8 E1-F2-T2 E1-F2-T3 E1-F3-T1 E1-F4-T1 E1-F5-T1 E1-F7-T1 E1-F7-T4 E5-F1-T1 |
| 4 | E1-F2-T4 E1-F2-T5 E1-F2-T6 E1-F3-T2 E1-F3-T5 E1-F4-T2 E1-F6-T1 E1-F7-T2 E5-F1-T2 E5-F1-T4 |
| 5 | E1-F2-T7 E1-F3-T3 E1-F3-T6 E1-F4-T3 E1-F5-T2 E1-F6-T2 E1-F7-T3 E5-F1-T3 E5-F1-T7 E5-F1-T8 E5-F2-T1 |
| 6 | E1-F2-T8 E1-F5-T3 E2-F1-T1 E5-F1-T9 E5-F2-T2 |
| 7 | E1-F5-T4 E1-F8-T1 E2-F1-T2 E3-F1-T1 E4-F1-T1 E5-F2-T3 |
| 8 | E1-F5-T5 E1-F6-T3 E1-F6-T4 E1-F6-T5 E1-F8-T2 E2-F1-T3 E2-F1-T4 E4-F1-T2 E5-F2-T4 |
| 9 | E1-F6-T6 E1-F6-T7 E1-F6-T10 E1-F8-T3 E4-F1-T3 E5-F2-T5 E5-F2-T6 E5-F4-T1 |
| 10 | E1-F6-T8 E4-F1-T4 E4-F1-T5 E4-F2-T1 E4-F3-T1 E5-F2-T7 E5-F4-T2 |
| 11 | E1-F6-T9 E4-F1-T6 E4-F1-T9 E4-F2-T2 E4-F3-T2 E4-F3-T3 E5-F2-T8 E5-F4-T3 |
| 12 | E2-F2-T1 E4-F1-T7 E4-F1-T8 E4-F2-T5 E4-F5-T1 E5-F2-T9 |
| 13 | E2-F2-T2 E2-F2-T3 E2-F2-T4 E4-F1-T10 E4-F2-T3 E4-F3-T4 E4-F4-T1 E4-F4-T4 E4-F5-T2 E4-F5-T3 E4-F5-T4 |
| 14 | E2-F2-T5 E2-F2-T6 E4-F2-T4 E4-F4-T2 |
| 15 | E2-F2-T7 E2-F3-T1 E4-F2-T6 E4-F4-T3 |
| 16 | E2-F2-T8 E2-F3-T2 E2-F3-T3 E2-F3-T11 E2-F3-T12 E4-F5-T5 E5-F3-T1 |
| 17 | E2-F3-T4 E2-F3-T6 E4-F5-T6 E4-F5-T7 E5-F3-T2 |
| 18 | E2-F3-T5 E2-F3-T8 E5-F3-T3 |
| 19 | E2-F3-T7 E5-F3-T4 E5-F3-T5 |
| 20 | E2-F3-T9 E5-F3-T6 E5-F3-T7 |
| 21 | E2-F3-T10 |
| 22 | E3-F1-T2 E6-F1-T2 E6-F1-T5 |
| 23 | E3-F1-T3 E6-F1-T3 |
| 24 | E3-F1-T4 E3-F1-T5 E6-F1-T4 |
| 25 | E0-F3-T1 E3-F1-T6 E3-F1-T7 E3-F1-T8 E3-F1-T9 E3-F1-T10 |
| 26 | E0-F3-T2 E3-F2-T1 |
| 27 | E0-F3-T3 E3-F2-T2 |
| 28 | E3-F2-T3 E3-F2-T4 |
| 29 | E3-F2-T5 E3-F2-T7 |
| 30 | E3-F2-T6 |
| 31 | E6-F1-T1 |
| 32 | E6-F1-T6 |
| 33 | E6-F1-T7 E6-F1-T8 E6-F1-T9 |
| 34 | E6-F1-T10 |

## Assumptions

Choices this backlog made that the documents leave open are listed at the end of each epic file under `## Assumptions` (`A-E1-01` and so on), as yes/no questions.
