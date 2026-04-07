#!/usr/bin/env bash
set -euo pipefail

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PACKAGES_FILE="$SKILLS_DIR/packages.txt"

if [ ! -f "$PACKAGES_FILE" ]; then
    echo "==> No skills package manifest found at $PACKAGES_FILE, skipping..."
    exit 0
fi

if ! command -v npx &>/dev/null; then
    echo "==> WARNING: npx not found, skipping code agent skills."
    exit 0
fi

echo "==> Installing code agent skills..."
skill_failures=()
while IFS= read -r line || [ -n "$line" ]; do
    [ -z "$line" ] && continue
    case "$line" in
        \#*) continue ;;
    esac

    if [[ "$line" == cmd:* ]]; then
        command_line="${line#cmd: }"
        read -r -a command_args <<< "$command_line"

        echo "    Running ${command_args[*]}..."
        if ! "${command_args[@]}"; then
            skill_failures+=("$line")
        fi
        continue
    fi

    read -r -a skill_args <<< "$line"

    echo "    Installing ${skill_args[*]}..."
    if ! npx --yes skills add "${skill_args[@]}" -g -a codex opencode gemini-cli claude-code -y; then
        skill_failures+=("$line")
    fi
done < "$PACKAGES_FILE"

if [ ${#skill_failures[@]} -gt 0 ]; then
    echo "==> WARNING: Failed to install skills: ${skill_failures[*]}"
    echo "    Resolve the failures manually and re-run setup."
fi
