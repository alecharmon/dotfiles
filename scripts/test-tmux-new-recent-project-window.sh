#!/usr/bin/env bash
set -euo pipefail

SCRIPT="$PWD/scripts/tmux-new-recent-project-window.zsh"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT/dev" "$ROOT/repo"

selected_output="$(TMUX_RECENT_PROJECT_START_DIR="$ROOT/dev" TMUX_RECENT_PROJECT_SELECTION="$ROOT/repo" TMUX_RECENT_PROJECT_TEST=1 "$SCRIPT")"
[[ "$selected_output" == "$ROOT/repo" ]] || { echo "expected selected repo cwd, got: $selected_output" >&2; exit 1; }

cancel_output="$(TMUX_RECENT_PROJECT_START_DIR="$ROOT/dev" TMUX_RECENT_PROJECT_SELECTION="" TMUX_RECENT_PROJECT_TEST=1 "$SCRIPT")"
[[ "$cancel_output" == "$ROOT/dev" ]] || { echo "expected start dir on cancel, got: $cancel_output" >&2; exit 1; }

echo "tmux recent project window tests passed"
