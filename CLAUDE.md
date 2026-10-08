# brain-swap

Keyboard-first terminal kanban in Rust whose real product is the per-card switch note that cuts the cost of returning to a parked AI session. Read `idea.md` before any product or design work.

## Documents and precedence

- `idea.md`: the distilled idea (problem, principles, user flows, scale, boundaries). It is a seed, not a contract.
- `PRD.md` and `TECHSPEC.md` (not written yet) are the contracts. Where they disagree with `idea.md`, they win; the disagreement is resolved by the next distillation.
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

Rust, `ratatui` for the TUI, one binary serving as both TUI and CLI. No code is scaffolded yet.
