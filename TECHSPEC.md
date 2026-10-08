# brain-swap: Technical Specification

## 1. Status

- Document: technical specification of brain-swap v0.1.
- Status: Draft for review.
- Date: 2026-10-08.
- Implements PRD.md FR/NFR IDs; where it disagrees with PRD.md, PRD.md wins.
- IDs: `I1` to `I9` invariants (3), `R1` to `R7` guarantees (4.8), `SP-n` spikes (14), `En-Fm` features (15), `T-nn` assumptions (16).

## 2. Architecture overview

One Cargo package: library `brain_swap` with all logic, binary `brain-swap` that reads the environment once, dispatches and maps errors to exit codes (T-01).

### 2.1 Module map

```
src/main.rs      read Env; no subcommand -> tui::run, else cli::run; Error -> exit code
src/core/        tool-agnostic: no processes, no env reads, no terminal
  model.rs frontmatter.rs card_file.rs board_file.rs board.rs   types, grammar, loading
  store.rs       lock, verify, atomic write, create, move, park, merge_edit
  template.rs config.rs keys.rs session.rs guess.rs time.rs env.rs error.rs
  failpoint.rs   feature "failpoints" only
src/cli/         args.rs (clap), cmd.rs (one fn per subcommand), out.rs (text, JSON)
src/tui/         app.rs (App, Mode, update), runtime.rs (loop, poll, effects), view_*.rs, editor.rs
src/adapters/    runner.rs (Runner trait), herdr.rs (capture, pane_session, locate, focus), claude.rs
pack/claude/skills/{bs-card,bs-park,bs-back,bs-link}/SKILL.md
templates/{feature,bug,research,chore}.md
```

### 2.2 Layering and the public CLI rule

- `core` imports neither `cli`, `tui` nor `adapters`, and uses neither `std::process` nor `std::env` (only `failpoint.rs` may): the environment arrives as an `Env` value, so core code is deterministic; `tests/layering.rs` enforces this. `adapters` import only `core`. The core stores a place as opaque strings (T-02).
- The pack calls only section 6 subcommands; the herdr key runs only `brain-swap`; the herdr adapter runs only `herdr pane get`, `pane list`, `workspace list`, `agent focus`, `workspace focus` and `tab focus` and, only if SP-4 confirms it, `pane current`, never the socket. Nothing reads Claude transcripts, settings, hooks or `CLAUDE_*` variables (bar `CLAUDE_CONFIG_DIR` in `install claude`, SP-6).
- Board data lives in board folders; references, pane hints, locks and editor copies in `$XDG_STATE_HOME/brain-swap/`; the log only where `BRAIN_SWAP_LOG` points (section 11).

## 3. Domain model

```rust
pub struct CardId { pub letter: char, pub number: u32 }   // "W-12"; parse accepts "w-12"
pub struct BoardMeta { pub letter: char, pub columns: Vec<String>, pub next: u32 }
pub struct Board { pub name: String, pub path: PathBuf, pub meta: BoardMeta,
                   pub cards: Vec<Card>, pub warnings: Vec<String> }
pub struct Card { pub id: CardId, pub path: PathBuf, pub title: String, pub column: String,
                  pub template: Option<String>, pub created: Option<Stamp>,
                  pub body: String, pub notes: Vec<Note>, pub writable: bool }
pub struct Note { pub heading: String,               // raw "### ..." line
                  pub at: Option<Stamp>, pub auto: bool,
                  pub doing: String, pub next: String, pub watch_out: String,
                  pub place: Place, pub extra: Vec<String> }
pub struct Place { pub cwd: Option<PathBuf>, pub herdr: Option<HerdrPlace>, pub session: Option<String> }
pub struct HerdrPlace { pub pane: String, pub tab: Option<String>, pub workspace: Option<String> }
pub struct SessionRef { pub card: CardId, pub board: String, pub pane: Option<String>,
                        pub cwd: Option<PathBuf>, pub set_at: Stamp }
pub type Stamp = jiff::Zoned;                          // fixed offset as written in the file
// adapters::herdr
pub struct LivePane { pub pane: String, pub tab: String, pub workspace: String, pub agent: bool }
pub enum PaneStatus { Live(LivePane), Moved(LivePane), Closed, OutsideHerdr, NoPane,
                      NotResponding, HerdrError(String) }
```

Invariants:

- I1. `letter` is `A` to `Z`, equal to the board letter; `number >= 1`.
- I2. A card's file name is exactly `<id>.md`; the ID is not repeated inside.
- I3. `columns` is non-empty and unique ignoring case; a new card enters the first column unless one is given (`new --column`, the TUI's focused column); the last column is done.
- I4. After any write, `next` exceeds every existing card number.
- I5. A `column` matching no column shows in an extra column and counts as open.
- I6. A note brain-swap writes has `at`, `place.cwd` and a non-empty part.
- I7. `notes` keeps file order.
- I8. A session ID is valid when it matches `[A-Za-z0-9._-]{1,128}`; empty, or containing `$`, `{` or `}`, it is an unsubstituted placeholder (6.4).
- I9. A reference resolves only if its board is configured and its card file exists.

Derived: `jump_note()` is the last note with a `cwd` or `herdr` (FR-25); `activity()` is the latest note's stamp, else `created`, else mtime, ordering a column newest first, ties to the higher number (FR-09); `is_open()` means not in the last column (FR-08).

Stamps are parsed into `jiff::fmt::temporal::Pieces` (or a `Timestamp` plus the parsed offset) and stored as `Zoned` in `TimeZone::fixed(offset)`. They are written with `strftime("%Y-%m-%dT%H:%M:%S%:z")`, never with `Display`, which appends a bracketed zone. `Env` carries the local `TimeZone`, resolved in `main`, for new stamps and offset-less stamps (4.4).

## 4. On-disk format

### 4.1 Board folder layout

```
<board folder>/            e.g. ~/.local/share/brain-swap/work, often a git repo
  board.md                 optional board settings
  W-9.md W-11.md W-12.md   cards: ^<L>-[1-9][0-9]*\.md$, <L> the board letter
  README.md .git/ ...      ignored, never touched
  .W-12.md.bs-tmp-4711     transient, during an atomic write only
```

Card files with another letter are ignored with a warning; subfolders are not scanned.

### 4.2 Frontmatter: yes, a flat subset

Machine fields need a fixed place that a one-line splice can change without touching prose (NFR-06, NFR-07), and forges render frontmatter as a table. A flat subset is parsed by hand (T-03):

```
frontmatter := "---" NL { key ":" [" " value] NL | fm_cont NL | "#" text NL | NL } "---" NL     at byte 0 only
key := [A-Za-z0-9_-]+   value := rest of line, one pair of surrounding quotes stripped
fm_cont := an indented line, or a line starting with "- ", continuing the previous key
```

Known keys match ignoring case (`Column: Doing` is `column`). An indented line, or one starting with `- `, continues the previous key and is kept verbatim, so a hand-added YAML list (indented or not) survives. Unknown keys, comments and blank lines are kept; setting a key replaces its line or inserts one before the closing `---`. Only a line that matches none of these rules invalidates the block: an invalid block reads as none, with a warning, and blocks writes to that file.

### 4.3 board.md

Keys: `letter` (default the board name's first letter, upper-cased; one character `A` to `Z`, lower case is upper-cased, any other value warns and the derived letter is used), `columns` (comma-separated; default `Todo, Doing, Done`), `next` (floor: highest existing number plus one). Text below is the user's. brain-swap writes only `next:`, creating the file at the first card. A `board.md` created by brain-swap is exactly `---`, `next: <n>`, `---`; letter and columns stay derived until the user adds them (an R4 fixture). If `board.md` has no frontmatter (for example only `# work`), writing `next:` prepends `---`, `next: <n>`, `---`. If its frontmatter is invalid, `new` fails with exit 1, code `invalid_input`, message `board.md:<line>: <problem>`; `context` and the TUI show the same text as a warning.

### 4.4 Card file grammar

```
card     := [frontmatter] ["# " title NL] body [timeline [post]]
timeline := "## Timeline" NL { note | other-line }
note     := "### " stamp [" auto"] NL { part | cont | place | other-line }
part     := ["- "] label ":" [" " text] NL        label := Doing | Next | Watch out (any case)
cont     := "  " text NL                           continues the part above
place    := "<!-- where: " pair { " " pair } " -->" NL
pair     := key "=" (bare | '"' escaped '"')       key := cwd | pane | tab | workspace | session
stamp    := RFC 3339 with offset, e.g. 2026-10-08T10:31:05+03:00
post     := the next "# " or "## " heading after the timeline, and all below it
```

- Card keys: `column` (missing: the first column), `template`, `created`. The title is the first `# ` line outside code fences, else `(untitled)`; the body lies between title and timeline.
- The place is one HTML comment, so a note renders as three parts (NFR-06) yet stays one editable line (T-18). Values with a space, `"` or `\` are quoted with `\"` and `\\` escapes; one containing `-->` is dropped with a warning; unknown keys are kept.
- The reader also accepts bold labels and stamps `YYYY-MM-DD HH:MM[:SS]` (local zone without offset); a bad stamp shows age `?`. The writer emits a blank line, the heading, all three parts and the place line, with two-space continuations.

### 4.5 Columns and order

Columns live in `board.md`, each card's column in its frontmatter; order is derived (section 3), so a move or park never rewrites another card (T-04).

### 4.6 ID allocation and crash safety

Inside the board lock (section 10):

1. Re-read `board.md` and list card files; `n = max(next, highest number + 1)`.
2. Write `board.md` with `next: n + 1` atomically (creating it if absent, 4.3).
3. Write the card to a temp file, fsync, `std::fs::hard_link(temp, "<L>-<n>.md")`, remove the temp. If linking fails because the name exists (made by hand meanwhile), `n = n + 1` and repeat from (2), at most 100 times.

A crash after (2) only skips `n` (FR-06 allows gaps); one inside (3) leaves a dot-prefixed temp, ignored and removed after an hour. Linking never overwrites, even for a writer that skips the lock. A board folder on a filesystem without hard links is unsupported, like network filesystems (T-13): `new` fails with exit 1, code `io`, message `board <name>: hard links not supported`. Deleting cards never lowers `next`. Offline creation on two machines can collide (T-05).

### 4.7 Hand edits

Changed text shows on the next poll, never rewritten. Without frontmatter a card is in the first column. An unknown column (`Doign`) gets an extra column `Doign (unknown)` and a warning. Extra columns render after the last configured column, in order of first appearance, and count as open (I5). `H` or `L` on a card in an extra column moves it to the first configured column; no move ever targets an extra column. A hand-made card is discovered and the next ID skips past it. A note without a place shows its age normally and is no jump target; a note with a bad stamp shows age `?` and remains a jump target if it has a place. A card without `## Timeline` (removed by hand) gets one appended at the end of the file by the next park, followed by the note (R3). CRLF is kept. Non-UTF-8 files (`unreadable`) and conflict markers (`<<<<<<<` at line start) show with a warning and refuse writes.

### 4.8 Round-trip guarantees

- R1. Existing files are never re-serialised; every write is a splice.
- R2. `set_column` changes or adds one line, or adds a three-line frontmatter.
- R3. `append_note` only adds lines inside the timeline (or a new final section).
- R4. Created files are canonical and parse back to the values rendered.
- R5. Inserted lines use the file's dominant line ending, after a missing final newline.
- R6. The editor merge (7.6) writes the user's bytes, plus the notes parked meanwhile, plus the file's column line when only the file changed it.
- R7. Before each rename the new bytes are parsed against the operation's postcondition (park: one more note, and the bytes before and after the inserted block identical; move: the new column; create: the rendered values); failure writes nothing (`verify_failed`).

### 4.9 Worked example

Board `work` at 11:52 on 2026-10-08; missing numbers were cards deleted by hand.

`board.md`:

```markdown
---
letter: W
columns: Todo, Doing, Done
next: 13
---
# work
```

`W-12.md` (Feature, Doing; the second note's parts were all the user's words, so no `auto`):

```markdown
---
column: Doing
template: Feature
created: 2026-10-08T10:02:11+03:00
---
# migrate invoices to v13

## Goal
Switch invoice reads and writes to the v13 procedures.

## Acceptance

## Context
The DBA applies the procedure on staging; we hand over the SQL only.

## Timeline

### 2026-10-08T10:31:05+03:00 auto
- Doing: switched InvoiceRepository to v13, unit tests green
- Next: rerun the migration test against the staging copy
- Watch out: v13 needs a 10 s timeout entry, the default is 2 s
<!-- where: cwd=/home/smilen/Work/ati.billing pane=w4V:p9 tab=w4V:t1 workspace=w4V session=55583127-a22a-49bc-a803-e777c377a595 -->

### 2026-10-08T11:12:40+03:00
- Doing: migration test passes on staging
- Next: open the MR, ask the DBA for the v13 deploy list
- Watch out: nothing new
<!-- where: cwd=/home/smilen/Work/ati.billing pane=w4V:p9 tab=w4V:t1 workspace=w4V session=55583127-a22a-49bc-a803-e777c377a595 -->
```

`W-9.md` (Chore, Done, parked from a shell outside herdr, so the place is only `cwd`):

```markdown
---
column: Done
template: Chore
created: 2026-10-06T16:20:00+03:00
---
# bump rust-version to 1.89

## Timeline

### 2026-10-06T17:05:12+03:00
- Doing: merged, CI green
- Next: nothing left
- Watch out: nothing
<!-- where: cwd=/home/smilen/Work/private/brain-swap -->
```

`W-11.md` (Bug, Todo) is fresh: frontmatter, `# invoice PDF shows the wrong VAT`, the Bug fields and an empty `## Timeline`.

The board at 80x24 with W-12 selected (empty rows elided):

```
 work  ~/.local/share/brain-swap/work                                    ? help
 Todo (1)                  > Doing (1)                 Done (1)
 W-11 invoice PDF shows t  W-12 migrate invoices to v  W-9 bump rust-version
   no note                   39 min  open the MR, ask    1 d  nothing left
 ------------------------------------------------------------------------------
 W-12 latest: 39 min ago, pane w4V:p9, /home/smilen/Work/ati.billing
 Doing:      migration test passes on staging
 Next:       open the MR, ask the DBA for the v13 deploy list
 Watch out:  nothing new
```

Enter on W-12 targets pane `w4V:p9`; on W-9, inside herdr, it shows `no pane recorded` and the directory.

## 5. Configuration

### 5.1 Locations

XDG variables with the usual defaults, also on macOS (T-08): config `~/.config/brain-swap/config.toml`; custom templates `~/.config/brain-swap/templates/*.md`; default boards `~/.local/share/brain-swap/<name>`; installed pack `~/.local/share/brain-swap/pack/<version>/`; state `~/.local/state/brain-swap/` with `sessions/`, `panes/`, `locks/`, `edit/`.

### 5.2 Full file with defaults

Written on first run with create-new semantics, so two simultaneous first runs cannot clobber each other (FR-66).

```toml
default_board = "work"   # must be in [boards]
editor = ""              # empty: $VISUAL, $EDITOR, vi; arguments allowed ("code --wait")

[boards]                 # name = folder; names match [a-z][a-z0-9_-]*
work = "~/.local/share/brain-swap/work"
home = "~/.local/share/brain-swap/home"
personal = "~/.local/share/brain-swap/personal"

[keys]                   # action = key or list of keys
up = ["k", "up"]
down = ["j", "down"]
left = ["h", "left"]
right = ["l", "right"]
move_left = "H"
move_right = "L"
open = "o"
jump = "enter"
new = "n"
edit = "e"
switch_board = "b"
scroll_down = "ctrl+d"
scroll_up = "ctrl+u"
help = "?"
quit = ["q", "esc"]
confirm = "enter"
cancel = "esc"
```

No other keys exist (T-17); unknown keys warn. Board uses every action but `scroll_*`, `confirm`, `cancel`; Detail uses `up` (newer), `down` (older), `scroll_*`, `jump`, `edit`, `help`, `quit` (back); pickers use `up`, `down`, `confirm`, `cancel` and template hotkeys. TitleInput is a text field, not a picker: every printable character and space is inserted, `backspace` deletes the last character, `confirm` creates (ignored while the trimmed title is empty) and `cancel` returns to Board; no other action applies there, and a binding of `confirm` or `cancel` to a printable character is ignored there with a warning. Template hotkeys are checked in TemplatePick after the action keys are settled (5.3): a hotkey colliding with the final `up`, `down`, `confirm` or `cancel` keys is dropped with a warning.

A config that parses but has `default_board` not in `[boards]`, an empty `[boards]`, a board name outside `[a-z][a-z0-9_-]*` (so the derived letter is always `A` to `Z`) or a value of the wrong type is a config error, handled exactly as FR-45 (file, line and column of the offending key). A leading `~/` in a board path expands to `HOME`; any other relative path is a config error. `[boards]` keeps file order (13). The editor string (`editor`, `$VISUAL` or `$EDITOR`) is split on ASCII whitespace, with no quoting and no shell; the first word is the program and the copy's path is appended as the last argument.

### 5.3 Key binding syntax

```
binding  := { modifier "+" } key            modifier := ctrl | alt | shift
key      := one printable character (case-sensitive, "H" is shift+h)
          | enter | esc | tab | backtab | space | backspace | delete | up | down | left | right
          | home | end | pageup | pagedown | f1 ... f12
```

An unparseable binding is dropped with `key config: <action>: <problem>` in the status line, and the action keeps its defaults (FR-44). Conflicts are checked per mode on the merged map, where a user entry replaces that action's defaults. While a key is bound to two actions in one mode, the user entry of every user-set action involved is dropped (from both if the user set both), that action returns to its defaults in every mode, `key config: <action>: <problem>` names each dropped entry, and the check repeats. Defaults are conflict-free and each round drops at least one user entry, so the check ends. Swapping two keys (`open = "n"`, `new = "o"`) is therefore valid; `new = "j"` alone collides with `down` and reverts `new` to `n`. When matching, SHIFT is ignored for printable characters (crossterm reports `H` as `H` with SHIFT set); `shift+h` in the config normalises to `H`; `ctrl` and `alt` must match exactly. A test keeps the defaults conflict-free, and a `keys.rs` test matches `H` with and without SHIFT.

### 5.4 Templates

Built-ins are embedded; a user file of the same name (ignoring case) replaces one, a new name adds one (FR-12). A template is frontmatter `name` and `key`, then the skeleton, whose `##` headings are the fields. Built-ins: Feature (`f`: Goal, Acceptance, Context), Bug (`b`: Symptom, Expected, Repro, Context), Research (`r`: Question, Done when, Context), Chore (`c`: Task, Why, Context). A user template's name is its frontmatter `name`, else its file stem with the first letter upper-cased; replacement matches that name ignoring case. Without `key` it has no hotkey and is chosen in TemplatePick with `up`, `down` and `confirm`. A file without frontmatter is all skeleton. A non-UTF-8 or unreadable file, or one with invalid frontmatter, is skipped with a warning; an unreadable templates folder only warns, so `templates` exits 0. A `## Timeline` heading in a template and duplicate hotkeys are dropped with a warning. A new card is frontmatter (`column`, `template`, `created`), `# <title>`, the skeleton or supplied body, and an empty `## Timeline`.

## 6. CLI specification

### 6.1 Global

`brain-swap [--board <name>] [--json] [<subcommand>]`, both flags global. `--board` names the board for commands without a card ID (default `default_board`); card IDs are case-insensitive and resolve by letter (FR-10). `--json` prints one object, first key `"v": 1`, bumped when a field is removed or renamed (FR-35). `main` reads into `Env`: cwd, `HOME`, `XDG_*`, `VISUAL`, `EDITOR`, `HERDR_*`, `BRAIN_SWAP_SESSION`, `BRAIN_SWAP_LOG`, `CLAUDE_CONFIG_DIR` (for `install claude` only), the process id, the local time zone (section 3), and now from `BRAIN_SWAP_NOW` (RFC 3339, for tests) or the clock; `BRAIN_SWAP_FAILPOINT` only with feature `failpoints`.

### 6.2 Exit codes

0 success (also `context` in every state, `locate` for every status, `move` to the card's current column, which writes nothing); 1 runtime error (I/O, unreadable file, invalid input such as an empty note or no session, config, `verify_failed`); 2 usage, or the TUI without a terminal; 3 not found (card, board, template or column); 4 no reference (`park` without `--card`); 5 busy (no lock in 2 seconds); 6 jump not performed.

Errors go to stderr as `brain-swap: <message>`. With `--json`, stdout always carries exactly one object: the result on success, or `{"v":1,"error":{"code","message"}}` on failure; stderr then carries only warnings. Codes: `io`, `unreadable` (exit 1: the file is non-UTF-8, has conflict markers or invalid frontmatter, so writes are refused; `Error::Unreadable` maps to it), `invalid_input`, `config`, `verify_failed`, `usage`, `not_found`, `no_reference`, `busy`. Exit 6 has no error object: `jump` prints the 6.7 status object with `focused: false`, and its `status` names the reason. Warnings go to stderr as `warning: <text>`.

An unknown board, template or column name exits 3 (`not_found`) and lists the valid names: `brain-swap: unknown column 'Doign' on work (Todo, Doing, Done)`, likewise `unknown template 'epic' (Feature, Bug, Research, Chore)` and `unknown board 'work2' (work, home, personal)`. The TUI with an unknown `--board` exits 3 before entering the alternate screen. Board, template and column names match ignoring case.

### 6.3 Subcommands

**`brain-swap`** opens the TUI (FR-19); exit 2 without a terminal.

**`new [--template <name>] [--column <name>] [--session <id>] [--body -] [<title>...]`** (FR-13, FR-48). Defaults: `feature`, first column. `--body -` reads the body from stdin instead of the skeleton. A first non-blank line starting with `# ` is removed from it and becomes the title; a positional title, when given, wins and the line is still removed. A `## Timeline` heading in a supplied body is dropped with a warning, as for templates (5.4), so a created card holds exactly one `# ` title line and one timeline (R4). No title: exit 1. Prints `created W-12: migrate invoices to v13 (Todo, work)`.

**`park [--card <ID>] [--session <id>] [--auto]`** (FR-14 to FR-18, FR-34, FR-49). Card: `--card`, else the session's reference, else exit 4. The note comes from stdin (6.5) or terminal prompts; the place is captured (cwd, herdr `capture`, session) and the note appended to the timeline. Prints `parked to W-12: migrate invoices to v13`.

`new`, `park --card` and `link` set the reference of a resolved session (6.4).

**`context [--session <id>]`** (FR-37, FR-56): the pack's one render-time call; expected states never exit non-zero. Fixed line order:

```
brain-swap context v1
session: 55583127-a22a-49bc-a803-e777c377a595
reference: W-12
card: W-12 migrate invoices to v13
board: work, column: Doing
latest note: 2026-10-08T11:12:40+03:00, 39 min ago, by this session
Doing: migration test passes on staging
Next: open the MR, ask the DBA for the v13 deploy list
Watch out: nothing new
Where: /home/smilen/Work/ati.billing (pane w4V:p9)
```

`latest note:` adds `auto` when marked and ends with `by this session`, `by another session` or `no session`, or reads `latest note: none`. Without a reference (or with one to a missing card, `reference: none (W-12 not found)`):

```
brain-swap context v1
session: 9b1e04d2-5c1f-4b0e-8a57-0d3c2f6e1a90
reference: none
open cards on work, best guess first:
1. W-12 migrate invoices to v13 [Doing] 39 min, linked here: open the MR, ask the DBA for the v13
2. W-7 fix rounding in act export [Doing] 2 h: check the 0.005 case
3. W-11 invoice PDF shows the wrong VAT [Todo] no note
o. other board: home, personal
```

Picker line := `<n>. <ID> <title> [<column>] <age>[, <label>]: <preview>`, or `<n>. <ID> <title> [<column>] no note[, <label>]`; the label is that of 6.6, the preview the first line of Next, else Doing, cut at the last word boundary within 40 characters.

Problems become lines: `warning: session id not substituted` (with `session: none`), or `error: config <path>:4:7: expected '='` and `error: board work: <os error>`, each error followed by a `hint:` line.

**`show <ID> [--all]`**: the block from `card:` on; `--all` adds every note, newest first. `show` resolves no session: its `latest note:` line ends with `by session <id>` or `no session`, and JSON `by_this_session` is null. **`ls [--open] [--guess]`** (FR-38): picker lines under `cards on work:`, numbered in column order, then newest activity first, without labels; with `--guess` in 6.6 order under `open cards on work, best guess first:` plus the `o. other board:` line.

**`link <ID> [--session <id>]`** (FR-54, FR-57): prints `linked to W-12: migrate invoices to v13` and the block from `card:` on, whose `latest note:` line has the `by ...` endings of `context`, since `link` has a session; unknown ID: exit 3, nothing changes; no session: exit 1.

**`move <ID> <column>`** (FR-39): column matched ignoring case; only the `column:` line changes; prints `moved W-12 to Doing`.

**`locate <ID> [--note <n>]`** (FR-40): 9.3 without focus, on the jump note or note `n` (1 is the oldest); prints `live w4V:p9`, `moved w7Q:p2`, `closed w4V:p9`, `not in herdr`, `no pane recorded`, `herdr not responding` or `herdr: <message>`, then `Where: <cwd>`; exit 0. A card with no jump note: `locate` prints `no note yet`, exit 0, JSON status `no_note`; `jump` prints the same and exits 6. `cmd.rs` decides this before `locate` runs, so `PaneStatus` does not carry it. `--note <n>` outside 1 to the note count: exit 3, code `not_found`, message `W-12 has <count> notes`.

**`jump <ID> [--note <n>]`** (FR-59 to FR-63): `locate`, then `focus` a live or moved pane: `jumped to W-12 (pane w4V:p9)` when the pane was focused, `jumped to W-12 (tab w4V:t1; pane not focusable)` when only its tab was (no agent runs in it), or `pane moved: now w7Q:p2`, exit 0. If focus fails: `focus failed: w4V:p9` and `Where: <cwd>`, exit 6, JSON `status` unchanged with `focused: false`; if a focus call times out (9.3): `herdr not responding` and `Where: <cwd>`, exit 6, JSON status `not_responding`. Otherwise the `locate` lines, exit 6.

**`templates`** (FR-11, FR-12): per template `template: Feature (key f)` (without a key: `template: Spike`) and the skeleton. It never parses the config, so it always exits 0.

**`install claude [--link <repo>] [--force] [--remove]`** (8.8), **`install herdr`** (9.4).

### 6.4 Session identity and reference store

A command resolves its session (FR-41) from (1) `--session` when valid (I8), (2) `BRAIN_SWAP_SESSION` when valid, (3) when (1) and (2) give no valid ID, and only inside herdr, `pane_session` (9.2), for every command that takes `--session` (`new`, `park`, `link`, `context`), else (4) none. `--session` is optional on every command that takes it; a missing one resolves like an empty one, without the warning. A valid `--session` never calls herdr (T-15). An empty or placeholder value warns `session id not substituted`, another invalid one `invalid session id`; neither fails the command. A failed `${CLAUDE_SESSION_ID}` substitution (SP-1) thus degrades to herdr, and outside herdr to the picker.

References live in `$XDG_STATE_HOME/brain-swap/sessions/<id>.toml` (FR-05) with `card`, `board`, `pane`, `cwd`, `set_at`; `set_at` is stored as an RFC 3339 string written with the section 3 strftime and parsed like a note stamp (a serde `with` helper, so jiff needs no `serde` feature); writes are atomic, files older than 30 days are pruned (section 10), an unparsable file reads as none. A reference set inside herdr also writes the pane hint `panes/<pane>` (card and session), which the best guess reads as `linked here`: it recovers the card after `/clear` even when the old session never parked. The hint file name encodes the pane ID like lock names (`%` to `%25`, `/` to `%2F`); a pane ID outside `[A-Za-z0-9:._-]{1,128}` writes no hint.

### 6.5 Note on stdin

A line matching `^\s*-?\s*(\*\*)?(doing|next|watch out|watch)\s*:\s*(\*\*)?\s*(.*)$` (any case) starts that part; other non-empty lines continue it; unlabelled lines before any label belong to Doing. Three empty parts: exit 1, `empty note` (FR-18).

### 6.6 Best guess

A pure function of the open cards, the current pane (when `HERDR_ENV=1`), the cwd and the pane hint (FR-38). Each card takes its best tier: 0 `linked here` (the hint's card); 1 `same pane` (latest note with a pane is from this pane); 2 `same folder` (the card's latest note that has a `cwd` has one equal to the current cwd or lying inside it, or the current cwd lies inside that note's cwd and that cwd is neither `/` nor `$HOME`); 3 a middle column; 4 the rest. Within a tier: newest activity, then higher number. JSON `reason` is `linked_here`, `same_pane`, `same_folder`, `middle_column` or `other`; the text labels are `linked here`, `same pane` and `same folder`, and tiers 3 and 4 have none.

### 6.7 JSON objects

A Card has `id`, `board`, `title`, `column`, `open`, `path`, `latest_note`; a note has `at`, `age_min`, `auto`, `doing`, `next`, `watch_out`, `by_this_session`, `place` (`cwd`, `session`, `herdr` with `pane`, `tab`, `workspace`). A Candidate adds `rank` and `reason`. Absent values are `null`. `new`, `park`, `move`, `link` return `card`; `show` returns `card` and `notes` (every note newest first with `--all`, else the latest only); `ls` returns `board` and `cards`, and with `--guess` `cards` as Candidates plus `other_boards`; `templates` returns `templates` (each `name`, `key`, `skeleton`); `install claude` returns `linked` and `removed` (path lists); `install herdr` returns `snippet`; `context` returns `session`, `reference`, `card`, `candidates`, `other_boards`, `problem` (null, or `{"code","message","hint"}` with a 6.2 code; distinct from the failure envelope's `error`), `warnings`; `locate` and `jump` return `status` (`live`, `moved`, `closed`, `outside`, `none`, `not_responding`, `herdr_error`, `no_note`), `pane`, `cwd`, and for `jump` `focused` (`"pane"`, `"tab"` or `false`).

## 7. TUI specification

### 7.1 Structure and state machine

Elm style, so TUI tasks are built and tested without a terminal, a store or herdr:

```rust
pub struct App { pub mode: Mode, pub board: Board, pub sel: Selection, pub keys: KeyMap,
                 pub status: Option<String>, pub now: Stamp }
pub enum Mode { Board, Detail { card: CardId, note: usize, scroll: u16 },
                TemplatePick { sel: usize }, TitleInput { template: String, buf: String },
                BoardPick { sel: usize }, Message { text: String, back: Box<Mode> },
                Help { back: Box<Mode> }, ConfigError(String) }
pub enum Input { Key(KeyEvent), Tick(Stamp), Reloaded(Board), Done(Outcome) }
pub enum Outcome { Moved(CardId), Created(CardId), Edited(CardId), Jumped,
                   NotJumped(PaneStatus, Option<PathBuf>),          // status, cwd
                   FocusFailed(String, Option<PathBuf>),            // pane, cwd
                   Failed(String) }
pub enum Effect { Move(CardId, String), Create { template: String, title: String, column: String },
                  Edit(CardId), Jump(CardId, usize), LoadBoard(String), Quit }
pub fn update(app: &mut App, input: Input) -> Vec<Effect>      // pure, no I/O
```

On start and after `LoadBoard`, `sel` is the open card with the newest `activity()` (ties: higher number) and its column is focused; with no open card, the first column (FR-19, A-27). `runtime.rs` draws, polls input every 250 ms, fingerprints the folder every second (7.4), runs effects through `core::store`, `tui::editor` and `adapters::herdr`, and returns results as `Input::Done`.

Transitions (5.2 actions): Board `move_*` emits `Move` (FR-29), `open` shows Detail on the newest note (FR-26), `jump` emits `Jump` on `jump_note()` (FR-25), `new` opens TemplatePick (FR-30), `edit` emits `Edit`, `switch_board` opens BoardPick (FR-24), `help` opens Help (FR-32), `quit` emits `Quit`. Detail `down`/`up` move the note index, clamped; `jump` targets the note shown (FR-28). A template hotkey opens TitleInput, whose `confirm` emits `Create` in the focused column; BoardPick `confirm` emits `LoadBoard`; `cancel` and any key in Message or Help return. Outcomes: `Moved`, `Created` and `Edited` reload the board and select that card, with no Message (FR-29, FR-30); `Failed` (including `board busy, try again`, `verify_failed` and a non-zero editor exit) sets the status line; `NotJumped` and `FocusFailed` open the 7.7 Message; `Jumped` emits `Quit` (FR-59). A broken config starts in ConfigError; any key exits 1 (FR-45).

### 7.2 Screens

Board: header, columns, preview panel, status line (4.9). Detail: `W-12 migrate invoices to v13` with column and board; `note 2/2`, stamp and age; the three parts; `Where: <cwd> (pane <pane>)`; a separator; the scrollable body; a key hint line. Also the template and board pickers, the title input, Message, Help (every action with its keys, then all warnings) and ConfigError.

### 7.3 Card rendering, preview panel and age

- Card: `<ID> <title>`, then age and the first line of the latest Next, else Doing, else `no note` (FR-20), truncated.
- Preview panel (FR-22): a separator, `<ID> latest: <age> ago[, auto], pane <pane>, <cwd>`, then the first line each of Doing, Next and Watch out (`no note yet` without notes); hidden below 20 rows.
- Age (FR-21): under 60 s or future `now`, under 60 min `N min`, under 24 h `N h`, else `N d`, no or bad stamp `?`; every unit is floored (59 min 59 s is `59 min`, 42 h 47 min is `1 d`); redrawn on events and at least every 30 seconds.
- Selection is reverse video, the focused column header bold with `>`; nothing relies on colour. Narrow columns scroll around the focused one. Each column scrolls vertically to keep its selected card visible; `up` and `down` stop at the ends without wrapping, as do `left` and `right` at the first and last column; the header shows the count, e.g. `Done (214)`. `H` on the first column and `L` on the last configured column do nothing (a card in an extra column follows 4.7).

### 7.4 External changes

Every second the runtime fingerprints card files and `board.md` (name, mtime, length); on change it reloads and sends `Reloaded`, and `update` keeps the selection by card ID, else the nearest card of that column (FR-23, T-06).

### 7.5 Terminal handling

Alternate screen and raw mode, restored on exit, around the editor and on panic.

### 7.6 Editing through the editor

The card bytes `C0` are copied to `$XDG_STATE_HOME/brain-swap/edit/<board>-<ID>.md` and opened in the editor (`editor`, `$VISUAL`, `$EDITOR`, `vi`). On a zero exit with changes `E`, inside the lock: if the file still equals `C0`, write `E`; else write `E` plus the file's notes whose block `C0` lacks (a note's merge identity is its whole block, heading, parts and place line, since two notes parked in one second share a heading), appended at the end of E's timeline in file order (a missing `## Timeline` is re-added as in 4.7). If E's `column` equals C0's and the file's `column` differs, E's column line is replaced by the file's; otherwise E's column wins. A non-zero exit writes nothing and keeps the copy. A copy is deleted after a successful merge; a kept one is overwritten by the next `e` on that card. This is the only path writing user-edited bytes (R6, FR-31, AS-8).

### 7.7 Jump from the TUI

Target: `jump_note()` on the board, the shown note in Detail (none: `no note yet`). Showing `jumping...`, the runtime calls `locate` (9.3). On `Live` or `Moved` it focuses first: on `Ok` it restores the terminal and exits 0 through `Done(Jumped)` and `Quit`, closing the popup (FR-59, FR-60); on `Err` it stays in the TUI and opens a Message `focus failed: <pane>` (or `herdr not responding` when a focus call timed out, 9.3) with the working directory. SP-2 may move the restore before the focus; a failed focus then re-enters the alternate screen and shows the same Message. Other outcomes open a Message with the working directory: `pane closed` plus `open a session there and run /bs-link W-12` (FR-61), `not in herdr`, `no pane recorded` (FR-62), `herdr not responding`, `herdr: <message>` (FR-63).

## 8. Claude Code pack

### 8.1 Layout and common rules

Each command is a skill directory, `pack/claude/skills/<name>/SKILL.md`, so the bare names work (a plugin command would be `/brain-swap:bs-park`). Every file sets `disable-model-invocation: true` (FR-46, idea.md "Out") and `allowed-tools: Bash(brain-swap:*)` (SP-3); runs inline only `context` and `templates`, which exit 0 in every expected state, as a failing inline command aborts the skill; uses the Bash tool for the rest, so Claude reports errors; passes `--session "${CLAUDE_SESSION_ID}"` quoted (6.4 handles a failed substitution); names the card in every park; carries `metadata` (version tested against Cargo.toml); and ends with the move line (FR-47).

### 8.2 `bs-card/SKILL.md`

````markdown
---
name: bs-card
description: Create a brain-swap card for a task from a template and link this session to it.
argument-hint: "<task in a few words> [on home|personal]"
disable-model-invocation: true
allowed-tools: Bash(brain-swap:*)
metadata:
  pack: brain-swap
  version: 0.1.0
---
Create a brain-swap card for this task: $ARGUMENTS

Templates:
!`brain-swap templates`

Ask nothing. If no task is given above, take it from this conversation. Pick the template that fits best. Title: the task words above as typed, without a board phrase such as `on home`, when they are at most 8 words; otherwise a title of at most 8 words in the user's wording and casing. Never include a card ID. Fill each `##` field of its skeleton from the task and this conversation; leave a field empty rather than invent facts. Run this one command, adding `--board <name>` only if the user named a board:

```
brain-swap new --template <template> --session "${CLAUDE_SESSION_ID}" --body - <<'BS_EOF'
# <title>

<filled skeleton>
BS_EOF
```

Reply with its output line only, or on failure its first error line.

If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.
````

### 8.3 `bs-park/SKILL.md`

````markdown
---
name: bs-park
description: Park a switch note (Doing, Next, Watch out) on this session's brain-swap card before switching away.
argument-hint: "[your own words, e.g. next: rerun the test, watch: timeout]"
disable-model-invocation: true
allowed-tools: Bash(brain-swap:*)
metadata:
  pack: brain-swap
  version: 0.1.0
---
Card state for this session:
!`brain-swap context --session "${CLAUDE_SESSION_ID}"`

The user's own words (may be empty): $ARGUMENTS

1. If the state has an `error:` line, reply with it and its `hint:` line, and stop.
2. The card is the `reference:` ID. If that is `none`, show the numbered list exactly as printed and ask "Park to which card?". Accept a number, an ID, or `o` (then ask which board and show `brain-swap ls --open --guess --board <name>` the same way). Ask nothing else.
3. Write the note for yourself returning cold in an hour, one short concrete line per part: Doing (in progress now), Next (the very next action), Watch out (the trap, or "nothing"). Use the user's words verbatim for the parts they gave. The labels are `doing`, `next`, `watch` and `watch out`. A label followed by a colon starts a part anywhere. A label word without a colon starts a part only at the start of the text or right after a comma. A part runs to the next part's start (drop the comma before it). Unlabelled leading words are Doing. Draft the rest from this conversation. Example: `next: rerun migration test, watch timeout` gives Next `rerun migration test` and Watch out `timeout`, and Doing is drafted from this conversation; `rerun the next test` is all Doing.
4. Save it with one command; drop `--auto` only if all three parts are the user's words:

```
brain-swap park --card <ID> --session "${CLAUDE_SESSION_ID}" --auto <<'BS_EOF'
Doing: ...
Next: ...
Watch out: ...
BS_EOF
```

5. Reply with its output line only (`parked to W-12: ...`), or on failure its first error line. Do not repeat the note or continue other work.

If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.
````

### 8.4 `bs-back/SKILL.md`

````markdown
---
name: bs-back
description: Return to this session's brain-swap card. Print its latest switch note, or save a catch-up note if it is missing or stale.
disable-model-invocation: true
allowed-tools: Bash(brain-swap:*)
metadata:
  pack: brain-swap
  version: 0.1.0
---
Card state for this session:
!`brain-swap context --session "${CLAUDE_SESSION_ID}"`

1. If the state has an `error:` line, reply with it and its `hint:` line, and stop.
2. If it says `reference: none`, show the numbered list exactly as printed and ask "Back to which card?" (for `o`, ask which board and show `brain-swap ls --open --guess --board <name>`). Run `brain-swap link <ID> --session "${CLAUDE_SESSION_ID}"`; its output is the state from now on. If `link` fails for an unknown card, reply with its first error line and stop. If it fails because no session resolves, run `brain-swap show <ID>` and use its output as the state.
3. The latest note is missing if the state says `latest note: none`, and stale if this conversation shows work on this card that the note does not reflect (for a note `by this session` whose park is visible here: exactly when a user prompt about the work came after that park). Otherwise, also in an empty or fresh conversation, it is current. When unsure, it is stale.
4. If current, run nothing and print only:

```
<ID> <title> (note <age> old)
Doing: ...
Next: ...
Watch out: ...
```

5. If missing or stale, write `no note since <age> ago on <ID>` or `no note yet on <ID>`, draft a catch-up note from this conversation (one short concrete line per part), save it, and print it as in step 4, with the first line `<ID> <title> (note just saved)`:

```
brain-swap park --card <ID> --session "${CLAUDE_SESSION_ID}" --auto <<'BS_EOF'
Doing: ...
Next: ...
Watch out: ...
BS_EOF
```

If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.
````

### 8.5 `bs-link/SKILL.md`

````markdown
---
name: bs-link
description: Link this session to an existing brain-swap card, for example W-12, and print its latest note.
argument-hint: "<card ID, e.g. W-12>"
disable-model-invocation: true
allowed-tools: Bash(brain-swap:*)
metadata:
  pack: brain-swap
  version: 0.1.0
---
Card to link: $ARGUMENTS

1. If a card ID is given, run `brain-swap link <ID> --session "${CLAUDE_SESSION_ID}"` and print its output as is, or on failure its first error line.
2. If none is given, run `brain-swap ls --open --guess`, show the numbered list exactly as printed and ask "Which card?". Accept a number, an ID, or `o` (then ask which board and add `--board <name>`). Then do step 1.

If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.
````

`link` runs through the Bash tool, not inline, so an unknown ID is reported by Claude instead of aborting the skill, and the user's text never reaches a render-time shell.

### 8.6 Catch-up rule and note passing

The CLI reads no transcripts (P3), so the prompt judges staleness from CLI facts: stamp, age, `auto`, `by this session` (T-10). Ties go to stale: an extra note costs lines, a missing one a transcript rebuild (principle 1). SP-8 scores the rule; below 9 of 10, step 3 of `/bs-back` becomes: current only when the note is `by this session` and its park is the last tool call. Notes and bodies go on stdin in a quoted heredoc (`<<'BS_EOF'`): no expansion, quoting or temp files (T-11); hence `new` takes the title from the body's `# ` line.

### 8.7 Session reference lifecycle

Set by `/bs-card`, every park, `/bs-link` and the `/bs-back` pick; read through `context`. `/clear`, a new session or an ID-changing compaction (SP-1) leaves the new ID unlinked, so the next `/bs-park` or `/bs-back` shows the picker with the pane's last card on top (FR-56, FR-57). No hook is needed (NFR-03).

### 8.8 Install

`brain-swap install claude` writes the embedded skills to `$XDG_DATA_HOME/brain-swap/pack/<version>/skills/<name>/` and symlinks `~/.claude/skills/<name>` (or `$CLAUDE_CONFIG_DIR/skills`, SP-6) to each, atomically: no checkout is needed, and the links follow the verified skills-dir install (T-12). `--link <repo>` links a checkout's `pack/claude/skills/<name>` instead. A path is ours when it is a symlink whose target is `$XDG_DATA_HOME/brain-swap/pack/<any version>/skills/<name>` or ends in `pack/claude/skills/<name>`. Anything at those paths that is not ours is refused (exit 1) unless `--force`; `--remove` deletes only our links, then `$XDG_DATA_HOME/brain-swap/pack/`. If SP-3 finds `brain-swap` off the inline shell's `PATH`, the copies get its absolute path in every command and in `allowed-tools: Bash(<absolute path>:*)`; with `--link`, install warns that the checkout's skills need `brain-swap` on the inline shell's `PATH`. It prints `run /reload-skills in sessions already open` when it created `~/.claude/skills/`, and suggests the allow rule `Bash(brain-swap:*)` so moves need no prompt; it never edits settings or hooks (FR-67).

## 9. herdr adapter

### 9.1 Detection and capture

Active when `HERDR_ENV` is `1` (FR-58); the binary is `HERDR_BIN_PATH`, else `herdr` on `PATH`. At park time `capture` builds `HerdrPlace` from `HERDR_PANE_ID`, `HERDR_TAB_ID` and `HERDR_WORKSPACE_ID` and runs no herdr command unless SP-4 enabled the moved-pane check below (FR-15). Inheritance into Claude's Bash tool is SP-4.

A process keeps the `HERDR_PANE_ID`, `HERDR_TAB_ID` and `HERDR_WORKSPACE_ID` it started with, so after its pane moves to another workspace (and gets a new ID) they name a dead pane. If SP-4 finds that `herdr pane current --current` resolves the caller's new pane, `capture` and `pane_session` use it when `herdr pane get $HERDR_PANE_ID` reports `pane_not_found` (T-15 then allows that check). Otherwise notes from a moved pane keep the old ID and each jump resolves them by session through the moved search (9.3).

### 9.2 Session of the current pane

`pane_session` runs `herdr pane get <HERDR_PANE_ID>` and returns `result.pane.agent_session.value`, or none; only 6.4 step 3 (and the 9.1 moved-pane case) uses it.

### 9.3 Locate and focus

```
locate(place, env, runner) -> PaneStatus:
  if env HERDR_ENV != "1":                                     return OutsideHerdr
  h = place.herdr, else                                        return NoPane
  r = run(herdr pane get <h.pane>)
  if r exits 0:                                                return Live(r .result.pane)
  if r timed out:                                              return NotResponding
  if stderr JSON error.code != "pane_not_found":               return HerdrError(first line)
  if place.session:                                   # pane_not_found: maybe moved
    for ws in run(herdr workspace list):
      for p in run(herdr pane list --workspace <ws.id>):
        if p.agent_session.value == place.session:             return Moved(p)
  return Closed

focus(p, runner) -> Result:                         # any call timing out: return Err(NotResponding)
  if p.agent and run(herdr agent focus <p.pane>) exits 0:      return Ok(Agent)
  if run(herdr workspace focus <p.workspace>) exits 0
     and run(herdr tab focus <p.tab>) exits 0:                 return Ok(Tab)
  return Err("focus failed")
```

Existence comes first, so a closed pane yields `pane closed` (FR-61); live IDs win over stored ones (T-16). The moved search (FR-60) stops at the first match; a timed-out list call returns `NotResponding`, and a list call that fails otherwise ends the search as `Closed`. The whole `locate` has a 3 second budget, after which it returns `NotResponding` (FR-63, AS-11); SP-4 confirms the list JSON, read with tolerant `serde_json::Value` lookups. `LivePane.agent` is true when the pane JSON has an `agent` or `agent_session` key, or `agent_status` is not `unknown`. The tab fallback is best effort. `locate` and `focus` share one 3 second budget; a focus call that times out returns `NotResponding`, shown as `herdr not responding` with the working directory (exit 6 in `jump`). Every call goes through `Runner` with a timeout of 2 seconds or the rest of the budget, whichever is less, that kills the child (T-15).

### 9.4 Key binding

`brain-swap install herdr` prints this snippet with the binary's canonical path (`current_exe`), since the herdr server may lack `~/.cargo/bin` on its `PATH`, then `herdr server reload-config` (FR-64):

```toml
# brain-swap: open the board in a popup (change the key if it collides)
[[keys.command]]
key = "prefix+alt+b"
type = "popup"
command = "/home/smilen/.cargo/bin/brain-swap"
width = "90%"
height = "90%"
```

`prefix+b` is herdr's sidebar toggle, hence `prefix+alt+b` (SP-7). A second binding with `--board home` opens another board.

### 9.5 Outside herdr and pane metadata

Outside herdr no herdr command runs and only the jump differs (FR-62, NFR-05). Later (FR-65): after a park, `herdr pane report-metadata --source brain-swap <pane> --token card=W-12`.

## 10. Concurrency and robustness

- **Board lock**: `$XDG_STATE_HOME/brain-swap/locks/<path>.lock`. A writer first runs `create_dir_all` on the board folder, then canonicalizes it, then locks; readers never create it. In the lock name `<path>`, `%` becomes `%25` and `/` becomes `%2F`. The lock is an exclusive std `File::try_lock` retried every 10 ms for 2 seconds, then `Busy` (exit 5; TUI status line: `board busy, try again`). Held for one read-modify-write, never across an editor or herdr call; readers take none (T-07).
- **Fresh reads**: a write re-reads its file inside the lock and splices the bytes on disk, never the TUI's copy, so a TUI move and a CLI park on one card both land (NFR-08).
- **Atomic writes**: temp file `.<name>.bs-tmp-<pid>` beside the target (`pid` from `Env`, so core needs no `std::process`), verify (R7), `sync_all`, `rename`, folder fsync; new cards are hard-linked (4.6).
- **External editors** are not coordinated: a racing save can drop a note; only the TUI's `e` merges (7.6).
- **Cleanup**: every write through `core::store` (CLI or TUI), after releasing the board lock, deletes `sessions/*.toml` and `panes/*` with mtime over 30 days and `.*.bs-tmp-*` files in that board folder with mtime over 1 hour; editor copies follow 7.6.
- **Bad files** (4.7) never stop a board from loading; a **missing folder** reads as empty (FR-04).
- **Config**: missing means first run (FR-66); unparseable means exit 1 with `file:line:col`, ConfigError in the TUI, an `error:` line in `context` (FR-45), never defaults that could write into wrong folders.
- **Duplicate board letters** warn; the first board in config file order wins (kept by `toml`'s `preserve_order`, 13). **Clock skew**: future stamps render `now`; order never depends on stamps alone. **Network filesystems** are unsupported (T-13).

## 11. Error handling and logging

`core::error::Error` has `Io`, `Unreadable`, `InvalidInput`, `Config { path, line, col }`, `Verify`, `Usage`, `NotFound`, `NoReference`, `Busy`; `exit_code()` and `code()` implement 6.2, and `Display` is one lower-case line with the path (T-14). Jump outcomes are `PaneStatus` values. The TUI shows action errors and the first load warning in the status line, all warnings in Help. Logging is off by default; `BRAIN_SWAP_LOG=<file>` appends herdr calls (argv, exit, ms), lock waits over 100 ms, reloads, merges and verify failures. A `BRAIN_SWAP_LOG` path inside a configured board folder is ignored with a warning (FR-05).

## 12. Testing strategy and definition of done

### 12.1 Test layers

Tests are hermetic: the author runs them inside a herdr pane and a Claude session, whose variables a child would inherit. Every process a test spawns goes through one helper. It calls `env_clear()`, then sets only `PATH`, `HOME`, `XDG_*`, `TZ=UTC`, `BRAIN_SWAP_NOW` and the scenario's `HERDR_*`, with `HERDR_BIN_PATH` always the fake script. Core tests never read the process environment.

- **Core unit tests** per module, with `Env` injected (fixed `now`).
- **Round trip**: fixtures in `tests/fixtures/cards/` (hand-edited, CRLF, no frontmatter, unknown keys, fenced `##`, bad stamps, quoted places, non-UTF-8); per fixture and splice an insta snapshot plus R2, R3 and R7. Parsing never panics.
- **CLI**: `assert_cmd` with `HOME` and `XDG_*` in `tempfile` folders, a fixed `BRAIN_SWAP_NOW` and per-test `HERDR_*`; text and JSON snapshots per subcommand and exit code; `context` pinned in every state (no or broken config, missing folder, no reference, deleted card, empty or placeholder session, with and without notes), always exit 0. `new --session ""` inside a fake herdr pane sets the reference under the herdr session.
- **Crash**: with feature `failpoints`, `BRAIN_SWAP_FAILPOINT` aborts at `create:after_next`, `create:after_tmp`, `create:after_link`, `park:after_tmp` or `edit:after_tmp`; a reload then asserts I1 to I4, that every file parses, and that no number is reused.
- **Concurrency**: 20 parallel `park` and 5 `move` on two cards, then 10 parallel `new`: all files parse, 20 notes, 10 distinct IDs.
- **TUI**: `update` tests per transition; `TestBackend` snapshots at 80x24 and 120x40; runtime tests for polling and for the editor merge, with a scripted `editor` that parks on the card before exiting. The scripted editor starts with `env -i` and the scenario's variables and passes `--session test`; the runtime takes its editor launcher and terminal guard as injectable parts so the test needs no tty.
- **herdr**: `FakeRunner` tests for every `PaneStatus` and focus path, a failing focus, a hang during the moved search (`NotResponding`) and the 3 second budget; CLI tests with `HERDR_BIN_PATH` pointing at a fake script that answers canned JSON per `FAKE_HERDR_SCENARIO` and logs argv.
- **Pack** (`tests/pack.rs`): frontmatter checks; each inline command run with `sh -c` in every `context` state (exit 0, header line); every `brain-swap ...` command of each SKILL.md, substituted, parses with `Cli::try_parse_from`; each heredoc command runs with sample text.
- **Docs**: no U+2014 or U+2013 in any repository markdown file. **Performance**: an `#[ignore]` release test on 300 cards with 20 notes, plus `hyperfine` (NFR-01, NFR-02). **Manual**: AS-1 to AS-11 in real herdr and Claude Code before release.

### 12.2 Definition of done

Inherited by every task except E0 spikes, whose DoD is their own; for manual checks, items 1 and 3 are replaced by recording the run (date, steps passed) in the commit message.

1. Starts from a failing test named after its requirement (`fr16_park_appends_only_to_timeline`) that passes at the end.
2. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --all-features` pass on Linux.
3. Snapshot changes reviewed (`cargo insta review`) and committed.
4. No crate beyond section 13 without editing this document first.
5. Layering, docs and pack tests pass; `--help` and README reflect user-visible changes.
6. One commit per task saying what and why, with no AI attribution.

## 13. Build, packaging, installation

```toml
[package]
name = "brain-swap"      # bin brain-swap, lib brain_swap
version = "0.1.0"
edition = "2024"
rust-version = "1.89"    # std File::lock and try_lock

[features]
failpoints = []          # crash tests only

[dependencies]
ratatui = "0.30"                                    # the TUI; re-exports crossterm
clap = { version = "4.6", features = ["derive"] }   # subcommands, help, usage errors
serde = { version = "1", features = ["derive"] }    # config, references, JSON
toml = { version = "1.1", features = ["preserve_order"] }   # config and session files; keeps [boards] order
serde_json = "1"                                    # --json output, herdr JSON
jiff = "0.2"                                        # stamps with offsets, time zones

[dev-dependencies]
insta = "1.49"
assert_cmd = "2.2"
tempfile = "3.27"
```

Six runtime crates (NFR-11); toolchain 1.98.1, no `rust-toolchain.toml`. Rejected: pulldown-cmark, notify, directories, anyhow, thiserror, chrono, ulid, fs2, YAML crates. Install: `cargo install --path .` (or `--git <url>`) into `~/.cargo/bin` (FR-68), optionally `alias bs=brain-swap`; then `brain-swap install claude` and `brain-swap install herdr` (paste, `herdr server reload-config`). Upgrades repeat all three.

## 14. Risks and spikes

Each spike: two hours at most, no product code, the answer recorded in section 16.

- **SP-1 Session ID.** Does `${CLAUDE_SESSION_ID}` expand in a user skill's inline commands and body, equal herdr's `agent_session.value`, and survive auto-compaction? Fallback: the 6.4 chain.
- **SP-2 Jump from a popup.** Does a `popup` command see `HERDR_*` and the user's `PATH`, does a `popup` close when its command exits, and does `herdr agent focus` from it hold after the popup exits? Fallback: exit first, then a detached `brain-swap jump <ID>`; if `HERDR_*` is missing, `install herdr` prints `command = "env HERDR_ENV=1 HERDR_BIN_PATH=<herdr path> <brain-swap path>"`.
- **SP-3 Permissions and PATH in Claude.** Does `Bash(brain-swap:*)` pre-approve inline commands and heredoc calls, and is `~/.cargo/bin` on the inline shell's `PATH`? Fallbacks: a documented allow rule; absolute paths in the copies.
- **SP-4 herdr from Claude.** Does Claude's Bash tool inherit `HERDR_*`, and what do `herdr workspace list` and `pane list --workspace` print? After a pane moves, does `herdr pane current --current` resolve the caller's new pane, or only echo `HERDR_PANE_ID` (9.1)? Fallback: notes from a moved pane keep the old ID; the jump's moved search (9.3) resolves them by session.
- **SP-5 Focus.** What does `agent focus` do without an agent; do `workspace focus` and `tab focus` cross workspaces?
- **SP-6 Skill install.** Does a symlinked `~/.claude/skills/bs-park/` give a bare `/bs-park` without restart that Claude cannot invoke? Does Claude Code load skills from `$CLAUDE_CONFIG_DIR/skills` when it is set? Fallback: copies.
- **SP-7 Key.** Is `prefix+alt+b` free in herdr 0.8.2 and the author's config?
- **SP-8 Staleness accuracy.** In 10 scripted sessions (5 stale, 5 fresh), does `/bs-back` judge 9 or more right? Fallback: the stricter rule of 8.6.

Unspiked risk: chatter despite "ask nothing" (manual AS-1, AS-3).

## 15. Work breakdown

Milestones follow principle 1: the cue (files, CLI, pack) ships before the board; order sets priority, not a gate. **M0** E0-F1, E0-F2; **M1** E1, E2, E5-F1; **M2** E3, E0-F3; **M3** E4; **M4** E5-F2 to E5-F4; **M5** E6.

Each feature: scope; Covers; Dep; DoD beyond 12.2.

### E0 Spikes (M0; E0-F3 in M2)

- **E0-F1 Claude spike.** SP-1, SP-3, SP-4, SP-6 in a herdr pane. Covers FR-41, FR-46, FR-57, FR-67. Dep: none. DoD: answers in 16; 6.4, 8, 9.1 updated if a fallback is taken.
- **E0-F2 herdr spike.** SP-2, SP-5, SP-7. Covers FR-59, FR-60, FR-64. Dep: none. DoD: answers in 16; 7.7, 9.3, 9.4 updated.
- **E0-F3 Staleness spike.** SP-8 on the shipped `/bs-back`. Covers FR-52, FR-53. Dep: E3-F1. DoD: 9 of 10 or more, with the 8.6 fallback if needed.

### E1 Core (M1)

- **E1-F1 Foundations.** Layout, `Env`, `Error`, exit codes, XDG paths, ages, stamps (section 3), failpoints, log sink (`BRAIN_SWAP_LOG`, section 11), the hermetic spawn helper (12.1), layering and docs tests. Covers FR-36, NFR-03, NFR-04, NFR-09, NFR-11. Dep: none. DoD: a planted `std::process` fails the layering test; every age boundary tested; stamps round-trip byte for byte; the log sink appends one line per event written through its API to the file named by `BRAIN_SWAP_LOG` and writes nothing when it is unset.
- **E1-F2 Card grammar.** Frontmatter, card file, place line, R1 to R5. Covers FR-01, FR-07, FR-14, FR-16 to FR-18, NFR-06, NFR-07. Dep: E1-F1. DoD: every fixture round-trips byte for byte.
- **E1-F3 Config and keys.** Defaults, first run, registry, key syntax, conflicts. Covers FR-02, FR-42 to FR-45, FR-66. Dep: E1-F1. DoD: a bad binding warns and keeps its default; a swap of two keys is accepted; a user binding colliding with another action's default reverts only the user-set action; a broken file reports line and column; each 5.2 validation case (unknown `default_board`, empty `[boards]`, bad board name, wrong type, relative path) is a config error; boards keep file order.
- **E1-F4 Templates.** Built-ins, user folder, hotkeys. Covers FR-11 to FR-13. Dep: E1-F1. DoD: user files override and add; a file without frontmatter, one without `key` and an unreadable one load as 5.4 says.
- **E1-F5 Board loading.** `board.md`, discovery, unknown columns, order, lookup. Covers FR-01, FR-03, FR-04, FR-08 to FR-10. Dep: E1-F2, E1-F3. DoD: 4.9 and AS-7 load as described.
- **E1-F6 Store.** Lock, verify, atomic write, create, move, park, merge, cleanup. Covers FR-04 to FR-06, FR-13, FR-16, NFR-06, NFR-08. Dep: E1-F4, E1-F5. DoD: 12.1 crash and concurrency tests pass; a lock wait over 100 ms is appended to the log file.
- **E1-F7 Session store.** References, pane hints, pruning, 6.4 steps 1, 2, 4. Covers FR-05, FR-41, FR-57. Dep: E1-F1. DoD: empty, placeholder and invalid IDs warn and fall through.
- **E1-F8 Best guess.** The tiers of 6.6. Covers FR-38. Dep: E1-F5, E1-F7. DoD: a test per tier and tie, including recovery by pane hint.

### E2 CLI (M1)

- **E2-F1 Skeleton.** clap, flags, JSON envelope, errors, `templates`. Covers FR-11, FR-33, FR-35, FR-36. Dep: E1-F3, E1-F4. DoD: exit codes 0, 1 and 2 and the JSON error envelope on stdout produced by tests; every other code is tested by the feature that produces it (3 in E2-F2, 4 and 5 in E2-F3, 6 in E5-F2).
- **E2-F2 new, move, ls, show.** Creation, moves, listings. Covers FR-13, FR-38, FR-39. Dep: E2-F1, E1-F6, E1-F8. DoD: text and JSON snapshots; `new --body -` snapshots with and without a positional title; unknown names exit 3.
- **E2-F3 park, link, context.** Stdin and prompts, capture, references, the context format. Covers FR-15, FR-16, FR-18, FR-34, FR-37, FR-41, FR-56, FR-57. Dep: E2-F2, E1-F7, E5-F1. DoD: `context` pinned in every 12.1 state; the CLI steps of AS-6 (`park --card` from a terminal and from stdin, no `auto`) and the CLI half of AS-8 (two parallel parks on different cards and on one card) pass.

### E3 Claude pack (M2)

- **E3-F1 Skills.** Section 8 files, embedding, pack tests. Covers FR-46 to FR-57. Dep: E2-F3, E0-F1. DoD: manual AS-1 (steps 1, 3, 4, 8), AS-2, AS-3 recorded; for the manual run, `pack/claude/skills/*` is symlinked into `~/.claude/skills/` by hand.
- **E3-F2 install claude.** Versioned copy, symlinks, `--link`, `--force`, `--remove`. Covers FR-67. Dep: E3-F1. DoD: temp `HOME` tests: install, upgrade, foreign folder refused without `--force` and replaced with it, removal.

### E4 TUI (M3)

- **E4-F1 Shell.** `update`, runtime, terminal, poll, status line, Help, ConfigError. Covers FR-19, FR-23, FR-32, FR-44, FR-45, NFR-02, NFR-10. Dep: E1-F3, E1-F5. DoD: a file change shows after one `Tick`; a snapshot test of the initial selection (newest open card, and the first column with none).
- **E4-F2 Board view.** Columns, cards, preview panel, ages, moves. Covers FR-20 to FR-22, FR-29. Dep: E4-F1, E1-F6. DoD: 4.9 snapshots at both sizes; no panel at 80x19.
- **E4-F3 Detail view.** Notes, place, body scroll. Covers FR-26, FR-27. Dep: E4-F1. DoD: first, middle, last note snapshots.
- **E4-F4 Pickers.** Template and board pickers, title input (5.2). Covers FR-24, FR-30. Dep: E4-F1, E1-F6. DoD: AS-6 creation emits one `Create`; `j` and `k` type into a title.
- **E4-F5 Editor.** The 7.6 merge. Covers FR-31. Dep: E4-F1, E1-F6, E2-F3. DoD: a note parked mid-edit survives; AS-8 with the TUI editor passes.

### E5 herdr adapter (M1, M4)

- **E5-F1 Runner, capture, pane session.** `Runner` with timeout, `FakeRunner`, fake herdr script, 9.1, 9.2. Covers FR-15, FR-41, FR-58. Dep: E1-F1, E0-F1. DoD: a hung child is killed at 2 s; no capture outside herdr; every Runner call is appended to the log file (argv, exit, ms).
- **E5-F2 Locate and jump.** 9.3, `locate`, `jump`. Covers FR-40, FR-59 to FR-63, NFR-05. Dep: E5-F1, E2-F1, E1-F5, E0-F1, E0-F2. DoD: every `PaneStatus` and focus path, a failing focus (exit 6), a timed-out focus (`not_responding`, exit 6), a card with no note and an out-of-range `--note`, a hang in the moved search and the 3 second budget, argv sequences asserted.
- **E5-F3 TUI jump.** Enter in Board and Detail, exit, messages (7.7). Covers FR-25, FR-28, FR-59 to FR-63. Dep: E5-F2, E4-F2, E4-F3, E4-F4, E2-F3. DoD: a state test per outcome, including a failed focus; AS-6 end to end; manual AS-1 step 7, AS-4, AS-5, AS-10, AS-11.
- **E5-F4 install herdr.** The 9.4 snippet and a README section. Covers FR-64. Dep: E2-F1, E0-F2. DoD: snippet snapshot.

### E6 Release (M5)

- **E6-F1 Release check.** README, AS-1 to AS-11, `hyperfine`, clean `cargo install`. Covers FR-68, NFR-01, NFR-02, NFR-05. Dep: all Core features. DoD: PRD section 9 measurable.

### E7 Later (not scheduled)

E7-F1 pane metadata (FR-65), E7-F2 WIP limit (FR-69), E7-F3 other packs (FR-70), E7-F4 archive (FR-71), E7-F5 manual order (FR-72), E7-F6 TUI notes (FR-73), E7-F7 clipboard (FR-74).

### 15.1 Parallel lanes

Dep lines are authoritative; this list only groups them. Start: E0-F1, E0-F2, E1-F1. After E1-F1: E1-F2, E1-F3, E1-F4, E1-F7, and E5-F1 once E0-F1 is done. After E1-F2 and E1-F3: E1-F5. After E1-F3 and E1-F4: E2-F1. After E1-F5: E1-F6 (with E1-F4), E1-F8 (with E1-F7), E4-F1; after E4-F1: E4-F3. After E1-F6 and E4-F1: E4-F2, E4-F4. After E2-F1, E1-F6 and E1-F8: E2-F2. After E2-F2, E1-F7 and E5-F1: E2-F3. After E2-F3 and E4-F1: E4-F5. After E2-F3 and E0-F1: E3-F1; after E3-F1: E3-F2 and E0-F3. After E5-F1, E2-F1, E1-F5, E0-F1 and E0-F2: E5-F2. After E5-F2, E4-F2, E4-F3, E4-F4 and E2-F3: E5-F3. After E2-F1 and E0-F2: E5-F4. E0-F1 gates E3-F1, E5-F1 and E5-F2; E0-F2 gates E5-F2 and E5-F4.

### 15.2 Traceability matrix

- FR-01 E1-F2 E1-F5; FR-02 E1-F3; FR-03 E1-F5; FR-04 E1-F5 E1-F6; FR-05 E1-F6 E1-F7
- FR-06 E1-F6; FR-07 E1-F2; FR-08 E1-F5; FR-09 E1-F5; FR-10 E1-F5
- FR-11 E1-F4 E2-F1; FR-12 E1-F4; FR-13 E1-F4 E1-F6 E2-F2
- FR-14 E1-F2; FR-15 E5-F1 E2-F3; FR-16 E1-F2 E1-F6 E2-F3; FR-17 E1-F2; FR-18 E1-F2 E2-F3
- FR-19 E4-F1; FR-20 E4-F2; FR-21 E4-F2; FR-22 E4-F2; FR-23 E4-F1; FR-24 E4-F4; FR-25 E5-F3
- FR-26 E4-F3; FR-27 E4-F3; FR-28 E5-F3; FR-29 E4-F2; FR-30 E4-F4; FR-31 E4-F5; FR-32 E4-F1
- FR-33 E2-F1; FR-34 E2-F3; FR-35 E2-F1; FR-36 E1-F1 E2-F1; FR-37 E2-F3; FR-38 E1-F8 E2-F2; FR-39 E2-F2; FR-40 E5-F2; FR-41 E1-F7 E5-F1 E2-F3 E0-F1
- FR-42 E1-F3; FR-43 E1-F3; FR-44 E1-F3 E4-F1; FR-45 E1-F3 E4-F1
- FR-46 E3-F1 E0-F1; FR-47 to FR-51 E3-F1; FR-52 E3-F1 E0-F3; FR-53 E3-F1 E0-F3; FR-54 E3-F1; FR-55 E3-F1; FR-56 E3-F1 E2-F3; FR-57 E1-F7 E2-F3 E3-F1 E0-F1
- FR-58 E5-F1; FR-59 E5-F2 E5-F3 E0-F2; FR-60 E5-F2 E5-F3 E0-F2; FR-61 to FR-63 E5-F2 E5-F3; FR-64 E5-F4 E0-F2; FR-65 E7-F1
- FR-66 E1-F3; FR-67 E3-F2 E0-F1; FR-68 E6-F1
- FR-69 E7-F2; FR-70 E7-F3; FR-71 E7-F4; FR-72 E7-F5; FR-73 E7-F6; FR-74 E7-F7
- NFR-01 E6-F1; NFR-02 E4-F1 E6-F1; NFR-03 E1-F1; NFR-04 E1-F1; NFR-05 E5-F2 E6-F1; NFR-06 E1-F2 E1-F6
- NFR-07 E1-F2; NFR-08 E1-F6; NFR-09 E1-F1; NFR-10 E4-F1; NFR-11 E1-F1

## 16. Assumptions and open questions (technical)

Choices made where the PRD and the environment leave room, as yes/no questions for the owner.

- **T-01** One package, no workspace?
- **T-02** A named herdr field in `Place`?
- **T-03** Hand-parsed flat frontmatter, no YAML or markdown crate?
- **T-04** No stored card order?
- **T-05** Accept ID collisions from offline creation on two machines?
- **T-06** One-second polling, no file watcher?
- **T-07** One std file lock per board per transaction, in the state folder?
- **T-08** Hand-computed XDG paths, also on macOS?
- **T-09** Session from `--session`, `BRAIN_SWAP_SESSION` or herdr; never `CLAUDE_*`, except `CLAUDE_CONFIG_DIR` in `install claude` (SP-6)?
- **T-10** Staleness judged by the model, scored by SP-8, no transcripts?
- **T-11** Notes and bodies via a quoted heredoc?
- **T-12** Skills as symlinks to a versioned copy, or a checkout with `--link`?
- **T-13** No network filesystems?
- **T-14** One hand-written error enum, no anyhow or thiserror?
- **T-15** 2 second herdr timeout, and no herdr call when parking with a valid session?
- **T-16** Live tab and workspace IDs over stored ones?
- **T-17** Constants, not settings: lock 2 s, poll 1 s, age refresh 30 s, pruning 30 days, temp cleanup 1 h, panel from 20 rows?
- **T-18** The place as one `<!-- where: ... -->` line?
- **T-19** Pane hint files instead of scanning session files?
- **T-20** Verify before rename on every write?
- **T-21** `BRAIN_SWAP_NOW` in every build, failpoints only with the feature?

Open until the spikes answer: SP-1 whether the 6.4 fallback is needed; SP-2 focus before or after exit; SP-3 absolute paths in copies; SP-4 the moved-pane check in `capture`; SP-7 the key; SP-8 the staleness rule.
