#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/handoff-ssh.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAKEBIN="$TMP/bin"
REMOTE="$TMP/remote"
LOG="$TMP/commands.log"
HOME_DIR="$TMP/home"
mkdir -p "$FAKEBIN" "$REMOTE" "$HOME_DIR/.pi/agent" "$HOME_DIR/.ssh" "$HOME_DIR/.config/gh" "$TMP/repo"

cat > "$HOME_DIR/.pi/agent/auth.json" <<'JSON'
{"token":"secret"}
JSON
cat > "$HOME_DIR/.pi/agent/settings.json" <<'JSON'
{"provider":"openai-codex"}
JSON
cat > "$HOME_DIR/.gitconfig" <<'EOF_GIT'
[user]
  name = Test
EOF_GIT
cat > "$HOME_DIR/.ssh/id_ed25519" <<'EOF_KEY'
secret-key
EOF_KEY
cat > "$HOME_DIR/.config/gh/hosts.yml" <<'EOF_GH'
github.com:
  oauth_token: secret
EOF_GH
cat > "$TMP/session.jsonl" <<'EOF_SESSION'
{"type":"session","version":3,"id":"test","cwd":"/tmp/repo"}
EOF_SESSION

cat > "$FAKEBIN/ssh" <<'EOF_SSH'
#!/usr/bin/env bash
set -euo pipefail
printf 'ssh %q' "$1" >> "$LOG"
shift
for arg in "$@"; do printf ' %q' "$arg" >> "$LOG"; done
printf '\n' >> "$LOG"
exit 0
EOF_SSH

cat > "$FAKEBIN/rsync" <<'EOF_RSYNC'
#!/usr/bin/env bash
set -euo pipefail
printf 'rsync' >> "$LOG"
for arg in "$@"; do printf ' %q' "$arg" >> "$LOG"; done
printf '\n' >> "$LOG"
exit 0
EOF_RSYNC

chmod +x "$FAKEBIN/ssh" "$FAKEBIN/rsync"

PATH="$FAKEBIN:$PATH" HOME="$HOME_DIR" LOG="$LOG" \
  "$SCRIPT" user@example.com \
  --cwd /remote/repo \
  --session "$TMP/session.jsonl" \
  --prompt "Continue this task" \
  --headless \
  --yes

assert_log_contains() {
  local needle="$1"
  if ! grep -F -- "$needle" "$LOG" >/dev/null; then
    echo "Expected log to contain: $needle" >&2
    echo "--- log ---" >&2
    cat "$LOG" >&2
    exit 1
  fi
}

assert_log_contains "mkdir -p ~/.pi/agent ~/.ssh ~/.config/gh"
assert_log_contains "/remote/repo"
assert_log_contains "/tmp/pi-handoff"
assert_log_contains "npm install -g @earendil-works/pi-coding-agent"
assert_log_contains "$HOME_DIR/.pi/agent/auth.json"
assert_log_contains "$HOME_DIR/.pi/agent/settings.json"
assert_log_contains "$HOME_DIR/.ssh/"
assert_log_contains "$HOME_DIR/.config/gh/"
assert_log_contains "$HOME_DIR/.gitconfig"
assert_log_contains "$TMP/session.jsonl"
assert_log_contains "pi\\ --fork"
assert_log_contains "Continue\\ this\\ task"

echo "handoff-ssh tests passed"
