#!/usr/bin/env python3
"""PreToolUse hook: deny git commit commands that contain AI attribution.

Blocks Co-Authored-By: Claude trailers, "Generated with Claude Code" footers,
session-id trailers and the robot marker in commit messages.
"""
import json
import re
import sys

try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

cmd = data.get("tool_input", {}).get("command", "")

# Only inspect commands where "commit" is a git subcommand
# (git commit, git -C path commit, git -c k=v commit).
if not re.search(r"\bgit\b(?:\s+(?:-\S+|-C\s+\S+|-c\s+\S+))*\s+commit\b", cmd):
    sys.exit(0)

BANNED = [
    r"co-authored-by:\s*claude",
    r"generated with[^\n]*claude",
    r"claude-session",
    r"noreply@anthropic\.com",
    r"\U0001F916",  # robot emoji
]

for pattern in BANNED:
    if re.search(pattern, cmd, re.IGNORECASE):
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": (
                    "Commit message contains AI attribution "
                    "(Co-Authored-By: Claude / Generated with Claude Code / "
                    "session id / robot emoji). Repository rule: commits never "
                    "mention the model, tool or session. Remove it and retry."
                ),
            }
        }))
        sys.exit(0)

sys.exit(0)
