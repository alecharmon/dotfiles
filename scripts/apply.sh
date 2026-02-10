#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Applying dotfiles (re-stowing)..."

for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue
    echo "    Stowing $pkg..."
    stow -d "$DOTFILES_DIR" -t "$HOME" -R "$pkg"
done

echo "==> Done."
