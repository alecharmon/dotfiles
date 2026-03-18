#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "==> Setting up dotfiles from $DOTFILES_DIR"

# Install Homebrew if not present
if ! command -v brew &>/dev/null; then
    echo "==> Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(/opt/homebrew/bin/brew shellenv)"
fi

# Install all packages from Brewfile
echo "==> Installing packages from Brewfile..."
brew bundle --file="$DOTFILES_DIR/Brewfile"

# Install Oh My Zsh if not present
if [ ! -d "$HOME/.oh-my-zsh" ]; then
    echo "==> Installing Oh My Zsh..."
    RUNZSH=no sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
fi

# Install custom zsh plugins
ZSH_CUSTOM="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}"
if [ ! -d "$ZSH_CUSTOM/plugins/zsh-fzf-history-search" ]; then
    echo "==> Installing zsh-fzf-history-search plugin..."
    git clone https://github.com/joshskidmore/zsh-fzf-history-search "$ZSH_CUSTOM/plugins/zsh-fzf-history-search"
fi

# Install gh extensions
if command -v gh &>/dev/null; then
    echo "==> Installing gh extensions..."
    gh extension install dlvhdr/gh-dash 2>/dev/null || true
fi

# Stow all config packages
echo "==> Stowing config packages..."
for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue
    [ "$pkg" = "vscode" ] && continue
    echo "    Stowing $pkg..."
    stow -d "$DOTFILES_DIR" -t "$HOME" "$pkg"
done

# SketchyBar dependencies
echo "==> Installing SketchyBar dependencies..."
# App font
if [ ! -f "$HOME/Library/Fonts/sketchybar-app-font.ttf" ]; then
    echo "    Downloading sketchybar-app-font..."
    curl -L https://github.com/kvndrsslr/sketchybar-app-font/releases/download/v2.0.54/sketchybar-app-font.ttf -o "$HOME/Library/Fonts/sketchybar-app-font.ttf"
fi
# SbarLua
if [ ! -f "$HOME/.local/share/sketchybar_lua/sketchybar.so" ]; then
    echo "    Building SbarLua (Lua 5.4 compat)..."
    git clone https://github.com/FelixKratz/SbarLua.git /tmp/SbarLua
    (cd /tmp/SbarLua && git checkout 437bd20 && make install)
    rm -rf /tmp/SbarLua
fi
# Build sketchybar C helpers (menus, event providers)
SKETCHYBAR_HELPERS="$HOME/.config/sketchybar/helpers"
if [ -d "$SKETCHYBAR_HELPERS" ]; then
    echo "    Building sketchybar helpers..."
    (cd "$SKETCHYBAR_HELPERS" && make)
fi

# Screensaver CLI
SCREENSAVER_DIR="$DOTFILES_DIR/../screensaver"
if [ -d "$SCREENSAVER_DIR" ]; then
    echo "==> Building and installing screensaver..."
    (cd "$SCREENSAVER_DIR" && go install ./cmd/...)
else
    echo "==> Screensaver repo not found at $SCREENSAVER_DIR, skipping..."
fi

# Worktrunk shell integration
if command -v wt &>/dev/null; then
    echo "==> Installing worktrunk shell integration..."
    wt config shell install
fi

# VS Code settings (symlinked separately due to non-HOME path)
VSCODE_USER_DIR="$HOME/Library/Application Support/Code/User"
if [ -d "$VSCODE_USER_DIR" ]; then
    echo "==> Linking VS Code settings..."
    ln -sf "$DOTFILES_DIR/vscode/settings.json" "$VSCODE_USER_DIR/settings.json"
    ln -sf "$DOTFILES_DIR/vscode/keybindings.json" "$VSCODE_USER_DIR/keybindings.json"
fi

# Install VS Code extensions
if command -v code &>/dev/null; then
    echo "==> Installing VS Code extensions..."
    while IFS= read -r ext; do
        code --install-extension "$ext" --force 2>/dev/null || true
    done < "$DOTFILES_DIR/vscode/extensions.txt"
fi

echo "==> Done! All dotfiles linked and tools installed."
