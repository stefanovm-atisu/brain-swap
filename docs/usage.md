# Using brain-swap

This guide assumes you installed the binary, and optionally the Claude Code pack and the herdr key (see the [README](../README.md#install)).

## The daily loop

1. Start a task in a Claude Code session and create a card for it with `/bs-card`. The session is now linked to the card.
2. Before you leave the session, run `/bs-park`. It saves a switch note on the card: Doing, Next, Watch out.
3. When you come back, run `/bs-back`. It prints the note, so you know where you were before you read anything else.
4. Open the board (`brain-swap` with no arguments, or the herdr key) to see all cards and their latest notes.
5. On the board, press `enter` on a card to jump to the herdr pane where it was parked.

Claude never runs these commands on its own. You type them.

## Claude Code commands

### /bs-card

Creates a card for the task and links this session to it.

```
/bs-card fix the login redirect
```

Claude picks the template that fits (Feature, Bug, Research or Chore) and fills its fields from the conversation. To pick the template yourself, start with its name and a colon, or end with `as a <template>`:

```
/bs-card feature: export the board as CSV
/bs-card timeout on export as a bug
```

The card goes to your default board. Add `on home` or `on personal` to use another board:

```
/bs-card buy paint for the fence on home
```

With no words, Claude takes the task from the conversation.

### /bs-park

Saves a switch note on this session's card before you leave.

```
/bs-park
```

With no words, Claude drafts all three parts from the conversation and the note is marked `auto`. A note stays marked `auto` unless you wrote all three parts yourself.

Give your own words with the labels `doing:`, `next:` and `watch out:` (or `watch:`). Claude keeps your words as typed and drafts only the parts you left out:

```
/bs-park next: rerun the migration test, watch out: staging uses another host
```

Words before the first label are Doing. Labels are matched literally, so a misspelled label is not recognised. A label without a colon counts only at the start or right after a comma.

The reply is one line, for example `parked to W-1: Fix login redirect`.

### /bs-back

Run it when you return to a session.

```
/bs-back
```

If the latest note is current, it just prints it:

```
W-1 Fix login redirect (note 2 h old)
Doing: tracing the redirect
Next: add a test for the callback
Watch out: staging uses a different host
```

If there is no note yet, or the conversation went on after the last park, Claude writes a catch-up note from the conversation, saves it (marked `auto`) and prints it. A fresh conversation (a new session or after `/clear`) counts as current, so `/bs-back` just prints the note. When Claude is unsure, it treats the note as stale.

### /bs-link

Links this session to an existing card and prints its latest note. Use it when you continue a card in a new session.

```
/bs-link W-12
```

With no ID it shows the picker below.

### The picker

After `/clear`, or in a new session, the session is not linked to a card yet. `/bs-park`, `/bs-back` and `/bs-link` then show a numbered list of the open cards, best guess first. Inside herdr the card of this pane comes first, marked `linked here`:

```
open cards on work, best guess first:
1. W-1 Fix login redirect [Doing] 2 h, linked here: add a test for the callback
2. W-2 Timeout on export [Todo] no note
o. other board: home, personal
```

Answer with a number, a card ID such as `W-2`, or `o` to pick a card on another board.

### Moving a card

After any of these commands you can ask Claude in plain words to move the card, for example "move it to Done". It runs `brain-swap move <ID> <column>`.

## The same loop with the CLI alone

Everything works without Claude Code and without herdr. Cards are on the default board unless you add `--board <name>`.

Create a card, optionally from a template:

```sh
brain-swap new Fix login redirect
brain-swap new --template Bug Timeout on export
brain-swap --board home new Buy paint
```

Each prints a line such as `created W-1: Fix login redirect (Todo, work)`. `brain-swap templates` lists the templates.

Park a note. It is read from standard input, one line per part:

```sh
brain-swap park --card W-1 <<'EOF'
Doing: tracing the redirect
Next: add a test for the callback
Watch out: staging uses a different host
EOF
```

Read it back, list cards, move a card:

```sh
brain-swap show W-1
brain-swap show W-1 --all
brain-swap ls
brain-swap ls --open
brain-swap move W-1 Doing
```

`show` prints the latest note, `--all` adds the older ones. The default columns are Todo, Doing and Done.

Link a session to a card, then see what a session is linked to:

```sh
brain-swap link W-1 --session <session-id>
brain-swap context --session <session-id>
```

`brain-swap jump W-1` focuses the card's pane; outside herdr it prints `not in herdr` and the folder where the card was parked.

## The board

Run `brain-swap` with no arguments. Default keys:

| Key | Board | Card detail |
|-----|-------|-------------|
| `h` `j` `k` `l` or arrows | move the selection | `k` newer note, `j` older note |
| `H` `L` | move the card one column left or right | |
| `o` | open the card | |
| `enter` | jump to the card's pane | jump to the pane of the note shown |
| `n` | new card (pick a template, type a title) | |
| `e` | edit the card in your editor | edit the card |
| `b` | switch board | |
| `ctrl+d` `ctrl+u` | | scroll |
| `?` | help: every key | help |
| `q` `esc` | quit | back to the board |

## Configuration

The config is `~/.config/brain-swap/config.toml` (or under `$XDG_CONFIG_HOME`). It is written with all defaults on first run. There you change the boards, the default board, the editor and every key. Custom templates go in `~/.config/brain-swap/templates/`. Boards are plain markdown folders, by default `~/.local/share/brain-swap/<board>`.
