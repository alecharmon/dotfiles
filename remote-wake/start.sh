#!/usr/bin/env bash
# start.sh — make this Mac reachable + awake + running Claude Code remote-control.
# Run on the Mac itself (locally, over SSH, or invoked by remote-start.sh).
#
#   start.sh [project-name]
#
# Idempotent: safe to re-run; reports each component's state.

set -uo pipefail

PROJECT_NAME="${1:-$(basename "$HOME")}"
TMUX_SESSION="claude-remote"
CAFFEINATE_PID_FILE="$HOME/.claude-remote-caffeinate.pid"

ok()   { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m!\033[0m %s\n' "$*"; }
err()  { printf '\033[31m✗\033[0m %s\n' "$*" >&2; }

# Find the Tailscale CLI. Falls back to the Mac App Store bundled binary.
find_tailscale() {
    if command -v tailscale >/dev/null; then echo tailscale; return; fi
    local bundled="/Applications/Tailscale.app/Contents/MacOS/Tailscale"
    [ -x "$bundled" ] && echo "$bundled"
}

echo "→ Tailscale"
TS=$(find_tailscale)
if [ -z "$TS" ]; then
    err "tailscale not found (install Tailscale.app or 'brew install tailscale')"
else
    if "$TS" ip -4 >/dev/null 2>&1; then
        ok "up ($("$TS" ip -4 | head -n1))"
    else
        warn "not connected — attempting 'tailscale up'"
        if "$TS" up 2>/dev/null && "$TS" ip -4 >/dev/null 2>&1; then
            ok "started ($("$TS" ip -4 | head -n1))"
        elif sudo -n "$TS" up 2>/dev/null && "$TS" ip -4 >/dev/null 2>&1; then
            ok "started with sudo ($("$TS" ip -4 | head -n1))"
        else
            err "could not start Tailscale (open Tailscale.app or run: sudo $TS up)"
        fi
    fi
fi

echo "→ Sleep prevention (caffeinate)"
if [ -f "$CAFFEINATE_PID_FILE" ] && kill -0 "$(cat "$CAFFEINATE_PID_FILE")" 2>/dev/null; then
    ok "already running (pid=$(cat "$CAFFEINATE_PID_FILE"))"
else
    nohup caffeinate -dimsu >/dev/null 2>&1 &
    echo $! > "$CAFFEINATE_PID_FILE"
    disown 2>/dev/null || true
    ok "started (pid=$(cat "$CAFFEINATE_PID_FILE"))"
fi

echo "→ Claude remote-control"
if ! command -v claude >/dev/null; then
    err "claude CLI not on PATH"
elif ! command -v tmux >/dev/null; then
    err "tmux not installed (brew install tmux)"
elif tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
    ok "tmux session '$TMUX_SESSION' already exists"
else
    tmux new-session -d -s "$TMUX_SESSION" "claude --remote-control '$PROJECT_NAME'"
    sleep 1
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        ok "started 'claude --remote-control \"$PROJECT_NAME\"' in tmux '$TMUX_SESSION'"
    else
        err "tmux session failed — try manually: tmux new -s $TMUX_SESSION 'claude --remote-control \"$PROJECT_NAME\"'"
    fi
fi

echo
echo "Ready. Visit claude.ai/code to control the session."
echo "  Attach locally: tmux attach -t $TMUX_SESSION"
echo "  Tear down:      $(dirname "$0")/stop.sh"
