#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Pull latest changes
echo "==> Pulling latest..."
git -C "$DOTFILES_DIR" pull

# Update Homebrew packages
echo "==> Running brew bundle..."
brew bundle --file="$DOTFILES_DIR/Brewfile"

# Re-stow all config packages
echo "==> Stowing config packages..."
stow_failures=()
for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue
    [ "$pkg" = "vscode" ] && continue
    echo "    Stowing $pkg..."
    if ! stow -d "$DOTFILES_DIR" -t "$HOME" -R "$pkg" 2>&1; then
        stow_failures+=("$pkg")
    fi
done
if [ ${#stow_failures[@]} -gt 0 ]; then
    echo "==> WARNING: Failed to stow: ${stow_failures[*]}"
    echo "    Resolve conflicts manually and re-run."
fi

echo "==> Done."
