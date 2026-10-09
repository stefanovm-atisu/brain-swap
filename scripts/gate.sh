#!/usr/bin/env bash
# Quality gate. Nothing merges to main without it. Usage: scripts/gate.sh [TASK-ID]
set -euo pipefail
cd "$(dirname "$0")/.."

if grep -rnP '[\x{2014}\x{2013}]' --include='*.md' --include='*.rs' --include='*.toml' . --exclude-dir=target --exclude-dir=.git --exclude-dir=.claude; then
  echo "gate: em or en dash found" >&2; exit 1
fi
python3 scripts/backlog.py check
[ -f Cargo.toml ] || { echo "gate: no Cargo.toml yet" >&2; exit 1; }
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
if [ $# -gt 0 ]; then python3 scripts/backlog.py verify "$1"; fi
echo "gate: ok"
