# tmux-tabs

A vertical window sidebar for tmux, written in Rust. Shows all tmux windows in
a narrow left pane grouped by project, with bell/agent-ready highlights, live
terminal titles, and optional AI-generated descriptions. This is a port of the
original Python/Textual implementation.

## Usage

The launcher lives at `../scripts/tmux-tabs` (referenced from `tmux.conf`):

```
tmux-tabs                 toggle the sidebar (kill if open, else spawn)
tmux-tabs --sidebar       run the sidebar TUI (invoked internally by the split)
tmux-tabs --select prev   move the sidebar to the previous grouped window
tmux-tabs --select next   move the sidebar to the next grouped window
```

Inside the sidebar:

- **Left-click** a window to switch to it.
- **Right/middle-click** a window to open its action menu (open PR when cached /
  refresh PR / rename / kill / clear ready).
- **Mouse wheel** scrolls the window list.
- **q** quits.

Colours are read live from the tmux theme (`window-status-style`, etc.) so the
sidebar matches your current scheme. The sidebar follows focus automatically
when you switch or open windows.

## Width

The sidebar grows to fit content, then clamps to a min/max width. Defaults are
18 and 48 cells. Override them in tmux config:

```tmux
set -g @tmux-tabs-min-width 24
set -g @tmux-tabs-max-width 60
```

Missing or invalid values fall back to defaults. If max is below min, min wins.

## Build

```
cargo build --release
```

The launcher builds this automatically on first use. `setup.sh` /
`setup-linux.sh` also build it during dotfiles setup.

Optional descriptions require `llm-redactor-exec` and `pi` on `PATH`; without
them the sidebar simply omits descriptions.

Optional PR status uses `../scripts/pr-status refresh --pane <pane> --path <repo>`,
which shells out to `gh pr view` and caches results in
`~/.cache/dotfiles/pr-status`. The sidebar only reads that cache during normal
rendering. Right-click a window and choose `refresh PR` to update the cache;
when a PR URL is cached, the menu also shows `open PR`.

## Architecture

The code is split so the interesting behaviour is testable without a terminal:

- `model.rs` — pure data + grouping/ordering/width/label logic.
- `layout.rs` — turns windows + menu state into addressable visual lines, each
  carrying an optional click target.
- `app.rs` — the pure `handle_event(state, Event) -> [Action]` reducer where all
  mouse/scroll/click/key behaviour lives.
- `tmux.rs` — a `Tmux` trait (real impl + fakeable) and window/pane gathering.
- `theme.rs` — parses tmux styles into colours.
- `control.rs` — toggle / `--select` / move-sidebar commands.
- `descriptions.rs` — background description generation + cache.
- `runtime.rs` — the [ratatui](https://ratatui.rs) render loop (over a crossterm
  backend): mouse capture, event translation, double-buffered drawing.

Rendering uses ratatui; `unicode-width` handles emoji/CJK padding so rows align.
`Cargo.lock` pins a couple of transitive deps to versions that still build on
Rust 1.75 (the toolchain in use) — bump them once the toolchain moves up.

## Tests

```
cargo test                       # everything
cargo test --test integration    # pure model/layout/event logic
cargo test --test e2e            # spawns a real tmux server + this binary
```

- **Integration tests** drive the pure reducer and model directly: clicks switch
  windows, the wheel moves the viewport (with bounds), right-click toggles the
  action menu, kill requires confirmation, grouping/width are correct.
- **End-to-end tests** start an isolated tmux server, launch the real binary in a
  split, and inject genuine SGR mouse sequences (`tmux send-keys -H`) for clicks,
  scrolling, and right-clicks — asserting the active window changes, the viewport
  scrolls, the menu appears, and `q` quits.
