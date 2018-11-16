" vim plug
call plug#begin('~/.vim/plugged')

" Auto Complete
Plug 'ncm2/ncm2'
Plug 'roxma/nvim-yarp'

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


"enable ncm2 for all buffers
autocmd BufEnter * call ncm2#enable_for_buffer()
" IMPORTANTE: :help Ncm2PopupOpen for more 0information
set completeopt=noinsert,menuone,noselect

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
nnoremap <leader><space> :nohlsearch<CR>

" turn off search highlight
nnoremap <leader><space> :nohlsearch<CR>

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

"set escape to 'jk'
inoremap jk <esc>

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
