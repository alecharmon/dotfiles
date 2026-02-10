#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Updating dotfiles..."

# Pull latest changes
echo "==> Pulling latest..."
git -C "$DOTFILES_DIR" pull

# Update Homebrew packages
echo "==> Running brew bundle..."
brew bundle --file="$DOTFILES_DIR/Brewfile"

# Re-stow all config packages
echo "==> Re-stowing config packages..."
for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue
    echo "    Stowing $pkg..."
    stow -d "$DOTFILES_DIR" -t "$HOME" -R "$pkg"
done

echo "==> Update complete."
