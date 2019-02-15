"set escape to 'jk'
map! jk <esc>

" vim plug
call plug#begin('~/.vim/plugged')

" ctl p
Plug 'https://github.com/ctrlpvim/ctrlp.vim'

" Nerd Tree
Plug 'https://github.com/scrooloose/nerdtree'
"" Open if no file specified
autocmd StdinReadPre * let s:std_in=1
autocmd VimEnter * if argc() == 0 && !exists("s:std_in") | NERDTree | endif

" Smooth Scrolling
Plug 'https://github.com/terryma/vim-smooth-scroll'
noremap <silent> <c-u> :call smooth_scroll#up(&scroll, 0, 2)<CR>
noremap <silent> <c-d> :call smooth_scroll#down(&scroll, 0, 2)<CR>
noremap <silent> <c-b> :call smooth_scroll#up(&scroll*2, 0, 4)<CR>
noremap <silent> <c-f> :call smooth_scroll#down(&scroll*2, 0, 4)<CR>

" Vim Go
Plug 'fatih/vim-go', { 'do': ':GoUpdateBinaries' }

" CSV
Plug 'chrisbra/csv.vim'

" Auto Complete
Plug 'autozimu/LanguageClient-neovim', {
    \ 'branch': 'next',
    \ 'do': 'bash install.sh',
    \ }

" Required for operations modifying multiple buffers like rename.
 set hidden
"
 let g:LanguageClient_serverCommands = {
     \ 'go': ['go-langserver'],
\ }

nnoremap <silent> K :call LanguageClient#textDocument_hover()<CR>
nnoremap <silent> gd :call LanguageClient#textDocument_definition()<CR>
nnoremap <silent> <leader>2 :call LanguageClient#textDocument_rename()<CR>

let g:completor_refresh_always = 0 "avoid flickering
let g:completor_python_omni_trigger = ".*"
set formatexpr=LanguageClient_textDocument_rangeFormatting()
set omnifunc=LanguageClient#complete

" Git
Plug 'airblade/vim-gitgutter'


" Plugin End
call plug#end()

" show existing tab with 4 spaces width
set tabstop=4
" when indenting with '>', use 4 spaces width
set shiftwidth=4
" On pressing tab, insert 4 spaces
set expandtab
"colorscheme badwolf set number "line numbers
set cursorline "cursor underline
"Searching
set incsearch           " search as characters are entered
set hlsearch            " highlight matches

" turn off search highlight
nnoremap <leader>? :nohlsearch<CR>

set nocompatible              " be iMproved, required
filetyp off                  " required <<========== We can turn it on later

set spelllang=en

"update on file change
:set autoread

" Syntax
syntax on

" Line num
:set number

"leader to ,
let mapleader=" "


"grundle undo
nnoremap <leader>u :GundoToggle<CR>

"reload vim
:command! ReloadVim :source ~/.vimrc
nnoremap <leader>rv :ReloadVim<CR>

" Window nav
nnoremap <leader>j <C-W><C-J>
nnoremap <leader>k <C-W><C-K>
nnoremap <leader>h <C-W><C-H>
nnoremap <leader>l <C-W><C-L>

"Splits windows
nnoremap <leader>- :sp<CR>
nnoremap <leader>\ :vsp<CR>

"Close Tab
nnoremap <leader>w :q<CR>

"Create Tab
nnoremap <leader>t :tabedit 

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

" Vimdiff
:command! UseRemote diffg REMOTE
:command! UseBase diffg BASE
:command! UseLocal diffg LOCAL

" Navigating Buffers
:nnoremap <leader>[ :bprev <CR>
:nnoremap <leader>] :bnext <CR>
