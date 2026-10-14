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
2. The card is the `reference:` ID. If that is `none`, show the list exactly as printed, from its `open cards on <board>, best guess first:` heading line to its last line, as plain text and without this skill's own labels, and ask "Park to which card?". Accept a number, an ID, or `o` (then ask which board and show `brain-swap ls --open --guess --board <name>` the same way). Ask nothing else.
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
