#!/usr/bin/env bash
set -euo pipefail

SCRIPT="$PWD/scripts/tmux-hunk-toggle"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

FAKE_BIN="$ROOT/bin"
LOG="$ROOT/tmux.log"
mkdir -p "$FAKE_BIN"

cat > "$FAKE_BIN/tmux" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$TMUX_FAKE_LOG"
case "${1:-}" in
  list-panes)
    printf '%s\n' "${TMUX_FAKE_PANES:-}"
    ;;
  split-window)
    printf '%%9\n'
    ;;
esac
FAKE
chmod +x "$FAKE_BIN/tmux"

export PATH="$FAKE_BIN:$PATH"
export TMUX_FAKE_LOG="$LOG"

TMUX_FAKE_PANES="" "$SCRIPT" "/tmp/project with spaces" "@1"
grep -Fx 'list-panes -t @1 -F #{pane_id}:#{pane_title}' "$LOG" >/dev/null || {
  echo "expected script to inspect panes in the current window" >&2
  cat "$LOG" >&2
  exit 1
}
grep -Fx 'split-window -h -P -F #{pane_id} -t @1 -c /tmp/project with spaces' "$LOG" >/dev/null || {
  echo "expected script to open an interactive vertical split at the supplied path" >&2
  cat "$LOG" >&2
  exit 1
}
grep -Fx 'select-pane -t %9 -T hunk' "$LOG" >/dev/null || {
  echo "expected script to label the new hunk pane" >&2
  cat "$LOG" >&2
  exit 1
}
grep -Fx 'send-keys -t %9 hunk diff --watch C-m' "$LOG" >/dev/null || {
  echo "expected script to run hunk in the interactive split" >&2
  cat "$LOG" >&2
  exit 1
}

: > "$LOG"
TMUX_FAKE_PANES=$'%7:hunk\n%8:shell' "$SCRIPT" "/tmp/project" "@1"
grep -Fx 'kill-pane -t %7' "$LOG" >/dev/null || {
  echo "expected script to close existing hunk pane" >&2
  cat "$LOG" >&2
  exit 1
}
if grep -F 'split-window' "$LOG" >/dev/null; then
  echo "expected script not to open another hunk pane when one exists" >&2
  cat "$LOG" >&2
  exit 1
fi

echo "tmux hunk toggle tests passed"
