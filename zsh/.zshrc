# Path to your Oh My Zsh installation.
export ZSH="$HOME/.oh-my-zsh"

ZSH_THEME="gruvbox"
SOLARIZED_THEME="dark"

# Plugins
plugins=(git zsh-fzf-history-search)
source $ZSH/oh-my-zsh.sh

# Show only current directory name in prompt (not full path)
PROMPT="${PROMPT//%~/%1~}"

# Replace OS icon with 🤙, show ❌ if last command failed
prompt_context() {
  if [[ $RETVAL -ne 0 ]]; then
    prompt_segment 237 7 "❌"
  else
    prompt_segment 237 7 "🤙"
  fi
}

# Remove broken heart error indicator
prompt_status() {
  local -a symbols
  [[ $UID -eq 0 ]] && symbols+="%{%F{11}%}\ue77a"
  [[ $(jobs -l | wc -l) -gt 0 ]] && symbols+="%{%F{15}%}\ufb36"
  [[ -n "$symbols" ]] && prompt_segment 166 7 "$symbols"
}

# pnpm
export PNPM_HOME="/Users/alecharmon/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end
eval "$(/opt/homebrew/bin/mise activate zsh)"

### ALIASES
alias dc="docker compose"
alias dcu="docker compose up -d"
alias dcb="docker-compose exec code bash"
alias grt='cd "$(git rev-parse --show-toplevel)"'

export HISTFILE="$HOME/.zsh_history"
export HISTSIZE=1000
export SAVEHIST=1000

fern_dev() {
  FERN_NO_VERSION_REDIRECTION=true node --enable-source-maps ~/dev/fern/fern/packages/cli/cli/dist/prod/cli.cjs "$@"
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

# worktrunk shell integration
eval "$(wt config shell init zsh)"

# Local overrides and secrets (not tracked in git)
[ -f "$HOME/.zshrc.local" ] && source "$HOME/.zshrc.local"
export PATH="$HOME/.local/bin:$PATH"


if command -v wt >/dev/null 2>&1; then eval "$(command wt config shell init zsh)"; fi

# opencode
export PATH=/Users/alecharmon/.opencode/bin:$PATH

alias claude-mem='bun "/Users/alecharmon/.claude/plugins/cache/thedotmack/claude-mem/10.5.5/scripts/worker-service.cjs"'
