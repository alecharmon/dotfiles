#!/usr/bin/env bash
set -euo pipefail

# Linux counterpart to setup.sh — installs the tmux/zsh tooling and links the
# (cross-platform) configs. Scope: core only (zsh, fzf, fd). No homebrew, no
# mise/worktrunk/sketchybar/aerospace (those are mac-only here).

DOTFILES_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "==> Setting up dotfiles (linux) from $DOTFILES_DIR"

# Root vs. sudo
if [ "$(id -u)" -eq 0 ]; then SUDO=""; else SUDO="sudo"; fi

# Require apt (this script targets Debian/Ubuntu)
if ! command -v apt-get &>/dev/null; then
    echo "!! This script targets apt-based distros. Install zsh, fzf and fd manually, then re-run the linking steps." >&2
    exit 1
fi

# Install core tools
echo "==> Installing core packages (zsh, fzf, fd, python pip)..."
$SUDO apt-get update -qq
$SUDO apt-get install -y zsh fzf fd-find python3-pip

ensure_python_textual() {
    if python3 -c 'import textual' >/dev/null 2>&1; then
        return 0
    fi

    echo "==> Installing Python Textual for tmux-tabs..."
    python3 -m pip install --user textual || \
        python3 -m pip install --user --break-system-packages textual
}
ensure_python_textual

# Debian/Ubuntu ship fd as 'fdfind'; the configs call 'fd'. Bridge it.
mkdir -p "$HOME/.local/bin"
if command -v fdfind &>/dev/null && ! command -v fd &>/dev/null; then
    echo "==> Linking fd -> fdfind in ~/.local/bin..."
    ln -sf "$(command -v fdfind)" "$HOME/.local/bin/fd"
fi

# Install Oh My Zsh if not present (keep our .zshrc, don't chsh here)
if [ ! -d "$HOME/.oh-my-zsh" ]; then
    echo "==> Installing Oh My Zsh..."
    RUNZSH=no CHSH=no KEEP_ZSHRC=yes \
        sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
fi

# Install custom zsh plugins
ZSH_CUSTOM="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}"
if [ ! -d "$ZSH_CUSTOM/plugins/zsh-fzf-history-search" ]; then
    echo "==> Installing zsh-fzf-history-search plugin..."
    git clone https://github.com/joshskidmore/zsh-fzf-history-search \
        "$ZSH_CUSTOM/plugins/zsh-fzf-history-search"
fi

# Link configs into $HOME (back up any real files that are in the way)
echo "==> Linking zsh + tmux configs..."
link() {
    local src="$1" dest="$2"
    if [ -e "$dest" ] && [ ! -L "$dest" ]; then
        echo "    Backing up existing $dest -> $dest.bak"
        mv "$dest" "$dest.bak"
    fi
    ln -sfn "$src" "$dest"
    echo "    $dest -> $src"
}
link "$DOTFILES_DIR/zsh/.zshrc"     "$HOME/.zshrc"
link "$DOTFILES_DIR/zsh/.zprofile"  "$HOME/.zprofile"
link "$DOTFILES_DIR/tmux/.tmux.conf" "$HOME/.tmux.conf"

# Make zsh the login shell
ZSH_BIN="$(command -v zsh)"
if [ "${SHELL:-}" != "$ZSH_BIN" ]; then
    echo "==> Setting zsh as login shell..."
    grep -qxF "$ZSH_BIN" /etc/shells || echo "$ZSH_BIN" | $SUDO tee -a /etc/shells >/dev/null
    chsh -s "$ZSH_BIN" || echo "    chsh failed; run 'chsh -s $ZSH_BIN' manually."
fi

echo "==> Done! Open a new shell (or run 'zsh') to start using it."
