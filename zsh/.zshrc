# Path to your Oh My Zsh installation.
export ZSH="$HOME/.oh-my-zsh"

ZSH_THEME="agnoster"

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
export PNPM_HOME="$HOME/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end
command -v mise >/dev/null 2>&1 && eval "$(mise activate zsh)"

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
[ -s "$HOME/.bun/_bun" ] && source "$HOME/.bun/_bun"

# bun
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"

# Added by Antigravity
export PATH="$HOME/.antigravity/antigravity/bin:$PATH"

# worktrunk shell integration
command -v wt >/dev/null 2>&1 && eval "$(wt config shell init zsh)"

# Local overrides and secrets (not tracked in git)
[ -f "$HOME/.zshrc.local" ] && source "$HOME/.zshrc.local"
export PATH="$HOME/.local/bin:$PATH"


if command -v wt >/dev/null 2>&1; then eval "$(command wt config shell init zsh)"; fi

# opencode
export PATH="$HOME/.opencode/bin:$PATH"

alias claude-mem='bun "$HOME/.claude/plugins/cache/thedotmack/claude-mem/10.5.5/scripts/worker-service.cjs"'
export PATH="$PATH:$HOME/go/bin"

# @ triggers fzf file picker, inserts selected path into command line
# Cancel fzf (Esc/Ctrl-C) to insert a literal @
fzf-file-at() {
  local selected
  selected=$(fd --type f --hidden --exclude .git | \
    awk -F/ '{print NF-1 "\t" $0}' | sort -n | cut -f2- | \
    fzf --height=40% --reverse --border --prompt="@ " \
      --tiebreak=index \
      --bind "ctrl-u:reload(fd --type f --hidden --exclude .git . {q} 2>/dev/null)+clear-query" \
      --header="ctrl-u: search from typed path")
  if [[ -n "$selected" ]]; then
    LBUFFER+="$selected"
  else
    LBUFFER+="@"
  fi
  zle redisplay
}
zle -N fzf-file-at
bindkey '@' fzf-file-at
# CF CLI completions
[[ -f "/Users/alec/.config/cf/completions/_cf.zsh" ]] && source "/Users/alec/.config/cf/completions/_cf.zsh"
