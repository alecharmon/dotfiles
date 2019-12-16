"leader to space
let mapleader=" "

"set escape to 'jk'
inoremap jk <esc>

" vim plug
call plug#begin('~/.vim/plugged')

" tabs
Plug 'https://github.com/webdevel/tabulous'

" FZF
Plug '/usr/local/opt/fzf'
Plug 'junegunn/fzf.vim'

" Show FZF in a window
function! s:fzf_handler(lines) abort
  if empty(a:lines)
    return
  endif
  let cmd = get({ 'ctrl-t': 'tabedit',
                \ 'ctrl-x': 'split',
                \ 'ctrl-v': 'vsplit' }, remove(a:lines, 0), 'e')
  for item in a:lines
    execute cmd escape(item, ' %#\')
  endfor
endfunction

nmap <C-p> :call fzf#run({
  \ 'options': '--expect=ctrl-t,ctrl-x,ctrl-v',
  \ 'down':      '40%',
  \ 'sink*':   function('<sid>fzf_handler')})<cr>

"nnoremap <C-R> :History: <CR>

"vim wiki
Plug 'vimwiki/vimwiki'
let g:vimwiki_list = [{'path': '~/wiki/', 'path_html': '~/wiki-html/', 'syntax': 'markdown', 'ext': '.md'}]

"NERDtree
Plug 'scrooloose/nerdtree'
""NERDtree and FZF conflicts
nnoremap <silent> <expr> <Leader><Leader> (expand('%') =~ 'NERD_tree' ? "\<c-w>\<c-w>" : '').":FZF\<cr>"
 
""Hidden files
let NERDTreeShowHidden=1
noremap <leader>1 :NERDTree<CR>

"Commenting out tool (eyeroll)
Plug 'scrooloose/nerdcommenter'
inoremap <C-/> :NERDCommenterToggle<CR>

"COC
Plug 'neoclide/coc.nvim', {'branch': 'release'}
" Use <c-space> to trigger completion.
inoremap <silent><expr> <c-space> coc#refresh()

" Use <cr> to confirm completion, `<C-g>u` means break undo chain at current position.
" Coc only does snippet and additional edit on confirm.
inoremap <expr> <cr> pumvisible() ? "\<C-y>" : "\<C-g>u\<CR>"

" Remap keys for gotos
nmap <silent> gd <Plug>(coc-definition)
nmap <silent> gy <Plug>(coc-type-definition)
nmap <silent> gi <Plug>(coc-implementation)
nmap <silent> gr <Plug>(coc-references)
" nav autocomplete
inoremap <expr> <Tab> pumvisible() ? "\<C-n>" : "\<Tab>"
inoremap <expr> <S-Tab> pumvisible() ? "\<C-p>" : "\<S-Tab>"
inoremap <expr> <cr> pumvisible() ? "\<C-y>" : "\<C-g>u\<CR>"
inoremap <silent><expr> <cr> pumvisible() ? coc#_select_confirm() : "\<C-g>u\<CR>"
inoremap <silent><expr> <cr> pumvisible() ? coc#_select_confirm() : "\<C-g>u\<CR>\<c-r>=coc#on_enter()\<CR>"

"White Space
Plug 'ntpeters/vim-better-whitespace'

"Ferrit 
Plug 'wincent/ferret'

"ALE
Plug 'dense-analysis/ale'
let g:ale_fixers = {'php': ['php_cs_fixer']}

"let g:ale_php_phpcs_standard = '/Volumes/casesensitive/mc-codesniffer-ruleset/MCStandard'
" Plugin End
call plug#end()
  
" show existing tab with 4 spaces width
set tabstop=4
" when indenting with '>', use 4 spaces width
set shiftwidth=4
" On pressing tab, insert 4 spaces
set expandtab
" Cursor underline
set cursorline
"Hidden Buffers
set hidden
"Searching
set incsearch           " search as characters are entered
set hlsearch            " highlight matches
:set smartcase          " Only case sensitive for uppercase chars

" turn off search highlight
nnoremap <leader>? :nohlsearch<CR>
" Smart case search
:set smartcase

" Spelling
set spelllang=en
noremap <leader>s :setlocal spell! spelllang=en_us<CR>

" Update on file change
:set autoread

" Syntax
syntax on

" turn hybrid line numbers on
:set number relativenumber
:set nu rnu

"reload vim
:command! Reload :source ~/.vimrc
nnoremap <leader>r :Reload<CR>


"turn off arrow keys
"" Left
nnoremap <Left> :echo "No left for you!"<CR>
vnoremap <Left> :<C-u>echo "No left for you!"<CR>
inoremap <Left> <C-o>:echo "No left for you!"<CR>
"" Right
nnoremap <Right> :echo "No right for you!"<CR>
vnoremap <Right> :<C-u>echo "No right for you!"<CR>
inoremap <Right> <C-o>:echo "No right for you!"<CR>
"" Up
nnoremap <Up> :echo "No up for you!"<CR>
vnoremap <Up> :<C-u>echo "No up for you!"<CR>
inoremap <Up> <C-o>:echo "No up for you!"<CR>
"" Down
nnoremap <Down> :echo "No down for you!"<CR>
vnoremap <Down> :<C-u>echo "No down for you!"<CR>
inoremap <Down> <C-o>:echo "No down for you!"<CR>

" Window nav
nnoremap <leader>j <C-W><C-J>
nnoremap <leader>k <C-W><C-K>
nnoremap <leader>h <C-W><C-H>
nnoremap <leader>l <C-W><C-L>

"Splits windows
nnoremap <leader>- :sp<CR>
nnoremap <leader>\ :vsp<CR>

" Navigating Tabs 
nnoremap <C-[> :tabprevious <CR>
nnoremap <C-]> :tabnext <CR>
nnoremap <leader>[ :tabprevious <CR>
nnoremap <leader>] :tabnext <CR>

"Close Tab
"nnoremap <C-w> :tabclose<CR>

"Create Tab
nnoremap <leader>t :tabedit 
nnoremap <C-t> :tabedit

nnoremap <silent> <leader><space> :Files<CR>
nnoremap <silent> <leader>a :Buffers<CR>
nnoremap <silent> <leader>A :Windows<CR><Paste>

nnoremap J :+5 <CR>
nnoremap K :-5 <CR>

"Go to 
colorscheme elflord
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

" Trying to solve paste mode woes
let &t_SI .= "\<Esc>[?2004h"
let &t_EI .= "\<Esc>[?2004l"

set pastetoggle=<F2>

inoremap <special> <expr> <Esc>[200~ XTermPasteBegin()

function! XTermPasteBegin()
    set pastetoggle=<Esc>[201~
    set paste
    return ""
endfunction

set clipboard=unnamed
