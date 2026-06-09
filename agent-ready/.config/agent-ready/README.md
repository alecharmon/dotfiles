# agent-ready

Generic readiness markers for tmux agent panes.

- Marker CLI: `~/dev/dotfiles/scripts/agent-ready`
- Claude hook adapter: `~/dev/dotfiles/scripts/claude-agent-ready-hook`
- Pi adapter: `~/.config/agent-ready/pi-extension.ts`
- Marker files: `${XDG_CACHE_HOME:-~/.cache}/dotfiles/agent-ready/*.json`

The tmux sidebar (`scripts/tmux-tabs`) reads these marker files and shows `🔔` next to a window when any pane in that window is marked ready.

## Manual test

Inside tmux:

```bash
~/dev/dotfiles/scripts/agent-ready set --source manual
~/dev/dotfiles/scripts/agent-ready list
~/dev/dotfiles/scripts/agent-ready clear
```

## Pi

Add this extension path to `~/.pi/agent/settings.json`:

```json
{
  "extensions": ["~/.config/agent-ready/pi-extension.ts"]
}
```

If `extensions` already exists, append the path.

## Claude Code

Merge the hook entries from:

```text
~/.config/agent-ready/claude-hooks.example.json
```

into `~/.claude/settings.json` or `~/.claude/settings.local.json`.
