#!/usr/bin/env bash
set -euo pipefail

ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

SCRIPT="$PWD/television/.config/television/scripts/recent-projects.zsh"

make_repo() {
  local path="$1"
  local when="$2"
  local subject="$3"
  mkdir -p "$path"
  git -C "$path" init -q
  git -C "$path" config user.email test@example.com
  git -C "$path" config user.name Test
  echo "$subject" > "$path/file.txt"
  git -C "$path" add file.txt
  GIT_AUTHOR_DATE="$when" GIT_COMMITTER_DATE="$when" git -C "$path" commit -q -m "$subject"
}

make_repo "$ROOT/older" "2024-01-01T00:00:00Z" "older commit"
make_repo "$ROOT/nested/newer" "2024-02-01T00:00:00Z" "newer commit"
make_repo "$ROOT/nested/newer/.worktrees/ignored" "2024-03-01T00:00:00Z" "ignored worktree commit"

list_output="$(RECENT_PROJECTS_ROOT="$ROOT" "$SCRIPT" list)"
printf '%s\n' "$list_output" | grep -q "ignored" && { echo "expected .worktrees repos to be ignored" >&2; exit 1; }
first_path="$(printf '%s\n' "$list_output" | head -n1 | cut -f1)"
first_name="$(printf '%s\n' "$list_output" | head -n1 | cut -f2)"
second_path="$(printf '%s\n' "$list_output" | sed -n '2p' | cut -f1)"
second_name="$(printf '%s\n' "$list_output" | sed -n '2p' | cut -f2)"

[[ "$first_path" == "$ROOT/nested/newer" ]] || { echo "expected newest repo first, got: $first_path" >&2; exit 1; }
[[ "$first_name" == "newer" ]] || { echo "expected newest repo display name, got: $first_name" >&2; exit 1; }
[[ "$second_path" == "$ROOT/older" ]] || { echo "expected older repo second, got: $second_path" >&2; exit 1; }
[[ "$second_name" == "older" ]] || { echo "expected older repo display name, got: $second_name" >&2; exit 1; }

preview_output="$("$SCRIPT" preview "$ROOT/nested/newer")"
printf '%s\n' "$preview_output" | grep -q "newer commit" || { echo "preview missing commit subject" >&2; exit 1; }
printf '%s\n' "$preview_output" | grep -q "Branch:" || { echo "preview missing branch" >&2; exit 1; }

echo "recent-projects tests passed"
