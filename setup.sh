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

build_tmux_tabs() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "==> WARNING: cargo not found; skipping tmux-tabs build. Install Rust (https://rustup.rs)." >&2
        return 0
    fi
    echo "==> Building tmux-tabs (Rust sidebar)..."
    cargo build --release --manifest-path "$DOTFILES_DIR/tmux-tabs/Cargo.toml"
}
build_tmux_tabs

ensure_llm_redactor() {
    if command -v llm-redactor-exec &>/dev/null; then
        return 0
    fi
    if ! command -v go &>/dev/null; then
        echo "==> WARNING: go not found; skipping llm-redactor-exec install for tmux-tabs descriptions."
        return 0
    fi

    echo "==> Installing llm-redactor-exec for tmux-tabs descriptions..."
    go install github.com/wangyihang/llm-redactor/cmd/llm-redactor-exec@latest || \
        echo "==> WARNING: failed to install llm-redactor-exec; tmux-tabs descriptions will be disabled."
}
ensure_llm_redactor

# Configure VS Code CLI standalone to use Code - OSS when available.
# Override CODE_CLI_INSTALL_DIR if Code - OSS is installed outside /Applications.
CODE_CLI_INSTALL_DIR="${CODE_CLI_INSTALL_DIR:-/Applications/Code - OSS.app}"
if command -v code &>/dev/null; then
    echo "==> Configuring code-cli to use Code - OSS..."
    if [ -e "$CODE_CLI_INSTALL_DIR" ]; then
        code version use oss --install-dir "$CODE_CLI_INSTALL_DIR"
    else
        echo "    Code - OSS not found at $CODE_CLI_INSTALL_DIR; skipping code-cli editor selection."
        echo "    Re-run with CODE_CLI_INSTALL_DIR=/path/to/installation once Code - OSS is installed."
    fi
fi

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

# herdr (terminal workspace manager) + its plugins.
# Preview channel is required by herdr-mirror (needs the terminal session streams).
ensure_herdr() {
    if ! command -v herdr &>/dev/null; then
        echo "==> Installing herdr..."
        curl -fsSL https://herdr.dev/install | sh || {
            echo "==> WARNING: herdr install failed; skipping herdr plugins." >&2
            return 0
        }
    fi
    command -v herdr &>/dev/null || return 0
    herdr channel set preview
    echo "==> Installing herdr plugins..."
    for plugin in \
        kakigakki/herdr-auto-namer \
        wyattjoh/herdr-plugin-gh-pr \
        miiraheart/herdr-beads \
        nikok6/herdr-mirror; do
        herdr plugin install -y "$plugin" || echo "==> WARNING: failed to install herdr plugin $plugin" >&2
    done
}
ensure_herdr

# beads: the real bd via mise, fronted by scripts/bd (ssh client for the remote workspace).
ensure_beads() {
    if command -v mise &>/dev/null; then
        echo "==> Installing beads (bd)..."
        mise use -g "github:gastownhall/beads@latest" || echo "==> WARNING: mise failed to install beads." >&2
    else
        echo "==> WARNING: mise not found; skipping beads install." >&2
    fi
    mkdir -p "$HOME/.local/bin"
    ln -sf "$DOTFILES_DIR/scripts/bd" "$HOME/.local/bin/bd"
}
ensure_beads

# Install gh extensions
if command -v gh &>/dev/null; then
    echo "==> Installing gh extensions..."
    gh extension install dlvhdr/gh-dash 2>/dev/null || true
fi

# Install code agent skills from tracked manifest
bash "$DOTFILES_DIR/skills/install.sh"

# Stow all config packages
echo "==> Stowing config packages..."
stow_failures=()
for dir in "$DOTFILES_DIR"/*/; do
    pkg="$(basename "$dir")"
    [ "$pkg" = "scripts" ] && continue
    [ "$pkg" = "skills" ] && continue
    [ "$pkg" = "vscode" ] && continue
    echo "    Stowing $pkg..."
    if ! stow -d "$DOTFILES_DIR" -t "$HOME" "$pkg" 2>&1; then
        stow_failures+=("$pkg")
    fi
done
if [ ${#stow_failures[@]} -gt 0 ]; then
    echo "==> WARNING: Failed to stow: ${stow_failures[*]}"
    echo "    Resolve conflicts manually and re-run."
fi

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
