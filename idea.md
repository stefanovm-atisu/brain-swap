# brain-swap

Snapshot of the idea at the time of distillation (2026-10-08). The idea is a seed, not a contract: the contracts are PRD and TECHSPEC, and where they disagree, they win. A disagreement is resolved by the next distillation.

## Idea

When you work with several AI coding sessions at once, every switch between them is expensive: coming back, you have to rebuild in your head what you were doing, what comes next and what to watch out for. Rebuilding that from a long chat transcript is slow, deliberate work ("thinking slow" in Kahneman's terms). Reading a short cue you left yourself is fast recognition ("thinking fast").

brain-swap is a keyboard-driven kanban board in the terminal, in the spirit of taskell, built around that cue. Each card is a task; each time you leave a session you park a two or three line note on the card, and when you come back you read it first and are calibrated in seconds. Boards are plain markdown folders you own. It is built for one developer (the author) who runs AI agents in parallel inside the herdr terminal multiplexer, with Claude Code as the first-class integration.

## Principles

1. **The re-entry cue comes before the board.** The note and the return path are the product; the kanban is how you look at them. When the two compete for time or design, the cue wins.
2. **Parking and returning cost almost nothing.** The common path is one command with zero picks. A feature that adds a step to park or return needs a very good reason.
3. **Plain markdown and a tool-agnostic core.** Boards are human-readable markdown folders. Claude Code and herdr are optional adapters that use only the public CLI.
4. **Keyboard first, your keys.** Everything is reachable with vim-like keys, and every binding can be changed in settings.

## User flows

### Flow 1: start, park, switch, return

1. The user starts a Claude session in a herdr pane and types `/bs-card migrate invoices to v13`. Claude picks a card template (Feature), fills in its fields, and the CLI creates card `W-12` in Todo on the work board. From now on the session references W-12.
2. The user works through several prompts and moves the card to Doing in the TUI (or asks Claude to).
3. The user decides to switch and types `/bs-park`, optionally with their own words (`/bs-park next: rerun migration test, watch timeout`). The note is appended to W-12's timeline as Doing / Next / Watch out, marked "auto" if Claude drafted it. Claude confirms in one line: "parked to W-12: migrate invoices".
4. The user moves to another herdr pane, opens session B and works on another card the same way (`/bs-card`, or `/bs-link W-7` for an existing card).
5. To return via the board, the user opens it with a herdr key. Each card shows its ID, its latest note as a one-line preview and the note's age ("12 min").
6. The user opens W-12. The detail view shows the newest note first; `j`/`k` move to older and newer ones.
7. The user presses Enter. herdr focuses the pane where that note was written; the user is back in session A, already calibrated.
8. To return via the chat instead, the user jumps to the pane with herdr and types `/bs-back`. Claude prints W-12's latest note immediately.

### Flow 2: when things go wrong

1. **Forgot to park.** The user returns and types `/bs-back`. Claude sees that W-12's latest note is older than the recent conversation (or there is no note), says "no note since 40 min ago", drafts a catch-up note from the conversation, saves it marked "auto" and prints it.
2. **Reference lost** (after `/clear`, a compacted context or a new session). The user types `/bs-park` or `/bs-back`. Claude shows the default board's open cards with its best guess on top and "other board..." at the bottom; the user picks one, and it becomes the session's reference again. `/bs-link W-12` restores it directly.
3. **Pane gone.** The user presses Enter on W-12 on the board. The board says "pane closed" and shows the working directory where the last note was written. The user reopens a session there and runs `/bs-link W-12`.
4. **Outside herdr.** Everything works the same except the jump: Enter on a card shows the working directory instead of switching panes.

## Scale

- 2-3 AI sessions run in parallel.
- A switch happens every 15-30 minutes.
- Returns to a parked task mostly happen within the hour.
- Up to about 10 open cards on the work board.
- Three boards (work, home, personal): work is used daily, the others weekly or less.

## Boundaries

**Core**
- A TUI and a CLI over named boards, each a folder of markdown files; a default board (work) used when none is named.
- Free-form columns per board, as in taskell (default Todo / Doing / Done); cards move with vim-like keys; parking never moves a card.
- Cards with a visible short ID (board letter plus number, e.g. `W-12`), created from templates (Feature, Bug, Research, Chore) in the TUI or from Claude.
- A timeline of switch notes per card in the Doing / Next / Watch out format; notes Claude drafted are marked "auto"; a detail view opens on the newest note with `j`/`k` navigation.
- Configurable keybindings in a settings file.
- A Claude Code pack shipped in this repo: `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`. A session keeps a card reference; with a reference, park and back act directly, and without one they show the default board's cards with the best guess on top.
- Catch-up on return: `/bs-back` detects a missing or stale note and drafts one.
- A herdr adapter: a key binding opens the board, notes remember their pane, and Enter on a card jumps to it.

**Later**
- WIP limit on active cards.
- Packs for other agents (Codex, Aider and others) on the same CLI.

**Out**
- A "needs you" signal for agents waiting on input: herdr already shows it.
- Reopening a closed pane or resuming a Claude session from a card.
- Background auto-parking after every Claude turn.

## Stack and environment

- Rust, with `ratatui` for the TUI; one binary serves as both the TUI and the CLI.
- Boards are plain markdown files on disk, each board a folder (typically its own git repo).
- Claude Code, through the optional `bs-` command pack.
- herdr, through its CLI and key bindings, as an optional adapter.
