#!/usr/bin/env zsh
set -euo pipefail

start_dir="${TMUX_RECENT_PROJECT_START_DIR:-}"
if [[ -z "$start_dir" ]]; then
  if [[ -d "$HOME/dev" ]]; then
    start_dir="$HOME/dev"
  else
    start_dir="$HOME"
  fi
fi

cd "$start_dir"

selected="${TMUX_RECENT_PROJECT_SELECTION-unset}"
if [[ "$selected" == "unset" ]]; then
  selected="$(tv recent-projects --no-sort || true)"
fi

if [[ -n "$selected" && -d "$selected" ]]; then
  cd "$selected"
fi

if [[ "${TMUX_RECENT_PROJECT_TEST:-}" == "1" ]]; then
  pwd
  exit 0
fi

exec "${SHELL:-zsh}" -l
