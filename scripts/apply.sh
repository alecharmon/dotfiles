#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Pull latest changes
echo "==> Pulling latest..."
git -C "$DOTFILES_DIR" pull

# Update Homebrew packages
echo "==> Running brew bundle..."
brew bundle --file="$DOTFILES_DIR/Brewfile"

# Install/update gruvbox zsh theme
GRUVBOX_DIR="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/gruvbox"
if [ -d "$GRUVBOX_DIR" ]; then
    echo "==> Updating gruvbox zsh theme..."
    git -C "$GRUVBOX_DIR" pull
else
    echo "==> Installing gruvbox zsh theme..."
    git clone https://github.com/sbugzu/gruvbox-zsh.git "$GRUVBOX_DIR"
fi

# Re-stow all config packages
echo "==> Stowing config packages..."
for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue

    # Check top-level entries for conflicts (stow symlinks at this level)
    conflicts=()
    for entry in "$dir"*; do
        rel_path="${entry#"$dir"}"
        target="$HOME/$rel_path"
        if [ -e "$target" ] && [ ! -L "$target" ]; then
            conflicts+=("$target")
        fi
    done

    if [ ${#conflicts[@]} -gt 0 ]; then
        echo "    ⚠ $pkg: the following files/folders already exist and are not symlinks:"
        for f in "${conflicts[@]}"; do
            echo "      - $f"
        done
        read -rp "    Delete these and use repo version? [Y/n] " answer
        if [[ "$answer" =~ ^[Nn]$ ]]; then
            echo "    Skipping $pkg."
            continue
        fi
        for f in "${conflicts[@]}"; do
            rm -rf "$f"
            echo "      Deleted $f"
        done
    fi

    echo "    Stowing $pkg..."
    stow -d "$DOTFILES_DIR" -t "$HOME" -R "$pkg"
done

echo "==> Done."
