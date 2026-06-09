#!/usr/bin/env zsh
set -euo pipefail

ROOT="${RECENT_PROJECTS_ROOT:-$HOME/dev}"

format_ts() {
  local ts="$1"
  if date -r "$ts" "+%Y-%m-%d %H:%M" >/dev/null 2>&1; then
    date -r "$ts" "+%Y-%m-%d %H:%M"
  else
    date -d "@$ts" "+%Y-%m-%d %H:%M"
  fi
}

repo_roots() {
  [[ -d "$ROOT" ]] || return 0

  if command -v fd >/dev/null 2>&1; then
    fd --hidden --no-ignore-vcs --type directory --type file '^\.git$' "$ROOT" \
      --exclude node_modules \
      --exclude .worktrees \
      --exclude .next \
      --exclude dist \
      --exclude build \
      --exclude target \
      --exclude .cache \
      --exclude vendor
  else
    find "$ROOT" \
      \( -type d \( -name node_modules -o -name .worktrees -o -name .next -o -name dist -o -name build -o -name target -o -name .cache -o -name vendor \) -prune \) -o \
      \( -type d -name .git -print -prune \) -o \
      \( -type f -name .git -print \)
  fi |
  while IFS= read -r git_entry; do
    dirname "$git_entry"
  done |
  sort -u
}

list_repos() {
  local rows=()
  local repo repo_name ts date branch subject

  while IFS= read -r repo; do
    git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 || continue
    ts="$(git -C "$repo" log -1 --format=%ct 2>/dev/null || true)"
    [[ -n "$ts" ]] || continue
    repo_name="$(basename "$repo" | tr '\t' ' ')"
    date="$(format_ts "$ts")"
    branch="$(git -C "$repo" branch --show-current 2>/dev/null || true)"
    [[ -n "$branch" ]] || branch="detached"
    subject="$(git -C "$repo" log -1 --format=%s 2>/dev/null | tr '\t' ' ')"
    rows+=("$ts	$repo	$repo_name	$date	$branch	$subject")
  done < <(repo_roots)

  if (( ${#rows[@]} == 0 )); then
    return 0
  fi

  printf '%s\n' "${rows[@]}" |
    sort -t $'\t' -k1,1nr |
    cut -f2-
}

preview_repo() {
  local repo="${1:-}"
  if [[ -z "$repo" || ! -d "$repo" ]]; then
    echo "No repository selected"
    return 0
  fi

  echo "Repository: $repo"
  echo "Branch: $(git -C "$repo" branch --show-current 2>/dev/null || echo detached)"
  echo
  echo "Status:"
  local repo_status
  repo_status="$(git -C "$repo" status --short 2>/dev/null || true)"
  if [[ -n "$repo_status" ]]; then
    printf '%s\n' "$repo_status"
  else
    echo "clean"
  fi
  echo
  echo "Recent commits:"
  git -C "$repo" log --date=short --pretty=format:'%C(yellow)%h%Creset %Cgreen%ad%Creset %s %C(dim white)%an%Creset' -n 8 2>/dev/null || true
  echo
}

case "${1:-list}" in
  list)
    list_repos
    ;;
  preview)
    shift || true
    preview_repo "${1:-}"
    ;;
  *)
    echo "usage: $0 [list|preview <repo>]" >&2
    exit 2
    ;;
esac
