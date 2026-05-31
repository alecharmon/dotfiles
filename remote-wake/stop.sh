#!/usr/bin/env bash
# stop.sh — undo what start.sh did. Leaves Tailscale alone.

set -uo pipefail

export PATH="$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

TMUX_SESSION="claude-remote"
CAFFEINATE_PID_FILE="$HOME/.claude-remote-caffeinate.pid"

ok()   { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m!\033[0m %s\n' "$*"; }

echo "→ Caffeinate"
if [ -f "$CAFFEINATE_PID_FILE" ] && kill -0 "$(cat "$CAFFEINATE_PID_FILE")" 2>/dev/null; then
    kill "$(cat "$CAFFEINATE_PID_FILE")" && ok "stopped (pid=$(cat "$CAFFEINATE_PID_FILE"))"
    rm -f "$CAFFEINATE_PID_FILE"
else
    warn "not running"
    rm -f "$CAFFEINATE_PID_FILE"
fi

echo "→ Claude remote-control"
if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
    tmux kill-session -t "$TMUX_SESSION" && ok "killed tmux session '$TMUX_SESSION'"
else
    warn "no tmux session '$TMUX_SESSION'"
fi

echo
echo "Done. Mac is free to sleep again."
