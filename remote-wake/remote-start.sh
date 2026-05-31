#!/usr/bin/env bash
# remote-start.sh — find a Mac on the Tailscale network and run start.sh on it.
# Run on the controller (your other computer).
#
#   remote-start.sh                          # interactive pick
#   remote-start.sh <host>                   # SSH straight to host
#   remote-start.sh <host> <project-name>    # pass project name to remote start.sh
#
# Env overrides:
#   REMOTE_WAKE_USER  SSH user on target (default: $USER)
#   REMOTE_WAKE_PATH  path to start.sh on target (default: $HOME/dev/dotfiles/remote-wake/start.sh)

set -uo pipefail

SSH_USER="${REMOTE_WAKE_USER:-$USER}"
REMOTE_PATH="${REMOTE_WAKE_PATH:-\$HOME/dev/dotfiles/remote-wake/start.sh}"

target="${1:-}"
project="${2:-}"

ok()   { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m!\033[0m %s\n' "$*"; }
err()  { printf '\033[31m✗\033[0m %s\n' "$*" >&2; }

find_tailscale() {
    if command -v tailscale >/dev/null; then echo tailscale; return; fi
    local bundled="/Applications/Tailscale.app/Contents/MacOS/Tailscale"
    [ -x "$bundled" ] && echo "$bundled"
}

discover_tailscale_macs() {
    local ts; ts=$(find_tailscale)
    [ -z "$ts" ] && return 1
    # `tailscale status` columns: IP  HOST  USER  OS  STATUS...
    # Include only online macOS peers (exclude "offline").
    "$ts" status 2>/dev/null | awk '$4 == "macOS" && $0 !~ /offline/ {print $1 "\t" $2}'
}

if [ -z "$target" ]; then
    echo "Discovering macOS hosts on Tailscale..."
    hosts=()
    while IFS= read -r line; do
        [ -n "$line" ] && hosts+=("$line")
    done < <(discover_tailscale_macs)

    if [ ${#hosts[@]} -eq 0 ]; then
        err "No macOS Tailscale peers found."
        echo "Pass a hostname or IP directly: $0 <host> [project-name]" >&2
        exit 1
    fi

    echo
    for i in "${!hosts[@]}"; do
        ip=$(printf '%s' "${hosts[$i]}" | cut -f1)
        name=$(printf '%s' "${hosts[$i]}" | cut -f2)
        printf '  %d) %-20s %s\n' "$((i+1))" "$name" "$ip"
    done
    echo
    read -rp "Pick host [1-${#hosts[@]}]: " choice
    if ! [[ "$choice" =~ ^[0-9]+$ ]] || [ "$choice" -lt 1 ] || [ "$choice" -gt ${#hosts[@]} ]; then
        err "invalid choice"
        exit 1
    fi
    target=$(printf '%s' "${hosts[$((choice-1))]}" | cut -f2)  # prefer hostname over IP
fi

echo "→ Probing $SSH_USER@$target"
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$SSH_USER@$target" true 2>/dev/null; then
    err "$target unreachable or SSH refused."
    echo "  - If the Mac is asleep, send a Wake-on-LAN packet first (requires being on its LAN)." >&2
    echo "  - Verify SSH is enabled: System Settings → General → Sharing → Remote Login." >&2
    echo "  - Verify Tailscale auth is current on this machine: tailscale status" >&2
    exit 1
fi
ok "reachable"

echo "→ Running start.sh on $target"
# shellcheck disable=SC2029  # we WANT REMOTE_PATH/project to interpolate locally; $HOME expands on remote
ssh "$SSH_USER@$target" "bash $REMOTE_PATH ${project:+\"$project\"}"
