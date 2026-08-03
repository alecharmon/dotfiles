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
tmux-tabs --list-json     print agent-readable window metadata
tmux-tabs --find <ref>    resolve a tab reference to matching metadata
tmux-tabs --capture <ref> capture the referenced tab's active pane
tmux-tabs --send <ref> <text>
                        paste text into the referenced tab and press Enter
tmux-tabs --paste <ref> <text>
                        paste text into the referenced tab without Enter
tmux-tabs --section create <name>
                        create a persistent custom sidebar section
tmux-tabs --section add <name> <ref>
                        add the referenced tab to a section
tmux-tabs --section remove <name> <ref>
                        remove the referenced tab from a section
tmux-tabs --section delete <name>
                        delete a custom section
tmux-tabs --section list-json
                        print the persistent section layout
tmux-tabs --tab set-status <ref> <status>
                        set durable status for a tab
tmux-tabs --tab set-resume <ref> <command>
                        set the command needed to resume work in a tab
tmux-tabs --tab status-json <ref>
                        print durable status/resume metadata for a tab
tmux-tabs --tabs-json     print the global durable tab registry
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

## Custom sections

Agents can create persistent sidebar sections that override folder-derived
project groups. Section assignments are saved per tmux session under the
existing tmux-tabs cache directory. Commands accept the same friendly tab
references as capture/send, then store stable tmux window ids internally.

```bash
tmux-tabs --section create "Review"
tmux-tabs --section add "Review" '@tab:sidebar'
tmux-tabs --section remove "Review" '@tab:tests'
tmux-tabs --section delete "Review"
tmux-tabs --section list-json
```

A tab can belong to one custom section at a time; adding it to a new section
removes it from any previous custom section. Tabs not assigned to a custom
section keep the existing folder-based grouping.

## Durable tab status and resume metadata

`tmux-tabs` maintains a global durable registry at `~/.tmux-tabs.yml`. Each live
window gets a generated persistent tab id stored in the tmux window option
`@tmux-tabs-tab-id`; the registry records the current tmux ids, name, path,
command, status, resume command, and last-seen timestamp. The sidebar refreshes
live metadata periodically, and CLI reads also refresh it.

Agents can explicitly set status and resume commands:

```bash
tmux-tabs --tab set-status '@tab:agent' waiting_for_input
tmux-tabs --tab set-resume '@tab:agent' 'pi resume abc123'
tmux-tabs --tab status-json '@tab:agent'
tmux-tabs --tabs-json
```

Supported statuses are `unknown`, `running`, `waiting_for_input`, `blocked`,
`done`, and `dead`.

## Agent tab references

Agents and scripts can reference other tabs through the CLI. A reference can be a
window index, generated tab name, description, title, path, or `@tab:<query>`.
Matching prefers exact index/name/description/title/path matches, then partial
matches.

Examples:

```bash
tmux-tabs --list-json
tmux-tabs --find '@tab:sidebar'
tmux-tabs --capture '@tab:sidebar'
tmux-tabs --send '@tab:tests' 'cargo test --manifest-path tmux-tabs/Cargo.toml'
tmux-tabs --paste '@tab:notes' 'remember to check release build'
```

Optional PR status uses `../scripts/pr-status refresh --pane <pane> --path <repo>`,
which shells out to `gh pr view` and caches results in
`~/.cache/dotfiles/pr-status`. The sidebar only reads that cache during normal
rendering. PR rows show a colored dot plus compact state (`CI…`, `failed`,
`ready`, `draft`, `changes`, `merged`, `closed`, or `open`). While the sidebar is
open it asynchronously refreshes PR state for every visible tmux window every 60
seconds. Left-click a PR row to open it; right-click a window and choose
`refresh PR` to update the cache immediately; when a PR URL is cached, the menu
also shows `open PR`.

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
  action menu, kill fires immediately, double-click opens PRs, grouping/width
  are correct.
- **End-to-end tests** start an isolated tmux server, launch the real binary in a
  split, and inject genuine SGR mouse sequences (`tmux send-keys -H`) for clicks,
  scrolling, and right-clicks — asserting the active window changes, the viewport
  scrolls, the menu appears, and `q` quits.
