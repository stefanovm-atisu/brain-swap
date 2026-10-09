#!/usr/bin/env bash
# Quality gate. Nothing merges to main without it. Usage: scripts/gate.sh [TASK-ID]
set -euo pipefail
cd "$(dirname "$0")/.."

# Byte patterns, so the check works in any locale; a grep error must fail the gate, not pass it.
set +e
dashes=$(grep -rn $'\xe2\x80\x94\|\xe2\x80\x93' --include='*.md' --include='*.rs' --include='*.toml' . --exclude-dir=target --exclude-dir=.git --exclude-dir=.claude)
rc=$?
set -e
case $rc in
  0) echo "$dashes"; echo "gate: em or en dash found" >&2; exit 1 ;;
  1) ;;
  *) echo "gate: dash check failed to run (grep exit $rc)" >&2; exit 1 ;;
esac
python3 scripts/backlog.py check
[ -f Cargo.toml ] || { echo "gate: no Cargo.toml yet" >&2; exit 1; }
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
if [ $# -gt 0 ]; then python3 scripts/backlog.py verify "$1"; fi
echo "gate: ok"
