zmodload zsh/zprof
# If you come from bash you might have to change your $PATH.
#export PATH=$HOME/bin:/usr/local/bin:$PATH

#PHP REPL show version 
export PHPBREW_SET_PROMPT=1
[[ -e ~/.phpbrew/bashrc ]] && source ~/.phpbrew/bashrc

# Path to your oh-my-zsh installation.
export ZSH=/Users/aharmon/.oh-my-zsh

# Set name of the theme to load. Optionally, if you set this to "random"
# it'll load a random theme each time that oh-my-zsh is loaded.
# See https://github.com/robbyrussell/oh-my-zsh/wiki/Themes
ZSH_THEME="agkozak"

# Uncomment the following line to use case-sensitive completion.
# CASE_SENSITIVE="true"

# Uncomment the following line to use hyphen-insensitive completion. Case
# sensitive completion must be off. _ and - will be interchangeable.
# HYPHEN_INSENSITIVE="true"

# Uncomment the following line to disable bi-weekly auto-update checks.
# DISABLE_AUTO_UPDATE="true"

# Uncomment the following line to change how often to auto-update (in days).
# export UPDATE_ZSH_DAYS=13

# Uncomment the following line to disable colors in ls.
# DISABLE_LS_COLORS="true"

# Uncomment the following line to disable auto-setting terminal title.
# DISABLE_AUTO_TITLE="true"

# Uncomment the following line to enable command auto-correction.
# ENABLE_CORRECTION="true"

# Uncomment the following line to display red dots whilst waiting for completion.
# COMPLETION_WAITING_DOTS="true"

# Uncomment the following line if you want to disable marking untracked files
# under VCS as dirty. This makes repository status check for large repositories
# much, much faster.
# DISABLE_UNTRACKED_FILES_DIRTY="true"

# Uncomment the following line if you want to change the command execution time
# stamp shown in the history command output.
# The optional three formats: "mm/dd/yyyy"|"dd.mm.yyyy"|"yyyy-mm-dd"
# HIST_STAMPS="mm/dd/yyyy"

# Would you like to use another custom folder than $ZSH/custom?
# ZSH_CUSTOM=/path/to/new-custom-folder

# Which plugins would you like to load? (plugins can be found in ~/.oh-my-zsh/plugins/*)
# Custom plugins may be added to ~/.oh-my-zsh/custom/plugins/
# Example format: plugins=(rails git textmate ruby lighthouse)
# Add wisely, as too many plugins slow down shell startup.
plugins=(git)

source $ZSH/oh-my-zsh.sh

# User configuration

# export MANPATH="/usr/local/man:$MANPATH"

# You may need to manually set your language environment
# export LANG=en_US.UTF-8

# Preferred editor for local and remote sessions
# if [[ -n $SSH_CONNECTION ]]; then
#   export EDITOR='vim'
# else
#   export EDITOR='mvim'
# fi
export VISUAL=/usr/local/bin/vim
export EDITOR="$VISUAL"

# Compilation flags
# export ARCHFLAGS="-arch x86_64"

# ssh
# export SSH_KEY_PATH="~/.ssh/rsa_id"

# Set personal aliases, overriding those provided by oh-my-zsh libs,
# plugins, and themes. Aliases can be placed here, though oh-my-zsh
# users are encouraged to define aliases within the ZSH_CUSTOM folder.
# For a full list of active aliases, run `alias`.
#
# Example aliases
# alias zshconfig="mate ~/.zshrc"
# alias ohmyzsh="mate ~/.oh-my-zsh"
export PATH=/Users/aharmon/.composer/vendor/bin:$PATH

export GITHUB_URL=https://git.rsglab.com

# Use the following alias to reset the DNS cache in OS X Yosemite v10.10 through v10.10.3:
alias dns="sudo launchctl unload -w /System/Library/LaunchDaemons/com.apple.discoveryd.plist; sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.discoveryd.plist; sudo discoveryutil mdnsflushcache"
  
# Use the following alias to reset the DNS cache in OS X Yosemite 10.10.4 and Greater:
alias dns="sudo killall -HUP mDNSResponder"
  
# If your computer doesn't connect via IPv6 by default:
alias vpn="sudo openconnect --config ~/.openconnect/config mcvam.rsglab.com"
  
# If you want to disable IPv6 (the OpenConnect client doesn't function over IPv6 as of 8/3/15 on OSX 10.10.4):
alias vpn="sudo openconnect --config ~/.openconnect/config mcvam.rsglab.com --disable-ipv6"
 
alias vpnoff='sudo kill -INT `cat ~/.openconnect/pid`; sudo route delete 205.201.132.5; sudo route delete 205.201.132.4'
alias vpnst="ps -ef | grep [o]penconnect"
alias vpnreset="vpnoff; sudo ifconfig en0 down; sudo ifconfig en0 up; sleep 5; dns"
alias maildir="cd /Volumes/casesensitive/mailchimp"
alias v-up='vagrant up'
alias v-halt='vagrant halt'
alias v-status='vagrant status'

alias ls='ls -la -tr'
alias v-reset='script/destroy ;script/setup ; script/run-tests setup'
# GITHUB STUFF

alias git='hub'
alias g='git'
alias update_shell='source ~/.zshrc'
alias fuck='g can; g pf'
alias pr='hub pull-request -F ~/.git-templates/hooks/pr_template.md --browse'
alias ssh-add='ssh-add -t 4h'

#DOTFILES

DOTFILES="$HOME/dev/dotfiles"

# TMUX
alias mux='tmuxinator'

# VI MODE FOR ZLE
bindkey -v

## jk for switching
bindkey -M viins 'jk' vi-cmd-mode


## BINDINGS
bindkey '^P' up-history
bindkey '^N' down-history
bindkey '^?' backward-delete-char
bindkey '^h' backward-delete-char
bindkey '^w' backward-kill-word
bindkey '^r' history-incremental-search-backward


#function zle-line-init zle-keymap-select {
#    VIM_PROMPT="%{$fg_bold[yellow]%} [% NORMAL]%  %{$reset_color%}"
#    RPS1="${${KEYMAP/vicmd/$VIM_PROMPT}/(main|viins)/} $EPS1"
#    zle reset-prompt
#}

## WIDGET
zle -N zle-line-init
zle -N zle-keymap-select
export KEYTIMEOUT=10

# Git Shortcuts
gheUser="alec-harmon"
gheMcFork="myfork"
setUpStream (){
    fork="myfork"
	branch=$(git branch | grep \* | cut -d ' ' -f2)
	git push --set-upstream $gheMcFork $branch 
    echo "Link to GHE branch: https://git.rsglab.com/$gheUser/mailchimp/tree/$branch"
}

openBranch (){
    branch=$(git branch | grep \* | cut -d ' ' -f2)
    open "https://git.rsglab.com/$gheUser/mailchimp/tree/$branch"
}

#bc git hooks are dumb and i hates them
gitcommit (){
	branch='['$(git branch | grep \* | cut -d ' ' -f2)'] '
	echo $branch | git commit --edit --file -
}

getbranch (){
	echo "test";
	branch=$(git branch | grep '*' | sed 's/* //')
	if [ ${branch:0:1} = '(' ] ;then
		end=${branch##* };
		branch=${end%?}
		$branch
	fi
}

export PATH=/Users/aharmon/.composer/vendor/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/Users/aharmon/.vimpkg/bin:/Users/aharmon/Library/Python/2.7/bin:/Users/aharmon/.rvm/gems/ruby-2.6.3/bin:/Users/aharmon/anaconda3/bin:$GOPATH/bin:/Users/aharmon/.rvm/scripts/rvm:/Users/aharmon/.rbenv/shims


eval "$(rbenv init -)"

export GOPATH=$HOME/go

# Neo Vim
alias vim="nvim"

#did file 
alias did="vim +'normal Go' +'r!date' ~/did.txt"

alias csql="mycli"

# Set SSH keys
#() {
#  setopt LOCAL_OPTIONS NO_NOTIFY NO_MONITOR
#  ssh-add -K -t 1d ~/.ssh/id_rsa  2>/dev/null & ssh-add -K -t 1d ~/.ssh/id_ed25519_ghe  2>/dev/null & 
#}

# Setup fzf
# ---------
if [[ ! "$PATH" == */usr/local/opt/fzf/bin* ]]; then
  export PATH="$PATH:/usr/local/opt/fzf/bin"
fi

# Auto-completion
# ---------------
[[ $- == *i* ]] && source "/usr/local/opt/fzf/shell/completion.zsh" 2> /dev/null

# Key bindings
# ------------
source "/usr/local/opt/fzf/shell/key-bindings.zsh"

# The next line updates PATH for the Google Cloud SDK.
if [ -f '/Users/aharmon/Downloads/google-cloud-sdk/path.zsh.inc' ]; then . '/Users/aharmon/Downloads/google-cloud-sdk/path.zsh.inc'; fi

# The next line enables shell command completion for gcloud.
if [ -f '/Users/aharmon/Downloads/google-cloud-sdk/completion.zsh.inc' ]; then . '/Users/aharmon/Downloads/google-cloud-sdk/completion.zsh.inc'; fi

export NAVI_PATH="/Volumes/casesensitive/mc-navi/lookup"
#export PATH="$HOME/.yarn/bin:$HOME/.config/yarn/global/node_modules/.bin:/Users/aharmon/Library/Python/3.7/bin:$PATH"
#export PATH="/Users/aharmon/.pyenv/shims/:$HOME/.yarn/bin:$HOME/.config/yarn/global/node_modules/.bin:$PATH"

#Navi Link
export NAVI_PATH="/Volumes/casesensitive/mc-navi/cheats"

export RUBY_CONFIGURE_OPTS="--with-openssl-dir=$(brew --prefix openssl@1.1)"
