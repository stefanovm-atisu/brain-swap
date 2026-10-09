# brain-swap

Keyboard-first terminal kanban in Rust whose real product is the per-card switch note that cuts the cost of returning to a parked AI session. Read `idea.md` before any product or design work.

## Documents and precedence

- `idea.md`: the distilled idea (problem, principles, user flows, scale, boundaries). It is a seed, not a contract.
- `PRD.md` and `TECHSPEC.md` are the contracts. Where they disagree with `idea.md`, they win; the disagreement is resolved by the next distillation.
- Neither the PRD nor the tech spec may contradict the user flows in `idea.md` without the user's explicit decision.

## Principles (resolve priority disputes with these)

1. The re-entry cue comes before the board.
2. Parking and returning cost almost nothing: one command, zero picks on the common path.
3. Plain markdown and a tool-agnostic core: Claude Code and herdr are optional adapters that use only the public CLI.
4. Keyboard first, every binding configurable.

## Environment

- The terminal multiplexer is herdr (`herdr --help`), not tmux. Do not design for tmux.
- Claude Code commands shipped by this repo use the `bs-` prefix.

## Stack

Rust, `ratatui` for the TUI, one binary serving as both TUI and CLI. Crates are fixed by TECHSPEC section 13; adding one means editing that document first.

## Working the backlog

- `backlog/README.md` explains the task format; `backlog/E*.md` hold the tasks. `scripts/backlog.py` selects, shows and updates tasks and prints spec sections (`spec TECHSPEC 6.3`, `spec PRD FR-41`) so nobody reads the whole documents.
- Code is written lazily: load `.claude/skills/ponytail/SKILL.md` (full intensity) before any coding task. Shortest diff that passes the listed tests, standard library first, no speculative abstractions.
- Test first: a task's tests are committed red (`test(<ID>): ...`) before the code that turns them green (`feat(<ID>): ...`). `scripts/gate.sh <ID>` must print `gate: ok` before anything merges.
- The cloud loop (`docs/loop/LEAD.md`) runs a lead that hands tasks to the `implementer` agent and validates with `spec-reviewer`, `bug-hunter` and the gate; validated tasks merge straight to `main`, one squash commit per task.

## Writing rules

- No em dash (U+2014) or en dash (U+2013) anywhere: docs, comments, commit messages. Use a comma, colon, parentheses or a hyphen.
- Commit messages describe the change and why. They never mention an AI model, tool or session; `.claude/hooks/block-ai-attribution.py` blocks offending commits.
