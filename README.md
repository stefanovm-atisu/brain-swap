# brain-swap

A keyboard-first kanban board for the terminal, built to make switching between parallel AI coding sessions cheap.

Each card is a task. When you leave a session, you park a short note on the card (Doing / Next / Watch out). When you come back, you read the note first and pick up where you left off in seconds, instead of rereading the chat transcript.

- Boards are plain markdown folders you own (e.g. work, home, personal).
- Vim-like keys, every binding configurable.
- Optional Claude Code commands: `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`.
- Optional herdr integration: open the board with a key, jump from a card to its pane.

Status: v0.1. Written in Rust, one binary that is both the board (run `brain-swap` with no arguments) and the CLI. The product idea is in [idea.md](idea.md), the requirements in [PRD.md](PRD.md) and the design in [TECHSPEC.md](TECHSPEC.md).

## Cheat sheet

In Claude Code:

- `/bs-card fix the login redirect` creates a card and links this session (`feature: ...` picks the template, `on home` the board).
- `/bs-park` saves a switch note before you leave (`/bs-park next: rerun the test` keeps your words).
- `/bs-back` prints the note when you return, or saves a catch-up note if it is missing or stale.
- `/bs-link W-12` links this session to an existing card.

In a shell:

- `brain-swap` opens the board.
- `brain-swap new Fix login redirect` creates a card.
- `brain-swap park --card W-1` saves a note read from standard input.
- `brain-swap show W-1` prints the card and its latest note.
- `brain-swap ls` lists the cards.
- `brain-swap move W-1 Done` moves a card.
- `brain-swap link W-1 --session <id>` links a session to a card.

Board keys: `hjkl` move, `H` `L` move card, `o` open, `enter` jump, `n` new, `e` edit, `b` board, `?` help, `q` quit.

The full guide with examples is [docs/usage.md](docs/usage.md).

## Install

You need Rust 1.89 or later and Linux. macOS should build as well.

1. Install the binary from a checkout of this repository. It goes to `~/.cargo/bin`, which must be on your `PATH`:

   ```sh
   cargo install --path .
   ```

   A short alias is optional:

   ```sh
   alias bs=brain-swap
   ```

2. Install the Claude Code pack (see [Claude Code pack](#claude-code-pack)):

   ```sh
   brain-swap install claude
   ```

3. Print the herdr key binding (see [herdr key binding](#herdr-key-binding)), paste it into herdr's config, then reload herdr:

   ```sh
   brain-swap install herdr
   herdr server reload-config
   ```

Steps 2 and 3 are optional. Without them the board and the CLI work on their own.

## Claude Code pack

Four Claude Code commands. Each one runs `brain-swap` for you:

- `/bs-card <words>` creates a card for the task in this session and links the session to it. Start with a template label to pick the template yourself, for example `/bs-card feature: export the board as CSV`, or end with it, for example `/bs-card migrate invoices to v13 as a feature`.
- `/bs-park` saves a switch note (Doing, Next, Watch out) before you leave the session.
- `/bs-back` prints the latest note when you come back, or writes a catch-up note if it is missing or stale.
- `/bs-link <ID>` links this session to an existing card, for example `/bs-link W-12`.

Install them with:

```sh
brain-swap install claude
```

It copies each skill as a plain folder `~/.claude/skills/<name>/SKILL.md` and prints one `installed <folder>` line per skill. When `$CLAUDE_CONFIG_DIR` is set and non-empty, the skills go to `$CLAUDE_CONFIG_DIR/skills` instead. When it created the skills folder, run `/reload-skills` in Claude Code sessions that are already open. It never edits Claude Code settings or hooks; it prints the allow rule with the absolute path of your binary, for example `Bash(/home/you/.cargo/bin/brain-swap:*)`. Add it to your settings yourself so moves need no prompt. `--json` returns the written folders as `linked`. If a skill path already holds something brain-swap did not create (another folder, a file or a symlink), install refuses and writes nothing; `--force` replaces those entries and lists them as `removed`.

To work on the skills from a checkout of this repository, run `brain-swap install claude --link <repo>`: it copies `<repo>/pack/claude/skills/<name>/SKILL.md` instead of the embedded skills, so run it again after each edit. A plain `brain-swap install claude` switches back to the embedded skills.

`brain-swap install claude --remove` deletes the skill folders brain-swap installed (embedded or from a checkout) and prints one `removed <folder>` line each; anything else in the skills folder, including a skill folder whose `SKILL.md` you replaced, stays. `--json` returns them as `removed`; `--force` and `--link` change nothing here.

## herdr key binding

Open the board in a herdr popup with one key. Run:

```sh
brain-swap install herdr
```

It prints a snippet with the absolute path of your `brain-swap` binary (the herdr server may not have `~/.cargo/bin` on its `PATH`), prefixed with `env` so the popup sees herdr's variables. For a binary at `/home/you/.cargo/bin/brain-swap` it looks like this:

```toml
# brain-swap: open the board in a popup (change the key if it collides)
[[keys.command]]
key = "prefix+alt+b"
type = "popup"
command = "env HERDR_ENV=1 HERDR_BIN_PATH=herdr /home/you/.cargo/bin/brain-swap"
width = "90%"
height = "90%"
```

Paste it into herdr's config, then run `herdr server reload-config`. `install herdr` never edits herdr's config itself; `--json` returns the snippet as JSON.

The default key is `prefix+alt+b` because `prefix+b` is herdr's sidebar toggle. If it collides with another binding, change `key` to any free one.

A second binding on another key opens a specific board, here `home`:

```toml
[[keys.command]]
key = "prefix+alt+h"
type = "popup"
command = "env HERDR_ENV=1 HERDR_BIN_PATH=herdr /home/you/.cargo/bin/brain-swap --board home"
width = "90%"
height = "90%"
```

Outside herdr everything works except the jump from a card to its pane.

## Upgrade

Repeat the three install steps from your updated checkout. Step 2 installs the built-in skills, so if you work on the skills from a checkout, run `brain-swap install claude --link <repo>` instead. Step 3 prints the binding again in case it changed:

```sh
cargo install --path .
brain-swap install claude
brain-swap install herdr
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | runtime error |
| 2 | usage |
| 3 | not found |
| 4 | no reference |
| 5 | busy |
| 6 | jump not performed |
