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
