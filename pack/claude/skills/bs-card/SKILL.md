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
