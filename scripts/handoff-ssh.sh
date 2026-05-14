#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/handoff-ssh.sh <ssh-host> [options]

Fast/insecure Pi handoff bootstrap for a trusted remote host. Copies Pi auth,
Git/SSH/GitHub auth, the selected Pi session, then launches Pi remotely.

Options:
  --cwd <path>        Remote working directory for Pi (default: same as local cwd)
  --session <path>    Pi session JSONL to transfer (default: newest ~/.pi/agent/sessions/*.jsonl)
  --prompt <text>     Prompt for headless mode (default: Continue this task from the transferred Pi session.)
  --headless          Run remote Pi with -p prompt instead of interactive TTY
  --tmux [name]       Start remote Pi in tmux session (default name: pi-handoff)
  --remote-dir <path> Remote temp dir for session (default: /tmp/pi-handoff)
  --yes               Skip confirmation about copying secrets
  -h, --help          Show this help

Examples:
  scripts/handoff-ssh.sh box --cwd ~/dev/repo
  scripts/handoff-ssh.sh box --cwd ~/dev/repo --headless --prompt "finish the task"
  scripts/handoff-ssh.sh box --cwd ~/dev/repo --tmux pi-work
EOF
}

fail() {
  echo "handoff-ssh: $*" >&2
  exit 1
}

shell_quote() {
  printf "'%s'" "${1//\'/\'\\\'\'}"
}

latest_session() {
  find "$HOME/.pi/agent/sessions" -type f -name '*.jsonl' -print0 2>/dev/null \
    | xargs -0 ls -t 2>/dev/null \
    | head -n 1
}

HOST=""
REMOTE_CWD="$(pwd)"
SESSION_FILE=""
PROMPT="Continue this task from the transferred Pi session."
HEADLESS=0
TMUX_NAME=""
REMOTE_DIR="/tmp/pi-handoff"
YES=0

while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --cwd)
      [ $# -ge 2 ] || fail "--cwd requires a path"
      REMOTE_CWD="$2"
      shift 2
      ;;
    --session)
      [ $# -ge 2 ] || fail "--session requires a path"
      SESSION_FILE="$2"
      shift 2
      ;;
    --prompt)
      [ $# -ge 2 ] || fail "--prompt requires text"
      PROMPT="$2"
      shift 2
      ;;
    --headless)
      HEADLESS=1
      shift
      ;;
    --tmux)
      if [ $# -ge 2 ] && [[ "$2" != --* ]]; then
        TMUX_NAME="$2"
        shift 2
      else
        TMUX_NAME="pi-handoff"
        shift
      fi
      ;;
    --remote-dir)
      [ $# -ge 2 ] || fail "--remote-dir requires a path"
      REMOTE_DIR="$2"
      shift 2
      ;;
    --yes)
      YES=1
      shift
      ;;
    --*)
      fail "unknown option: $1"
      ;;
    *)
      if [ -z "$HOST" ]; then
        HOST="$1"
        shift
      else
        fail "unexpected argument: $1"
      fi
      ;;
  esac
done

[ -n "$HOST" ] || { usage; exit 1; }

if [ -z "$SESSION_FILE" ]; then
  SESSION_FILE="$(latest_session || true)"
fi
[ -n "$SESSION_FILE" ] || fail "no session file found; pass --session <path>"
[ -f "$SESSION_FILE" ] || fail "session file not found: $SESSION_FILE"

if [ "$YES" -ne 1 ]; then
  cat <<EOF
About to copy secrets to $HOST:
  - ~/.pi/agent/auth.json
  - ~/.pi/agent/settings.json
  - ~/.ssh/
  - ~/.gitconfig
  - ~/.config/gh/
  - Pi session: $SESSION_FILE
EOF
  read -r -p "Continue? [y/N] " answer
  case "$answer" in
    y|Y|yes|YES) ;;
    *) fail "cancelled" ;;
  esac
fi

REMOTE_SESSION="$REMOTE_DIR/session.jsonl"
REMOTE_CWD_Q="$(shell_quote "$REMOTE_CWD")"
REMOTE_DIR_Q="$(shell_quote "$REMOTE_DIR")"
REMOTE_SESSION_Q="$(shell_quote "$REMOTE_SESSION")"
PROMPT_Q="$(shell_quote "$PROMPT")"

BOOTSTRAP="set -e
mkdir -p ~/.pi/agent ~/.ssh ~/.config/gh $REMOTE_CWD_Q $REMOTE_DIR_Q
if ! command -v pi >/dev/null 2>&1; then
  if command -v npm >/dev/null 2>&1; then
    npm install -g @earendil-works/pi-coding-agent
  elif command -v mise >/dev/null 2>&1; then
    mise use -g node@latest
    npm install -g @earendil-works/pi-coding-agent
  else
    curl -fsSL https://mise.run | sh
    export PATH=\"\$HOME/.local/bin:\$HOME/.local/share/mise/shims:\$PATH\"
    mise use -g node@latest
    npm install -g @earendil-works/pi-coding-agent
  fi
fi
command -v git >/dev/null 2>&1 || echo 'WARNING: git is not installed on remote host' >&2
command -v pi >/dev/null
chmod 700 ~/.ssh || true
find ~/.ssh -type f -exec chmod 600 {} + 2>/dev/null || true
ssh -T git@github.com >/dev/null 2>&1 || true
gh auth status >/dev/null 2>&1 || true"

echo "==> Checking SSH and bootstrapping remote Pi on $HOST..."
ssh "$HOST" "$BOOTSTRAP"

copy_if_exists() {
  local src="$1"
  local dest="$2"
  if [ -e "$src" ]; then
    rsync -a "$src" "$HOST:$dest"
  else
    echo "==> Skipping missing $src"
  fi
}

echo "==> Copying Pi, Git, SSH, and GitHub auth..."
copy_if_exists "$HOME/.pi/agent/auth.json" "~/.pi/agent/auth.json"
copy_if_exists "$HOME/.pi/agent/settings.json" "~/.pi/agent/settings.json"
copy_if_exists "$HOME/.gitconfig" "~/.gitconfig"
copy_if_exists "$HOME/.ssh/" "~/.ssh/"
copy_if_exists "$HOME/.config/gh/" "~/.config/gh/"

echo "==> Copying Pi session..."
rsync -a "$SESSION_FILE" "$HOST:$REMOTE_SESSION"

REMOTE_FIX_PERMS="chmod 700 ~/.ssh || true; find ~/.ssh -type f -exec chmod 600 {} + 2>/dev/null || true"
ssh "$HOST" "$REMOTE_FIX_PERMS"

PI_CMD="cd $REMOTE_CWD_Q && pi --fork $REMOTE_SESSION_Q"
if [ "$HEADLESS" -eq 1 ]; then
  PI_CMD="$PI_CMD -p $PROMPT_Q"
fi

if [ -n "$TMUX_NAME" ]; then
  TMUX_NAME_Q="$(shell_quote "$TMUX_NAME")"
  echo "==> Starting remote tmux session $TMUX_NAME on $HOST..."
  ssh "$HOST" "command -v tmux >/dev/null || { echo 'tmux not installed' >&2; exit 1; }; tmux new-session -Ad -s $TMUX_NAME_Q $(shell_quote "$PI_CMD"); tmux attach -t $TMUX_NAME_Q"
elif [ "$HEADLESS" -eq 1 ]; then
  echo "==> Running remote headless Pi on $HOST..."
  ssh "$HOST" "$PI_CMD"
else
  echo "==> Launching remote interactive Pi on $HOST..."
  ssh -t "$HOST" "$PI_CMD"
fi
