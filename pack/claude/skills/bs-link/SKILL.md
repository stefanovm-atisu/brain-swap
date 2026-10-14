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

1. If a card ID is given, run `brain-swap link <ID> --session "${CLAUDE_SESSION_ID}"` and print its output exactly as printed, as plain text and not in a code block, or on failure its first error line.
2. If none is given, run `brain-swap ls --open --guess`, show the list exactly as printed, from its `open cards on <board>, best guess first:` heading line to its last line, as plain text and without this skill's own labels, and ask "Which card?". Accept a number, an ID, or `o` (then ask which board and add `--board <name>`). Then do step 1.

If the user later asks to move this card, run `brain-swap move <ID> <column>` and reply with its output line.
