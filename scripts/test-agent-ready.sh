#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AGENT_READY="$DOTFILES_DIR/scripts/agent-ready"
TEST_CACHE="$(mktemp -d)"
trap 'rm -rf "$TEST_CACHE"' EXIT
export XDG_CACHE_HOME="$TEST_CACHE"

pane='%agent-ready-test'

"$AGENT_READY" clear --pane "$pane" >/dev/null
"$AGENT_READY" set --pane "$pane" --source manual --message 'waiting for input' >/dev/null

marker="$TEST_CACHE/dotfiles/agent-ready/${pane#%}.json"
[ -f "$marker" ] || { echo "missing marker $marker" >&2; exit 1; }

grep -F '"pane": "%agent-ready-test"' "$marker" >/dev/null
grep -F '"source": "manual"' "$marker" >/dev/null
grep -F '"state": "ready"' "$marker" >/dev/null

list_output="$($AGENT_READY list)"
printf '%s\n' "$list_output" | grep -F '%agent-ready-test' >/dev/null
printf '%s\n' "$list_output" | grep -F 'manual' >/dev/null

"$AGENT_READY" clear --pane "$pane" >/dev/null
[ ! -e "$marker" ] || { echo "marker still present after clear" >&2; exit 1; }

unset TMUX_PANE
"$AGENT_READY" set --source no-tmux >/dev/null

echo "agent-ready tests passed"
