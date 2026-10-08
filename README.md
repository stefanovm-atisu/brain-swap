# brain-swap

A keyboard-first kanban board for the terminal, built to make switching between parallel AI coding sessions cheap.

Each card is a task. When you leave a session, you park a short note on the card (Doing / Next / Watch out). When you come back, you read the note first and pick up where you left off in seconds, instead of rereading the chat transcript.

- Boards are plain markdown folders you own (e.g. work, home, personal).
- Vim-like keys, every binding configurable.
- Optional Claude Code commands: `/bs-card`, `/bs-park`, `/bs-back`, `/bs-link`.
- Optional herdr integration: open the board with a key, jump from a card to its pane.

Status: idea stage, no code yet. See [idea.md](idea.md) for the product idea. Written in Rust.
