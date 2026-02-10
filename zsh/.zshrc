# Path to your Oh My Zsh installation.
export ZSH="$HOME/.oh-my-zsh"

ZSH_THEME="robbyrussell"

# Plugins
plugins=(git zsh-fzf-history-search)
source $ZSH/oh-my-zsh.sh

# pnpm
export PNPM_HOME="/Users/alecharmon/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end
eval "$(/Users/alecharmon/.local/bin/mise activate zsh)"

### ALIASES
alias dc="docker compose"
alias dcu="docker compose up -d"
alias dcb="docker-compose exec code bash"

export HISTFILE="$HOME/.zsh_history"
export HISTSIZE=1000
export SAVEHIST=1000

fern_dev() {
  FERN_NO_VERSION_REDIRECTION=true node --enable-source-maps ~/dev/fern/fern/packages/cli/cli/dist/prod-unminified/cli.cjs "$@"
}

# bun completions
[ -s "/Users/alecharmon/.bun/_bun" ] && source "/Users/alecharmon/.bun/_bun"

# bun
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"

# carapace auto complete
autoload -U compinit && compinit
export CARAPACE_BRIDGES='zsh,fish,bash,inshellisense' # optional
zstyle ':completion:*' format $'\e[2;37mCompleting %d\e[m'
source <(carapace _carapace)

# Added by Antigravity
export PATH="/Users/alecharmon/.antigravity/antigravity/bin:$PATH"

# Local overrides and secrets (not tracked in git)
[ -f "$HOME/.zshrc.local" ] && source "$HOME/.zshrc.local"
