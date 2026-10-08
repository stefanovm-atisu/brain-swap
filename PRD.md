# brain-swap: Product Requirements

## 1. Status

- Product: brain-swap, a keyboard-first terminal kanban built around the per-card switch note.
- Status: Draft for review.
- Date: 2026-10-08.
- Derived from idea.md (2026-10-08 snapshot); where this document and idea.md disagree, this document wins and the disagreement is resolved by the next distillation.
- Companion: TECHSPEC.md implements these FR and NFR IDs.
- Tags: Core ships in v0.1, Later is deferred. Traces: `F1.3` is idea.md Flow 1 step 3, `F2.1` Flow 2 case 1, `AS-n` section 6, `P1` to `P4` the principles, `B` the idea.md Boundaries.

## 2. Problem, goals and non-goals

### Problem

A developer running two or three AI coding sessions in parallel pays a re-entry cost at every switch: rebuilding what was being done, what comes next and what to watch out for, usually from a long transcript. That is slow work; reading a short cue written when leaving is fast recognition. Nothing today captures that cue, keeps it with the task, and leads back to the right pane.

### Goals

- G1. Re-enter a parked session calibrated in seconds by reading a short note instead of a transcript.
- G2. Park with one command and zero picks on the common path.
- G3. Get back to the right herdr pane with a few keys, or get one plain next step when it is gone.
- G4. Keep all data in plain markdown folders the user owns, edits by hand and versions with git.
- G5. Work fully without Claude Code and herdr; they add speed, not function.

### Non-goals

From idea.md "Out": a "needs you" signal for agents waiting on input (herdr shows it); reopening a closed pane or resuming a Claude session from a card; background auto-parking after every Claude turn (brain-swap installs no hook, and Claude cannot invoke the pack commands on its own).

Also excluded: tmux or any multiplexer but herdr; several users, sync, network services, a daemon, a web UI; due dates, priorities, tags, search, time tracking; an in-TUI form editor; Windows.

## 3. Users and context

One user: the author, on Arch Linux, running AI coding agents in parallel in herdr 0.8.2 with Claude Code 2.1.294, one session per pane, fluent in vim-like keys. herdr's Claude integration reports each pane's Claude session ID to herdr.

Scale, from idea.md:

- 2-3 AI sessions in parallel.
- A switch every 15-30 minutes (an estimated 15-30 parks a day).
- Returns to a parked task mostly within the hour.
- Up to about 10 open cards on the work board.
- Three boards: work (daily), home and personal (weekly or less).

Consequences: no search, database or index; a board is read in full on every command. Performance targets assume about 300 card files per board after a year (an estimate: a few done cards per week).

## 4. Principles

Priority disputes are settled by these, in this order.

1. **The re-entry cue comes before the board.** The note and the return path are the product; the kanban is how you look at them. When the two compete for time or design, the cue wins.
2. **Parking and returning cost almost nothing.** The common path is one command with zero picks. A feature that adds a step to park or return needs a very good reason.
3. **Plain markdown and a tool-agnostic core.** Boards are human-readable markdown folders. Claude Code and herdr are optional adapters that use only the public CLI.
4. **Keyboard first, your keys.** Everything is reachable with vim-like keys, and every binding can be changed in settings.

## 5. Concepts and glossary

- **Board**: a named folder of card files, typically a git repo: `work` (default), `home`, `personal`. The config maps names to folders; an optional `board.md` holds the letter, columns and next number.
- **Column**: a free-form stage, default Todo, Doing, Done. The last is the done column; an **open card** is in any other.
- **Move**: a change of a card's column (`H`/`L`, `brain-swap move`). The position inside a column is derived from activity (FR-09) and is not a move.
- **Card**: one task in one markdown file: title, column, template name, creation time, body, timeline.
- **Card ID**: board letter, hyphen, number (`W-12`, `H-3`, `P-7`); monotonic per board, never reused. The file name is the ID.
- **Template**: a markdown skeleton whose `##` headings are its fields: Feature, Bug, Research, Chore.
- **Switch note** (note): the cue left when leaving: Doing, Next, Watch out, a timestamp, an `auto` marker when an agent drafted any part, and a place.
- **Place**: where a note was written: working directory, herdr pane, tab and workspace, session ID. The pane ID is the **pane reference**, the jump target.
- **Timeline**: a card's notes in written order at the end of its file; the last is the newest.
- **Session reference**: the card a session (one agent conversation, opaque ID) works on, stored outside the boards.
- **Jump**: Enter on a card or note: focus the pane of its place (or that pane's tab when no agent runs in it), or show its working directory.
- **Picker** and **best guess**: the numbered list of open cards shown when a session has no reference, likeliest first (FR-38).
- **Adapter**: optional integration through public CLIs only: the herdr adapter and the Claude Code **pack** (`/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`).

## 6. User flows as acceptance scenarios

Setup unless stated: herdr runs, the pack and the herdr key binding (default `prefix+alt+b`; the author's prefix is `ctrl+a`) are installed, the work board holds W-1 to W-11.

### AS-1: start, park, switch, return (Flow 1)

1. (F1.1) In pane `w4V:p9` the user types `/bs-card migrate invoices to v13`. Claude asks nothing, picks Feature, fills its fields and runs one `brain-swap new`. `W-12.md` appears in Todo; Claude replies `created W-12: migrate invoices to v13 (Todo, work)`. The session references W-12.
2. (F1.2) The user presses `L` on W-12 in the TUI, or asks Claude, which runs `brain-swap move W-12 Doing`. Only the `column:` line of `W-12.md` changes. Earlier in the session the user already ran `/bs-park` once, so W-12's timeline holds one older note.
3. (F1.3) The user types `/bs-park`, or `/bs-park next: rerun migration test, watch timeout`. Claude asks nothing and runs one `brain-swap park`. W-12's timeline gains a note (three parts, time, `auto`, place); the column is unchanged. Claude replies exactly `parked to W-12: migrate invoices to v13`.
4. (F1.4) In pane `w4V:p11`, session B runs `/bs-card ...` or `/bs-link W-7` (replies `linked to W-7: <title>` and W-7's latest note). Session A's reference is untouched.
5. (F1.5) The herdr board key opens the work board in a popup. In Doing, W-12 reads `W-12 migrate invoices to v13` over `12 min  rerun migration test`; the preview panel shows its whole latest note.
6. (F1.6) The user presses `o` on W-12. The detail view shows the newest note, `note 2/2`; `j` shows the older one, `k` the newest again.
7. (F1.7) The user presses Enter. The popup closes and herdr focuses pane `w4V:p9`, where that note was written.
8. (F1.8) Alternatively, the user switches to `w4V:p9` with herdr and types `/bs-back`. Claude prints, without running a tool, `W-12 migrate invoices to v13 (note 12 min old)` and the three parts.

### AS-2: forgot to park (F2.1)

W-12's latest note is 40 minutes old and the conversation shows later work. On `/bs-back`, Claude replies `no note since 40 min ago on W-12`, drafts a catch-up note from the conversation, saves it marked `auto`, and prints it as in AS-1 step 8, headed `(note just saved)`. With no note at all, the first line is `no note yet on W-12`.

### AS-3: reference lost (F2.2)

After `/clear` (or in a new session, or if compaction changed the session ID) the user types `/bs-park`. Claude lists the work board's open cards numbered, best guess first (W-12, marked `linked here` because the pane's previous session referenced it), `o. other board: home, personal` last, and waits. The user types `1`; Claude parks to W-12, the reference again, so the next `/bs-park` asks nothing. `o` lists another board the same way; `/bs-back` picks alike; `/bs-link W-12` needs no list.

### AS-4: pane gone (F2.3)

Pane `w4V:p9` was closed and no live pane runs W-12's last session. Enter on W-12 keeps the TUI open and shows `pane closed`, `/home/smilen/Work/ati.billing` (the note's working directory) and `open a session there and run /bs-link W-12`. The user does so; the next park records the new pane.

### AS-5: outside herdr (F2.4)

In a plain terminal all behaves as in AS-1 except the jump: Enter shows `not in herdr` and the note's working directory. Inside herdr, a note parked outside it shows `no pane recorded` and the directory.

### AS-6: TUI and CLI only, no Claude

In the TUI the user presses `n`, `f` (Feature), types a title and Enter: the next card (W-12 on the setup board) appears in the focused column, selected; `L` moves it to Doing. In a herdr pane running a plain shell (no Claude), `brain-swap park --card W-12` prompts for the three parts and replies `parked to W-12: <title>` (no `auto`). Enter on W-12 focuses the herdr tab that holds that pane.

### AS-7: hand-editing the markdown

With the TUI open, the user edits `W-12.md`: the TUI shows it within 2 seconds, W-12 still selected. A hand-made `W-20.md` holding only `# spike on caching` appears in the first column with `no note`; the next card is W-21. `column: Doign` puts a card in an extra column marked unknown. A later park only appends.

### AS-8: two sessions park at the same time

Sessions A and B park W-12 and W-7 in the same second while the TUI is open. Both succeed, nothing else changes, and the TUI shows both within 2 seconds. Two parks on one card append both notes. A park on W-12 while the TUI's editor (`e`) has W-12 open survives the editor's save.

### AS-9: first run with no config

With no config file, `brain-swap` writes `~/.config/brain-swap/config.toml` (boards work, home and personal under `~/.local/share/brain-swap/`, default work), shows `created config <path>` and an empty work board with Todo, Doing and Done. The first card creates the folder. Subcommands do the same with the notice on stderr, so `/bs-card` works on day one.

### AS-10: pane moved

The pane of W-12's newest note moved to another herdr workspace and got the ID `w7Q:p2`; its Claude session still runs. Enter on W-12 focuses `w7Q:p2` and the TUI exits; `brain-swap jump W-12` prints `pane moved: now w7Q:p2`.

### AS-11: herdr not responding

The herdr server hangs. Enter on W-12 shows `herdr not responding` and the note's working directory within 3 seconds; the TUI stays usable.

## 7. Functional requirements

### 7.1 Boards and files

- **FR-01** Core. A board is a folder holding one file per card, `<letter>-<number>.md`, in its root; brain-swap never modifies other files or subfolders. (P3)
- **FR-02** Core. The config maps board names to folders; `default_board` (initially `work`) applies when a command names none. (B)
- **FR-03** Core. An optional `board.md` stores the letter, columns and next number, defaulting to the name's first letter upper-cased, Todo, Doing, Done, and the highest number plus one. (P3)
- **FR-04** Core. A missing board folder reads as an empty board; the first write creates it. (AS-9)
- **FR-05** Core. Locks, references and logs live outside board folders; inside, brain-swap writes only card files, `board.md` and transient temp files. (P3)

### 7.2 Cards and IDs

- **FR-06** Core. A new card's number exceeds every number used before on its board (gaps allowed), and creation never overwrites a file. (F1.1)
- **FR-07** Core. A card holds a title, column, template name, creation time, free markdown body and timeline. (F1.1)
- **FR-08** Core. A card is open unless it is in the board's last column. (F2.2)
- **FR-09** Core. Within a column, cards are ordered newest activity first (latest note, else creation time). (P1)
- **FR-10** Core. Card IDs are case-insensitive (`w-12` is `W-12`) and resolve to a board by letter. (F1.4)

### 7.3 Templates

- **FR-11** Core. Built-in templates Feature, Bug, Research and Chore are markdown skeletons whose `##` headings are their fields. (F1.1)
- **FR-12** Core. A file in the user's template folder adds a template or replaces the built-in of the same name. (P3)
- **FR-13** Core. A new card gets the template skeleton, or a body the caller supplies. (F1.1)

### 7.4 Switch notes and timeline

- **FR-14** Core. A note has parts Doing, Next and Watch out, a timestamp with UTC offset, an `auto` marker when an agent drafted any part, and a place. (F1.3)
- **FR-15** Core. The place is captured without user input: working directory; herdr pane, tab and workspace inside herdr; session ID when known. (F1.7, P2)
- **FR-16** Core. Parking appends the note to the end of the card's timeline, changes nothing else, and never moves the card. (F1.3, B)
- **FR-17** Core. The timeline is in the card file in written order; the last note is the newest whatever its timestamp. (F1.6)
- **FR-18** Core. A note with all three parts empty is rejected. (P1)

### 7.5 TUI board view

- **FR-19** Core. `brain-swap` without a subcommand opens the TUI on the default board, or on `--board <name>`. (F1.5)
- **FR-20** Core. Columns sit side by side; a card shows ID and title, then the latest note's age and a one-line preview of its Next part (else Doing), or `no note`. (F1.5)
- **FR-21** Core. Ages read `now` (under a minute, or future), `N min`, `N h` or `N d`, refreshed at least every 30 seconds. (F1.5)
- **FR-22** Core. A preview panel under the columns shows the selected card's whole latest note with age, `auto`, pane and working directory. (P1)
- **FR-23** Core. External changes to the board appear within 2 seconds without a key press, keeping the selection. (AS-7, AS-8)
- **FR-24** Core. The switch-board key lists the configured boards and opens the chosen one. (B)
- **FR-25** Core. Enter on a card jumps to the place of its newest note that has one (FR-59 to FR-63). (F1.7, F2.3)

### 7.6 TUI detail view

- **FR-26** Core. The open key shows the card's detail view on its newest note; `j` goes older, `k` newer, with an indicator such as `note 2/5`. (F1.6)
- **FR-27** Core. The detail view shows the note's parts, timestamp, age, `auto` and place, then the scrollable card body. (F1.6)
- **FR-28** Core. Enter in the detail view jumps to the place of the note shown. (F1.7)

### 7.7 TUI card editing and moving

- **FR-29** Core. `H` and `L` move the selected card one column left or right, keeping it selected. (F1.2)
- **FR-30** Core. The new-card key asks for a template (one key each) and a title, then creates and selects the card in the focused column. (B)
- **FR-31** Core. The edit key opens the card in the configured editor (`editor`, else `$VISUAL`, else `$EDITOR`, else `vi`) and saves it on close, keeping notes appended meanwhile. (AS-8)
- **FR-32** Core. The help key lists every action with its current keys. (P4)

### 7.8 CLI

- **FR-33** Core. One binary is the TUI and the subcommands `new`, `park`, `context`, `show`, `ls`, `link`, `move`, `locate`, `jump`, `templates` and `install`. (Stack)
- **FR-34** Core. `park` reads `Doing:`, `Next:` and `Watch out:` lines from stdin, or prompts for each when stdin is a terminal. (P2, AS-6)
- **FR-35** Core. With `--json`, every subcommand prints exactly one versioned JSON object on stdout. (P3)
- **FR-36** Core. Exit codes are fixed and documented, distinct for not found, no reference, busy and jump not performed. (P3)
- **FR-37** Core. `context --session <id>` prints all a pack command needs (reference, card, latest note with age and author session, or the picker) and exits 0 in every expected state, reporting problems as lines. (P2, P3)
- **FR-38** Core. `ls --open --guess` orders open cards by best guess: last linked from this herdr pane, latest note from this pane, then from this directory, middle columns, the rest; newest activity first within each. (F2.2)
- **FR-39** Core. `move <ID> <column>` changes only the card's column. (F1.2)
- **FR-40** Core. `locate <ID>` reports, without changing focus, whether the target pane is live, moved, closed, outside herdr or unrecorded, with its directory; `jump <ID>` locates, then focuses. (F1.7, F2.3)
- **FR-41** Core. A session is an opaque ID the caller passes; an empty or unsubstituted one falls back to `BRAIN_SWAP_SESSION`, then herdr's agent session for the current pane, then none, with a warning, not a failure; only `link` fails (exit 1) when no session resolves, since setting a reference is its whole job. (F2.2, P3)

### 7.9 Configuration and key bindings

- **FR-42** Core. Configuration is one TOML file at the XDG config location: default board, board registry, editor, key map. (P4)
- **FR-43** Core. Every TUI action has a configurable key list: characters, named keys or combinations such as `ctrl+d`. (P4)
- **FR-44** Core. An invalid or conflicting binding only warns: the action keeps its defaults and the status line names the problem. (P2, P4)
- **FR-45** Core. An unparseable config blocks writes: commands that read it fail naming file, line and column, the TUI shows the error, and `context` reports it with exit 0. (P2)

### 7.10 Claude Code pack

#### 7.10.1 All pack commands

- **FR-46** Core. `/bs-card`, `/bs-park`, `/bs-back` and `/bs-link` run only when the user types them and act only through the CLI. (P3, Out)
- **FR-47** Core. Every pack command tells Claude to run `brain-swap move <ID> <column>` when the user asks to move the card. (F1.2)

#### 7.10.2 `/bs-card`

- **FR-48** Core. `/bs-card <words>` asks nothing, picks a template, fills its fields from the words and conversation, creates the card in the first column of the default (or named) board, links the session, and replies with one line. (F1.1)

#### 7.10.3 `/bs-park`

- **FR-49** Core. With a reference, `/bs-park` saves a note without asking anything and replies exactly `parked to <ID>: <title>`. (F1.3, P2)
- **FR-50** Core. `/bs-park <words>` keeps the user's words verbatim for the parts they cover, drafts the rest, and marks the note `auto` unless every part is the user's. A part starts at a label word `doing`, `next`, `watch` or `watch out`, with or without a colon: `next: rerun migration test, watch timeout` gives Next `rerun migration test` and Watch out `timeout`, and Doing is drafted. (F1.3)

#### 7.10.4 `/bs-back` and catch-up

- **FR-51** Core. With a reference and a current note, `/bs-back` prints the card line and the note without running a tool. (F1.8)
- **FR-52** Core. With a missing or stale note, `/bs-back` says `no note since <age> ago on <ID>` or `no note yet on <ID>`, saves an `auto` catch-up note drafted from the conversation, and prints it. (F2.1)
- **FR-53** Core. A note is stale when the conversation shows work on the card after what the note describes; Claude judges this, using the note's age and author session, and unsure means stale. (F2.1, P1)

#### 7.10.5 `/bs-link`

- **FR-54** Core. `/bs-link <ID>` sets the reference and prints `linked to <ID>: <title>` and the latest note; an unknown ID gets a one-line error and changes nothing. (F1.4, F2.2, F2.3)
- **FR-55** Core. `/bs-link` without an ID shows the FR-56 picker. (F2.2)

#### 7.10.6 No-reference picker and reference lifetime

- **FR-56** Core. Without a reference, `/bs-park` and `/bs-back` list the default board's open cards numbered by best guess, `o. other board: <names>` last; the pick becomes the reference, then the command continues. (F2.2)
- **FR-57** Core. A reference lasts as long as the session ID; a new ID (`/clear`, a new session, maybe compaction) starts unlinked, and FR-56 restores it with one pick. (F2.2)

### 7.11 herdr adapter

- **FR-58** Core. The herdr adapter is active only when `HERDR_ENV=1`; outside herdr only the jump differs (FR-62). (P3, F2.4)
- **FR-59** Core. The jump focuses the target pane with `herdr agent focus` when an agent runs in it; otherwise it focuses that pane's workspace and tab (herdr 0.8.2 cannot focus a pane by ID). On success the TUI exits. (F1.7)
- **FR-60** Core. If the pane ID is gone but a live pane runs the note's agent session (a moved pane gets a new ID), the jump focuses it and reports `pane moved: now <pane>`. (F1.7)
- **FR-61** Core. If the pane is gone, the TUI stays open and shows `pane closed`, the working directory and `open a session there and run /bs-link <ID>`. (F2.3)
- **FR-62** Core. Outside herdr, or for a note without a pane, the jump shows `not in herdr` or `no pane recorded` and the directory. (F2.4)
- **FR-63** Core. If herdr does not answer within 2 seconds, the jump shows `herdr not responding` and the working directory; the TUI stays usable. (G3)
- **FR-64** Core. `brain-swap install herdr` prints a herdr key binding that opens the board in a popup by absolute path; it never edits herdr's config. (F1.5)
- **FR-65** Later. The pane border shows the card ID through herdr pane metadata.

### 7.12 First run and installation

- **FR-66** Core. Without a config file, any invocation writes the default config, says so in one line, and continues. (AS-9, P2)
- **FR-67** Core. `brain-swap install claude` symlinks the four skills into `~/.claude/skills/` (to a versioned copy it writes, or a checkout with `--link`), never replaces what it did not create unless `--force` is given, has `--remove`, and never edits Claude settings. (P3)
- **FR-68** Core. `cargo install` yields the whole product; nothing else is needed at runtime. (Stack)

### 7.13 Deferred

- **FR-69** Later. A WIP limit on active cards. (idea.md Later)
- **FR-70** Later. Packs for other agents (Codex, Aider and others) on the same CLI. (idea.md Later)
- **FR-71** Later. Archiving done cards out of the board view.
- **FR-72** Later. Manual card order within a column.
- **FR-73** Later. Writing a note from inside the TUI.
- **FR-74** Later. Copying the working directory to the clipboard when the jump falls back.

## 8. Non-functional requirements

- **NFR-01** CLI speed. Every subcommand except `locate`, `jump` and `install` finishes within 50 ms (p95, release build, warm cache, author's machine) on a board of 300 cards with 20 notes each, excluding time waiting for herdr.
- **NFR-02** TUI speed. First frame within 150 ms on that board; key reaction within 50 ms; idle, one directory scan per second. A jump adds at most 50 ms to its herdr calls (2 seconds each at most).
- **NFR-03** No daemon. Nothing runs between invocations: no background process, service or installed hook.
- **NFR-04** No network. The binary opens no network socket; herdr is reached only through its CLI.
- **NFR-05** Optional adapters. Without herdr and Claude Code, every Core requirement but the pack and the pane switch behaves identically.
- **NFR-06** Hand-friendly files. Plain UTF-8 markdown where a note renders as its three parts; a park only appends to one timeline; a move changes or adds one line (or adds a three-line frontmatter to a card that has none); a new card adds one file and changes or adds the `next:` line of `board.md` (creating `board.md` at the first card).
- **NFR-07** Byte preservation. Bytes outside the edited line or appended block stay identical, including unknown keys, comments and line endings.
- **NFR-08** Data safety. Concurrent brain-swap processes (CLI and TUI, including the TUI's `e` editor) never lose a note or a move, no reader sees a half-written file, and a crash leaves every file parseable and no ID reused. An external editor that saves a stale buffer can drop a note parked meanwhile.
- **NFR-09** Platforms. Linux first; macOS should build and pass the tests; Windows is not targeted.
- **NFR-10** Keyboard accessibility. Every action is keyboard-reachable; defaults need no modifier beyond Shift except body scrolling; nothing is conveyed by colour alone; usable at 80x24.
- **NFR-11** Footprint. At most six runtime crate dependencies, each justified in TECHSPEC.md.

## 9. Success metrics

Measured by the author over two weeks of daily use after v0.1.

- **SM-1** Board to pane: median at most 5 seconds and 4 key presses from the herdr board key to typing in the right pane, over 10 timed returns.
- **SM-2** Zero-pick parking: at least 9 of 10 `/bs-park` calls in a day need no pick.
- **SM-3** Note coverage: at most 1 in 5 `/bs-back` returns needs a catch-up note.
- **SM-4** Calibration: in at least 8 of 10 sampled returns, the first prompt after the note continues the task without scrolling the transcript.
- **SM-5** Speed: NFR-01 and NFR-02 hold in the release build, measured with `hyperfine` on a generated 300-card board.

## 10. Release scope

v0.1 ships every Core requirement (FR-01 to FR-64, FR-66 to FR-68) and every NFR, in milestones ordered by principle 1: core files and CLI, the Claude pack, the TUI, the herdr jump. The pack and the TUI may be built in parallel.

Deferred: every Later requirement (FR-65, FR-69 to FR-74) and the A-24 commands.

Out: section 2 "Non-goals".

## 11. Assumptions and open questions

Decisions made where idea.md is silent: yes keeps one, no reopens it.

- **A-01** Is order inside a column derived from note recency (a park lifts its card to the top of its column, which is not a move in the idea.md sense), with no manual order, rather than a taskell-like `board.md` listing each column's cards? (FR-09, FR-16, FR-72)
- **A-02** Does a custom board's last column act as Done, so open means any other? (FR-08)
- **A-03** Do letter, columns and next number live in `board.md`, travelling with the board's repo, while the config only maps names to folders? (FR-03)
- **A-04** Does the first run write a config without asking, boards under `~/.local/share/brain-swap/`? (FR-66)
- **A-05** Does the one-line preview show the latest note's Next part, falling back to Doing? (FR-20)
- **A-06** Does Enter on the board jump to the newest placed note, with `o` opening the detail view, where Enter targets the note shown? (FR-25, FR-26, FR-28)
- **A-07** Does a successful jump close the TUI and with it the herdr popup? (FR-59)
- **A-08** Is a note `auto` unless all three parts are the user's own words? (FR-50)
- **A-09** Is card content edited only in an external editor? (FR-31)
- **A-10** Is there no delete or archive command, so a card is removed by deleting its file and the done column keeps growing? (FR-06, FR-71)
- **A-11** Does `/bs-link` print the latest note, making linking a return? (FR-54)
- **A-12** Can only the user, never Claude on its own, invoke the pack commands? (FR-46)
- **A-13** Is staleness judged by Claude, unsure meaning stale, with its accuracy measured in a spike? (FR-53)
- **A-14** Is the reference keyed by the Claude session ID, so `/clear` loses it and one pick, with the pane's last card on top, restores it? (FR-38, FR-57)
- **A-15** Is the Claude picker a numbered text list answered by number or ID? (FR-56)
- **A-16** Is a TUI note key Later, so TUI-only use parks from the shell? (FR-34, FR-73)
- **A-17** Do custom templates live in the config folder only, not per board? (FR-12)
- **A-18** Is the default herdr key `prefix+alt+b`, changeable in herdr's config? (FR-64)
- **A-19** Is the preview panel always shown, except under 20 rows? (FR-22)
- **A-20** Is a moved pane found through the Claude session ID herdr reports per pane? (FR-60)
- **A-21** Is the recorded working directory that of the parking process (for Claude, its shell's current directory)? (FR-15)
- **A-22** Is the pack installed as symlinks to a versioned copy the binary writes, needing no checkout? (FR-67)
- **A-23** Does a broken config stop the TUI with the error, rather than run on defaults that may write to wrong folders? (FR-45)
- **A-24** Do `doctor`, `check`, an event log with `stats`, delete, rename and column commands stay out of v0.1? (section 10)
- **A-25** Does a card created in the TUI enter the focused column, while `/bs-card` uses the first? (FR-30, FR-48)
- **A-26** Does `install claude --force` replace a foreign `~/.claude/skills/<name>`? If not, `--force` is dropped. (FR-67)

Open questions:

- **Q-01** Should the default config point the work board at an existing git repo instead of `~/.local/share/brain-swap/work`?
- **Q-02** Outside herdr, is showing the working directory enough, or should Enter also copy it (FR-74 to Core)?
