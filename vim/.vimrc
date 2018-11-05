" show existing tab with 4 spaces width
set tabstop=4
" when indenting with '>', use 4 spaces width
set shiftwidth=4
" On pressing tab, insert 4 spaces
set expandtab
"colorscheme badwolf 
set number "line numbers
set cursorline "cursor underline
"Searching 
set incsearch           " search as characters are entered
set hlsearch            " highlight matches
nnoremap <leader><space> :nohlsearch<CR>
" turn off search highlight

" turn off search highlight
nnoremap <leader><space> :nohlsearch<CR>

set nocompatible              " be iMproved, required
filetype off                  " required <<========== We can turn it on later

set spelllang=en
"update on file change
:set autoread
" set the runtime path to include Vundle and initialize
set rtp+=~/.vim/bundle/Vundle.vim
call vundle#begin()
" alternatively, pass a path where Vundle should install plugins
"call vundle#begin('~/some/path/here')

" let Vundle manage Vundle, required
Plugin 'VundleVim/Vundle.vim'

" <============================================>
" Specify the plugins you want to install here.
" We'll come on that later
" <============================================>
" All of your Plugins must be added before the following line

" Stuff For Go
" <============================================>
" Specify the plugins you want to install here.
" We'll come on that later
" <============================================>
" All of your Plugins must be added before the following line
Plugin 'fatih/vim-go'

" Json
Plugin 'elzr/vim-json'

" Php Handling
Plugin 'shawncplus/phpcomplete.vim'

" Gundo
Plugin 'sjl/gundo.vim'
if has('python3')
    let g:gundo_prefer_python3 = 1
endif

" Ctrl-p
Plugin 'ctrlp.vim'

" Nerd tree
Plugin 'scrooloose/nerdtree'
map <C-n> :NERDTreeToggle<CR>
autocmd StdinReadPre * let s:std_in=1
autocmd VimEnter * if argc() == 0 && !exists("s:std_in") | NERDTree | endif

"Plugins end
call vundle#end()            " required
filetype plugin indent on    " required
" To ignore plugin indent changes, instead use:
"filetype plugin on
"
" Brief help
" :PluginList       - lists configured plugins
" :PluginInstall    - installs plugins; append `!` to update or just :PluginUpdate
" :PluginSearch foo - searches for foo; append `!` to refresh local cache
" :PluginClean      - confirms removal of unused plugins; append `!` to auto-approve removal
"
" see :h vundle for more details or wiki for FAQ
" Put your non-Plugin stuff after this line
" Put the rest of your .vimrc file here

syntax on

"leader to ,
let mapleader=","

"set escape to 'jk'
inoremap jk <esc>

"grundle undo
nnoremap <leader>u :GundoToggle<CR>

"reload vim
:command ReloadVim :source ~/.vimrc

colorscheme desert
" presets
" list auto complete
set wildmode=longest,list,full
set wildmenu
" backspace
set backspace=indent,eol,start

" Persistent History
set undodir=~/.vim/undodir
set undofile   " Maintain undo history between sessions

" GET WRECKED TABS
filetype plugin indent on


