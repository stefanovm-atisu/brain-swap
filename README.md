# brain-swap

A keyboard-first kanban board for the terminal, built to make switching between parallel AI coding sessions cheap.

Each card is a task. When you leave a session, you park a short note on the card (Doing / Next / Watch out). When you come back, you read the note first and pick up where you left off in seconds, instead of rereading the chat transcript.

- Boards are plain markdown folders you own (e.g. work, home, personal).
- Vim-like keys, every binding configurable.
- Optional Claude Code commands: `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`.
- Optional herdr integration: open the board with a key, jump from a card to its pane.

Status: idea stage, no code yet. See [idea.md](idea.md) for the product idea. Written in Rust.

## Claude Code pack

Install the `/bs-card`, `/bs-park`, `/bs-back` and `/bs-link` skills with:

```sh
brain-swap install claude
```

It copies each skill as a plain folder `~/.claude/skills/<name>/SKILL.md` and prints one `installed <folder>` line per skill. When it created the skills folder, run `/reload-skills` in Claude Code sessions that are already open. It never edits Claude Code settings or hooks; add the allow rule `Bash(brain-swap:*)` to your settings yourself so moves need no prompt. `--json` returns the written folders as `linked`. If a skill path already holds something brain-swap did not create (another folder, a file or a symlink), install refuses and writes nothing; `--force` replaces those entries and lists them as `removed`.

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
