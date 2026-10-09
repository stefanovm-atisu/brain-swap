# E7 Later

Milestone: none. This epic holds the requirements PRD.md marks Later (section 7.13 and FR-65) so that the traceability matrix in TECHSPEC 15.2 stays complete. Nothing here is scheduled, split into tasks or estimated. A feature leaves this epic only by a new distillation of `idea.md` (see `CLAUDE.md`, documents and precedence) that moves its requirement into release scope, after which it gets a scope, Covers, Dep and DoD line in TECHSPEC section 15 and a task split here. Spec: PRD 7.13, PRD section 11 assumptions A-01, A-10, A-16 and open question Q-02.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E7-F1 | Pane metadata | not scheduled | 0 | none |
| E7-F2 | WIP limit | not scheduled | 0 | none |
| E7-F3 | Other packs | not scheduled | 0 | none |
| E7-F4 | Archive | not scheduled | 0 | none |
| E7-F5 | Manual order | not scheduled | 0 | none |
| E7-F6 | TUI notes | not scheduled | 0 | none |
| E7-F7 | Clipboard | not scheduled | 0 | none |

## E7-F1 Pane metadata

- Depends on: not scheduled
- Covers: FR-65
- Spec: PRD 7.11 FR-65; TECHSPEC 9.5
- Scope: the herdr pane border shows the card ID of the pane's current reference through `herdr pane report-metadata`. Deferred as an optional nicety; the jump and the reference store do not need it.
- Provides: none yet
- Requires: E5-F1, E1-F7 when scheduled
- Feature DoD: not scheduled

## E7-F2 WIP limit

- Depends on: not scheduled
- Covers: FR-69
- Spec: PRD 7.13 FR-69; idea.md, Later
- Scope: a configurable limit on the number of cards in the active (non-first, non-last) columns, surfaced in the TUI and refused or warned on `move`. Deferred by idea.md.
- Provides: none yet
- Requires: E1-F3, E1-F6, E4-F2 when scheduled
- Feature DoD: not scheduled

## E7-F3 Other packs

- Depends on: not scheduled
- Covers: FR-70
- Spec: PRD 7.13 FR-70; idea.md, Later; TECHSPEC 2.2 (packs use only the public CLI)
- Scope: packs for other agents (Codex, Aider and others) built on the same section 6 subcommands, each a folder under `pack/`. Deferred by idea.md; the public CLI rule in TECHSPEC 2.2 keeps the door open.
- Provides: none yet
- Requires: E2-F3 when scheduled
- Feature DoD: not scheduled

## E7-F4 Archive

- Depends on: not scheduled
- Covers: FR-71
- Spec: PRD 7.13 FR-71; PRD A-10
- Scope: moving done cards out of the board view into an archive folder so the last column stops growing. Deferred by assumption A-10 (no delete or archive command in v0.1).
- Provides: none yet
- Requires: E1-F6, E2-F2 when scheduled
- Feature DoD: not scheduled

## E7-F5 Manual order

- Depends on: not scheduled
- Covers: FR-72
- Spec: PRD 7.13 FR-72; PRD A-01; TECHSPEC 4.5, T-04
- Scope: a stored card order within a column, overriding the recency order of FR-09. Deferred by assumption A-01 and TECHSPEC T-04 (no stored order).
- Provides: none yet
- Requires: E1-F2, E1-F5, E4-F2 when scheduled
- Feature DoD: not scheduled

## E7-F6 TUI notes

- Depends on: not scheduled
- Covers: FR-73
- Spec: PRD 7.13 FR-73; PRD A-16
- Scope: writing a switch note from inside the TUI with a key, instead of `brain-swap park` from the shell. Deferred by assumption A-16.
- Provides: none yet
- Requires: E4-F3, E1-F6 when scheduled
- Feature DoD: not scheduled

## E7-F7 Clipboard

- Depends on: not scheduled
- Covers: FR-74
- Spec: PRD 7.13 FR-74; PRD Q-02; TECHSPEC 7.7
- Scope: when the jump falls back to showing the working directory (outside herdr, closed pane), Enter also copies that directory to the clipboard. Deferred until open question Q-02 is answered.
- Provides: none yet
- Requires: E5-F3 when scheduled
- Feature DoD: not scheduled

## Assumptions

- A-E7-01 Is "not scheduled" the right state for all seven, meaning none of them enters a milestone before the first release check (E6-F1) is done?
